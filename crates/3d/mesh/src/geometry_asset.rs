//! Static glTF geometry import. Materials and texture decoding are intentionally independent.
use glam::{Mat4, Vec3};
use std::{collections::BTreeMap, path::Path};
#[derive(Debug, Clone, Default)]
pub struct MeshGeometryAsset {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub dropped_degenerate_triangles: usize,
}

pub fn load_gltf_geometry(path: &Path) -> Result<MeshGeometryAsset, String> {
    let document = gltf::Gltf::open(path).map_err(|e| e.to_string())?;
    let root = path
        .parent()
        .ok_or("model has no parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let mut buffers = Vec::new();
    for buffer in document.buffers() {
        let bytes = match buffer.source() {
            gltf::buffer::Source::Bin => document.blob.clone().ok_or("missing GLB buffer")?,
            gltf::buffer::Source::Uri(uri) => {
                let file = root
                    .join(uri)
                    .canonicalize()
                    .map_err(|e| format!("buffer {uri}: {e}"))?;
                if !file.starts_with(&root) {
                    return Err("buffer escapes model directory".into());
                }
                std::fs::read(file).map_err(|e| e.to_string())?
            }
        };
        if bytes.len() < buffer.length() {
            return Err("truncated glTF buffer".into());
        }
        buffers.push(bytes);
    }
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or("model has no scene")?;
    let mut asset = MeshGeometryAsset::default();
    for node in scene.nodes() {
        visit(node, Mat4::IDENTITY, &buffers, &mut asset)?;
    }
    if asset.positions.is_empty() || asset.indices.is_empty() {
        return Err("model contains no triangles".into());
    }
    let min = asset
        .positions
        .iter()
        .map(|p| Vec3::from_array(*p))
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = asset
        .positions
        .iter()
        .map(|p| Vec3::from_array(*p))
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let scale = 2.0 / (max - min).max_element().max(1e-6);
    let center = (min + max) * 0.5;
    for point in &mut asset.positions {
        *point = ((Vec3::from_array(*point) - center) * scale).to_array();
    }
    Ok(asset)
}
fn visit(
    node: gltf::Node<'_>,
    parent: Mat4,
    buffers: &[Vec<u8>],
    out: &mut MeshGeometryAsset,
) -> Result<(), String> {
    if node.skin().is_some() {
        return Err("skinned geometry is not supported by static importer".into());
    }
    let matrix = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err("static importer requires triangles".into());
            }
            let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(Vec::as_slice));
            let positions = reader
                .read_positions()
                .ok_or("primitive has no positions")?
                .map(|p| matrix.transform_point3(Vec3::from_array(p)))
                .collect::<Vec<_>>();
            if positions.iter().any(|p| !p.is_finite()) {
                return Err("non-finite model geometry".into());
            }
            let indices = reader
                .read_indices()
                .map(|i| i.into_u32().collect::<Vec<_>>())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());
            if indices.len() % 3 != 0 || indices.iter().any(|i| *i as usize >= positions.len()) {
                return Err("invalid triangle indices".into());
            }
            let min = positions
                .iter()
                .copied()
                .fold(Vec3::splat(f32::INFINITY), Vec3::min);
            let max = positions
                .iter()
                .copied()
                .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
            let epsilon = (max - min).length().max(1e-6) * 1e-6;
            let mut welded = BTreeMap::new();
            let mut remap = Vec::new();
            for p in positions {
                let key = (
                    (p.x / epsilon).round() as i64,
                    (p.y / epsilon).round() as i64,
                    (p.z / epsilon).round() as i64,
                );
                let index = *welded.entry(key).or_insert_with(|| {
                    let i = out.positions.len() as u32;
                    out.positions.push(p.to_array());
                    i
                });
                remap.push(index);
            }
            let mut edge_counts = BTreeMap::new();
            for tri in indices.chunks_exact(3) {
                let mut t = [
                    remap[tri[0] as usize],
                    remap[tri[1] as usize],
                    remap[tri[2] as usize],
                ];
                if matrix.determinant() < 0.0 {
                    t.swap(1, 2);
                }
                let [a, b, c] = t.map(|i| Vec3::from_array(out.positions[i as usize]));
                if (b - a).cross(c - a).length_squared() <= epsilon.powi(4) {
                    out.dropped_degenerate_triangles += 1;
                    continue;
                }
                for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                    let count = edge_counts.entry((a.min(b), a.max(b))).or_insert(0);
                    *count += 1;
                    if *count > 2 {
                        return Err("non-manifold mesh edge".into());
                    }
                }
                out.indices.extend(t);
            }
        }
    }
    for child in node.children() {
        visit(child, matrix, buffers, out)?;
    }
    Ok(())
}
