use super::validation::{layout, normalized, typed};
use super::{
    MeshAnimationClip, MeshAnimationInterpolation, MeshAnimationProperty, MeshAnimationTrack,
    MeshGeometryAsset,
    pose::{GeometryDefinition, Node, Primitive, Skin},
    topology,
};
use base64::Engine;
use glam::{Mat4, Quat, Vec3};
use gltf::accessor::{DataType as T, Dimensions as D};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const MAX_BUFFER_BYTES: usize = 128 * 1024 * 1024;
const MAX_NODES: usize = 65_536;
const MAX_VERTICES: usize = 4_000_000;

pub fn load_gltf_geometry(path: &Path) -> Result<MeshGeometryAsset, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_BUFFER_BYTES as u64 {
        return Err("model file exceeds import budget".into());
    }
    let document = gltf::Gltf::open(path).map_err(|e| e.to_string())?;
    let buffers = read_buffers(&document, path)?;
    let (mut definition, active) = read_nodes(&document)?;
    for skin in document.skins() {
        if let Some(accessor) = skin.inverse_bind_matrices() {
            typed(accessor, D::Mat4, &[T::F32])?;
        }
        let joints = skin.joints().map(|j| j.index()).collect::<Vec<_>>();
        let reader = skin.reader(|b| buffers.get(b.index()).map(Vec::as_slice));
        let mut inverse_bind = if skin.inverse_bind_matrices().is_some() {
            reader
                .read_inverse_bind_matrices()
                .ok_or("invalid inverse bind matrices")?
                .map(|m| Mat4::from_cols_array_2d(&m))
                .collect::<Vec<_>>()
        } else {
            vec![Mat4::IDENTITY; joints.len()]
        };
        if joints.is_empty()
            || joints.len() > inverse_bind.len()
            || joints.iter().collect::<BTreeSet<_>>().len() != joints.len()
            || inverse_bind
                .iter()
                .any(|m| !m.is_finite() || m.row(3) != glam::Vec4::W)
        {
            return Err("invalid skin joints or inverse bind matrices".into());
        }
        inverse_bind.truncate(joints.len());
        let ancestors = |mut node: usize| {
            let mut chain = BTreeSet::from([node]);
            while let Some(parent) = definition.nodes[node].parent {
                chain.insert(parent);
                node = parent;
            }
            chain
        };
        let mut common = ancestors(joints[0]);
        for &joint in &joints[1..] {
            let chain = ancestors(joint);
            common.retain(|node| chain.contains(node));
        }
        if common.is_empty()
            || skin
                .skeleton()
                .is_some_and(|node| !common.contains(&node.index()))
        {
            return Err("skin joints must share the declared hierarchy root".into());
        }
        definition.skins.push(Skin {
            joints,
            inverse_bind,
        });
    }
    let dropped = read_primitives(&document, &buffers, &active, &mut definition)?;
    let animations = read_animations(&document, &buffers, &definition)?;
    let bind = definition.sample(None, 0.0)?;
    if bind.positions.is_empty() || bind.indices.is_empty() {
        return Err("model contains no triangles".into());
    }
    let min = bind
        .positions
        .iter()
        .map(|p| Vec3::from_array(*p))
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = bind
        .positions
        .iter()
        .map(|p| Vec3::from_array(*p))
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let scale = 2.0 / (max - min).max_element().max(1e-6);
    definition.normalization =
        Mat4::from_scale(Vec3::splat(scale)) * Mat4::from_translation(-(min + max) * 0.5);
    let bind = definition.sample(None, 0.0)?;
    Ok(MeshGeometryAsset {
        positions: bind.positions,
        indices: bind.indices,
        dropped_degenerate_triangles: dropped,
        animations,
        definition,
    })
}

fn read_buffers(document: &gltf::Gltf, path: &Path) -> Result<Vec<Vec<u8>>, String> {
    let root = path
        .parent()
        .ok_or("model has no parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let mut buffers = Vec::new();
    let mut total = 0usize;
    for buffer in document.buffers() {
        if buffer.length() > MAX_BUFFER_BYTES {
            return Err("model buffer exceeds import budget".into());
        }
        let bytes = match buffer.source() {
            gltf::buffer::Source::Bin => document.blob.clone().ok_or("missing GLB buffer")?,
            gltf::buffer::Source::Uri(uri) if uri.starts_with("data:") => {
                let (header, data) = uri.split_once(',').ok_or("invalid data URI")?;
                if !matches!(
                    header,
                    "data:application/octet-stream;base64" | "data:application/gltf-buffer;base64"
                ) {
                    return Err("unsupported buffer data URI".into());
                }
                if data.len() > MAX_BUFFER_BYTES * 4 / 3 + 4 {
                    return Err("model buffer exceeds import budget".into());
                }
                base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|e| format!("buffer data URI: {e}"))?
            }
            gltf::buffer::Source::Uri(uri) => {
                let decoded = percent_encoding::percent_decode_str(uri)
                    .decode_utf8()
                    .map_err(|e| e.to_string())?;
                let file = root
                    .join(decoded.as_ref())
                    .canonicalize()
                    .map_err(|e| format!("buffer {uri}: {e}"))?;
                if !file.starts_with(&root) {
                    return Err("buffer escapes model directory".into());
                }
                if std::fs::metadata(&file).map_err(|e| e.to_string())?.len()
                    > MAX_BUFFER_BYTES as u64
                {
                    return Err("model buffer exceeds import budget".into());
                }
                std::fs::read(file).map_err(|e| e.to_string())?
            }
        };
        total = total
            .checked_add(bytes.len())
            .ok_or("model buffers exceed import budget")?;
        if total > MAX_BUFFER_BYTES {
            return Err("model buffers exceed import budget".into());
        }
        if bytes.len() < buffer.length() {
            return Err("truncated glTF buffer".into());
        }
        buffers.push(bytes);
    }
    for view in document.views() {
        if view
            .offset()
            .checked_add(view.length())
            .is_none_or(|end| end > view.buffer().length())
        {
            return Err("buffer view exceeds buffer".into());
        }
    }
    layout(document, &buffers)?;
    Ok(buffers)
}

fn read_nodes(document: &gltf::Gltf) -> Result<(GeometryDefinition, Vec<bool>), String> {
    let count = document.nodes().len();
    if count > MAX_NODES {
        return Err("model has too many nodes".into());
    }
    let mut nodes = Vec::with_capacity(count);
    for node in document.nodes() {
        let matrix = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => Some(Mat4::from_cols_array_2d(&matrix)),
            _ => None,
        };
        let (translation, rotation, scale) = if matrix.is_some() {
            ([0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3])
        } else {
            node.transform().decomposed()
        };
        let (translation, rotation, scale) = (
            Vec3::from_array(translation),
            Quat::from_array(rotation),
            Vec3::from_array(scale),
        );
        if !translation.is_finite()
            || !scale.is_finite()
            || !rotation.is_finite()
            || !rotation.length_squared().is_finite()
            || rotation.length_squared() < 1e-12
            || matrix.is_some_and(|m| !m.is_finite())
        {
            return Err("invalid node transform".into());
        }
        let target_counts = node
            .mesh()
            .map(|mesh| {
                mesh.primitives()
                    .map(|p| p.morph_targets().len())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if target_counts.len() > 1 {
            return Err("mesh primitives have inconsistent morph target counts".into());
        }
        let targets = target_counts.first().copied().unwrap_or(0);
        let weights = node
            .weights()
            .or_else(|| node.mesh().and_then(|m| m.weights()))
            .map(<[f32]>::to_vec)
            .unwrap_or_else(|| vec![0.0; targets]);
        if weights.len() != targets || weights.iter().any(|w| !w.is_finite()) {
            return Err("invalid default morph weights".into());
        }
        nodes.push(Node {
            parent: None,
            path: node
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Node {}", node.index() + 1)),
            translation,
            rotation: rotation.normalize(),
            scale,
            matrix,
            weights,
        });
    }
    for node in document.nodes() {
        for child in node.children() {
            if nodes[child.index()].parent.replace(node.index()).is_some() {
                return Err("node has multiple parents".into());
            }
        }
    }
    // Iterative traversal bounds stack usage and rejects cycles outside the active scene too.
    let mut pending = (0..count)
        .filter(|&i| nodes[i].parent.is_none())
        .collect::<Vec<_>>();
    let mut order = Vec::with_capacity(count);
    let children = document
        .nodes()
        .map(|n| n.children().map(|c| c.index()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    while let Some(index) = pending.pop() {
        if let Some(parent) = nodes[index].parent {
            nodes[index].path = format!("{}/{}", nodes[parent].path, nodes[index].path);
        }
        if nodes[index].path.len() > 16_384 {
            return Err("node hierarchy is too deep".into());
        }
        order.push(index);
        pending.extend(&children[index]);
    }
    if order.len() != count {
        return Err("cyclic node hierarchy".into());
    }
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or("model has no scene")?;
    let mut active = vec![false; count];
    for root_node in scene.nodes() {
        if nodes[root_node.index()].parent.is_some() {
            return Err("scene root has a parent".into());
        }
        pending.push(root_node.index());
    }
    while let Some(index) = pending.pop() {
        active[index] = true;
        pending.extend(&children[index]);
    }
    Ok((
        GeometryDefinition {
            nodes,
            order,
            normalization: Mat4::IDENTITY,
            ..Default::default()
        },
        active,
    ))
}

fn read_primitives(
    document: &gltf::Gltf,
    buffers: &[Vec<u8>],
    active: &[bool],
    definition: &mut GeometryDefinition,
) -> Result<usize, String> {
    let mut dropped = 0;
    let mut vertices = 0usize;
    for node in document.nodes().filter(|n| active[n.index()]) {
        let Some(mesh) = node.mesh() else { continue };
        let skin = node.skin().map(|s| s.index());
        if skin.is_some_and(|s| definition.skins[s].joints.iter().any(|&j| !active[j])) {
            return Err("skin joint is outside the active scene".into());
        }
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err("geometry importer requires triangles".into());
            }
            typed(
                primitive
                    .get(&gltf::Semantic::Positions)
                    .ok_or("primitive has no positions")?,
                D::Vec3,
                &[T::F32],
            )?;
            if let Some(accessor) = primitive.indices() {
                typed(accessor, D::Scalar, &[T::U8, T::U16, T::U32])?;
            }
            for (semantic, accessor) in primitive.attributes() {
                match semantic {
                    gltf::Semantic::Joints(_) => {
                        if accessor.normalized() {
                            return Err("joint indices cannot be normalized".into());
                        }
                        typed(accessor, D::Vec4, &[T::U8, T::U16])?;
                    }
                    gltf::Semantic::Weights(_) => {
                        normalized(accessor, D::Vec4, &[T::U8, T::U16, T::F32])?
                    }
                    _ => (),
                }
            }
            for target in primitive.morph_targets() {
                for accessor in [target.positions(), target.normals(), target.tangents()]
                    .into_iter()
                    .flatten()
                {
                    typed(accessor, D::Vec3, &[T::F32])?;
                }
            }
            let reader = primitive.reader(|b| buffers.get(b.index()).map(Vec::as_slice));
            let positions = reader
                .read_positions()
                .ok_or("primitive has no valid positions")?
                .map(Vec3::from_array)
                .collect::<Vec<_>>();
            vertices = vertices
                .checked_add(positions.len())
                .ok_or("model has too many vertices")?;
            if vertices > MAX_VERTICES {
                return Err("model has too many vertices".into());
            }
            if positions.is_empty() || positions.iter().any(|p| !p.is_finite()) {
                return Err("invalid model geometry".into());
            }
            let indices = if primitive.indices().is_some() {
                reader
                    .read_indices()
                    .ok_or("invalid index accessor")?
                    .into_u32()
                    .collect()
            } else {
                (0..positions.len() as u32).collect::<Vec<_>>()
            };
            if indices.len() % 3 != 0 || indices.iter().any(|&i| i as usize >= positions.len()) {
                return Err("invalid triangle indices".into());
            }
            let mut morph_targets = Vec::new();
            for (target, (deltas, _, _)) in
                primitive.morph_targets().zip(reader.read_morph_targets())
            {
                let deltas = if target.positions().is_some() {
                    deltas
                        .ok_or("invalid morph positions")?
                        .map(Vec3::from_array)
                        .collect::<Vec<_>>()
                } else {
                    vec![Vec3::ZERO; positions.len()]
                };
                if deltas.len() != positions.len() || deltas.iter().any(|p| !p.is_finite()) {
                    return Err("invalid morph target geometry".into());
                }
                morph_targets.push(deltas);
            }
            let mut influences = vec![Vec::new(); positions.len()];
            if let Some(skin_index) = skin {
                let joint_sets = primitive
                    .attributes()
                    .filter_map(|(semantic, _)| {
                        if let gltf::Semantic::Joints(set) = semantic {
                            Some(set)
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();
                let weight_sets = primitive
                    .attributes()
                    .filter_map(|(semantic, _)| {
                        if let gltf::Semantic::Weights(set) = semantic {
                            Some(set)
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();
                if joint_sets.is_empty() || joint_sets != weight_sets {
                    return Err("skin requires matching joint/weight sets".into());
                }
                if joint_sets.iter().copied().ne(0..joint_sets.len() as u32) {
                    return Err("skin attribute sets must be consecutive from zero".into());
                }
                for set in joint_sets {
                    let joints = reader
                        .read_joints(set)
                        .ok_or("invalid joint accessor")?
                        .into_u16()
                        .collect::<Vec<_>>();
                    let weights = reader
                        .read_weights(set)
                        .ok_or("invalid weight accessor")?
                        .into_f32()
                        .collect::<Vec<_>>();
                    if joints.len() != positions.len() || weights.len() != positions.len() {
                        return Err("skin attribute counts disagree".into());
                    }
                    for (index, (joints, weights)) in joints.iter().zip(weights).enumerate() {
                        for (&joint, weight) in joints.iter().zip(weights) {
                            if joint as usize >= definition.skins[skin_index].joints.len()
                                || !weight.is_finite()
                                || weight < 0.0
                            {
                                return Err("invalid skin influence".into());
                            }
                            if weight > 0.0 {
                                influences[index].push((joint as usize, weight));
                            }
                        }
                    }
                }
                for vertex in &mut influences {
                    let mut merged = BTreeMap::<usize, f32>::new();
                    for &(joint, weight) in vertex.iter() {
                        *merged.entry(joint).or_default() += weight;
                    }
                    let sum = merged.values().sum::<f32>();
                    if !sum.is_finite() || sum <= 0.0 {
                        return Err("skin weights have zero or invalid sum".into());
                    }
                    *vertex = merged
                        .into_iter()
                        .map(|(joint, weight)| (joint, weight / sum))
                        .collect();
                }
            }
            let mut primitive = Primitive {
                node: node.index(),
                positions,
                indices,
                morph_targets,
                skin,
                influences,
            };
            dropped += topology::weld(&mut primitive)?;
            definition.primitives.push(primitive);
        }
    }
    Ok(dropped)
}

fn read_animations(
    document: &gltf::Gltf,
    buffers: &[Vec<u8>],
    definition: &GeometryDefinition,
) -> Result<Vec<MeshAnimationClip>, String> {
    let mut animations = Vec::new();
    for animation in document.animations() {
        let mut tracks = Vec::new();
        let mut targets = BTreeSet::new();
        let mut duration = 0.0f32;
        for channel in animation.channels() {
            let node = channel.target().node().index();
            let property = match channel.target().property() {
                gltf::animation::Property::Translation => MeshAnimationProperty::Translation,
                gltf::animation::Property::Rotation => MeshAnimationProperty::Rotation,
                gltf::animation::Property::Scale => MeshAnimationProperty::Scale,
                gltf::animation::Property::MorphTargetWeights => MeshAnimationProperty::Weights,
            };
            if property != MeshAnimationProperty::Weights && definition.nodes[node].matrix.is_some()
            {
                return Err("animation cannot target a matrix node".into());
            }
            if !targets.insert((node, property)) {
                return Err("animation has duplicate node/property channels".into());
            }
            let interpolation = match channel.sampler().interpolation() {
                gltf::animation::Interpolation::Step => MeshAnimationInterpolation::Step,
                gltf::animation::Interpolation::Linear => MeshAnimationInterpolation::Linear,
                gltf::animation::Interpolation::CubicSpline => {
                    MeshAnimationInterpolation::CubicSpline
                }
            };
            let reader = channel.reader(|b| buffers.get(b.index()).map(Vec::as_slice));
            typed(channel.sampler().input(), D::Scalar, &[T::F32])?;
            match property {
                MeshAnimationProperty::Weights | MeshAnimationProperty::Rotation => normalized(
                    channel.sampler().output(),
                    if property == MeshAnimationProperty::Weights {
                        D::Scalar
                    } else {
                        D::Vec4
                    },
                    &[T::F32, T::I8, T::U8, T::I16, T::U16],
                )?,
                _ => typed(channel.sampler().output(), D::Vec3, &[T::F32])?,
            }
            let times = reader
                .read_inputs()
                .ok_or("invalid animation timestamps")?
                .collect::<Vec<_>>();
            let (dimension, values): (usize, Vec<f32>) =
                match reader.read_outputs().ok_or("invalid animation output")? {
                    gltf::animation::util::ReadOutputs::Translations(v)
                    | gltf::animation::util::ReadOutputs::Scales(v) => (3, v.flatten().collect()),
                    gltf::animation::util::ReadOutputs::Rotations(v) => {
                        (4, v.into_f32().flatten().collect())
                    }
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(v) => {
                        (definition.nodes[node].weights.len(), v.into_f32().collect())
                    }
                };
            let track = MeshAnimationTrack {
                node,
                node_path: definition.nodes[node].path.clone(),
                property,
                interpolation,
                keyframes: times.len(),
                times: times.into(),
                values: values.into(),
                dimension,
            };
            track.validate().map_err(|e| {
                format!(
                    "animation {} ({}): {e}",
                    animation.index() + 1,
                    track.node_path
                )
            })?;
            duration = duration.max(*track.times.last().unwrap());
            tracks.push(track);
        }
        animations.push(MeshAnimationClip {
            index: animation.index(),
            name: animation
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Animation {}", animation.index() + 1)),
            duration_seconds: duration,
            tracks,
        });
    }
    Ok(animations)
}
