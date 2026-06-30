use std::collections::BTreeMap;
use std::sync::Arc;

use amigo_math::Vec3;
use gltf::accessor::{DataType, Dimensions};
use gltf::image::{Data as GltfImageData, Format as GltfImageFormat};
use gltf::mesh::Semantic;

use crate::renderer::{WgpuSceneRenderer, cross, normalize, sub};

const NPR_EDGE_WELD_EPSILON: f32 = 1.0e-5;
const NPR_BLACK_MASS_SOLID_LUMA_THRESHOLD: f32 = 0.18;
const NPR_BLACK_MASS_TEXTURE_AVERAGE_LUMA_THRESHOLD: f32 = 0.14;
const NPR_BLACK_MASS_TEXTURE_DARK_RATIO_THRESHOLD: f32 = 0.85;
const NPR_BLACK_MASS_TEXTURE_DARK_LUMA: f32 = 0.22;

#[derive(Debug, Clone)]
pub(crate) struct CachedMeshGeometry3d {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) triangles: Vec<MeshTriangle3d>,
    pub(crate) edges: Vec<MeshEdge3d>,
    pub(crate) inferred_black_mass_material_ids: Vec<u32>,
}

impl CachedMeshGeometry3d {
    #[cfg(test)]
    pub(crate) fn from_test_vertices(vertices: Vec<Vec3>) -> Self {
        Self {
            vertices,
            triangles: Vec::new(),
            edges: Vec::new(),
            inferred_black_mass_material_ids: Vec::new(),
        }
    }

    pub(crate) fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    pub(crate) fn triangles(&self) -> &[MeshTriangle3d] {
        &self.triangles
    }

    pub(crate) fn edges(&self) -> &[MeshEdge3d] {
        &self.edges
    }

    pub(crate) fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub(crate) fn inferred_black_mass_material_ids(&self) -> &[u32] {
        &self.inferred_black_mass_material_ids
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MeshTriangle3d {
    pub(crate) indices: [usize; 3],
    pub(crate) normal: Vec3,
    pub(crate) material_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct MeshEdge3d {
    pub(crate) edge_id: u64,
    pub(crate) a: usize,
    pub(crate) b: usize,
    pub(crate) faces: Vec<usize>,
    pub(crate) material_seam: bool,
}

pub(crate) fn mesh_geometry_from_asset(
    assets: &dyn amigo_render_api::RenderAssetSource,
    mesh_asset: &amigo_assets::AssetKey,
) -> Option<CachedMeshGeometry3d> {
    let prepared = assets.prepared_asset(mesh_asset)?;
    if !matches!(prepared.kind, amigo_assets::PreparedAssetKind::Mesh3d) {
        return None;
    }
    let source_file = prepared.metadata.get("source.file")?;
    let mesh_path = prepared.resolved_path.parent()?.join(source_file);
    if prepared.format.as_deref() != Some("glb") && mesh_path.extension()?.to_str()? != "glb" {
        return None;
    }
    load_glb_geometry(&mesh_path).ok().filter(|geometry| {
        !geometry.vertices.is_empty()
            && !geometry.triangles.is_empty()
            && !geometry.edges.is_empty()
    })
}

impl WgpuSceneRenderer {
    pub(crate) fn mesh_geometry_3d(
        &mut self,
        assets: &dyn amigo_render_api::RenderAssetSource,
        mesh_asset: &amigo_assets::AssetKey,
    ) -> Arc<CachedMeshGeometry3d> {
        let cache_key = mesh_asset.as_str().to_owned();
        if let Some(cached) = self.mesh_3d_geometry_cache.get(&cache_key) {
            return Arc::clone(cached);
        }

        let geometry =
            Arc::new(mesh_geometry_from_asset(assets, mesh_asset).unwrap_or_else(cube_geometry));
        self.mesh_3d_geometry_cache
            .insert(cache_key, Arc::clone(&geometry));
        geometry
    }
}

pub(crate) fn load_glb_geometry(
    path: &std::path::Path,
) -> Result<CachedMeshGeometry3d, gltf::Error> {
    let (document, buffers, images) = import_gltf_for_geometry(path)?;
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let inferred_black_mass_material_ids =
        collect_inferred_black_mass_material_ids(&document, &images);
    let node_transforms = collect_gltf_scene_node_transforms(&document);

    if let Some(scene) = document
        .default_scene()
        .or_else(|| document.scenes().next())
    {
        for node in scene.nodes() {
            append_gltf_node_geometry(
                &mut vertices,
                &mut triangles,
                &buffers,
                node,
                GltfNodeTransform3d::IDENTITY,
                &node_transforms,
            );
        }
    }

    if vertices.is_empty() {
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                append_gltf_primitive_geometry(
                    &mut vertices,
                    &mut triangles,
                    &buffers,
                    primitive,
                    GltfNodeTransform3d::IDENTITY,
                    None,
                    &node_transforms,
                );
            }
        }
    }

    normalize_geometry(&mut vertices);
    rebuild_triangle_normals(&mut triangles, &vertices);
    let edges = build_edges(&vertices, &triangles);
    Ok(CachedMeshGeometry3d {
        vertices,
        triangles,
        edges,
        inferred_black_mass_material_ids,
    })
}

fn append_gltf_node_geometry(
    vertices: &mut Vec<Vec3>,
    triangles: &mut Vec<MeshTriangle3d>,
    buffers: &[gltf::buffer::Data],
    node: gltf::Node<'_>,
    parent_transform: GltfNodeTransform3d,
    node_transforms: &[Option<GltfNodeTransform3d>],
) {
    let node_transform = parent_transform.then(GltfNodeTransform3d::from_node(&node));
    let skin = node.skin();
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            append_gltf_primitive_geometry(
                vertices,
                triangles,
                buffers,
                primitive,
                node_transform,
                skin.clone(),
                node_transforms,
            );
        }
    }
    for child in node.children() {
        append_gltf_node_geometry(
            vertices,
            triangles,
            buffers,
            child,
            node_transform,
            node_transforms,
        );
    }
}

fn append_gltf_primitive_geometry(
    vertices: &mut Vec<Vec3>,
    triangles: &mut Vec<MeshTriangle3d>,
    buffers: &[gltf::buffer::Data],
    primitive: gltf::Primitive<'_>,
    transform: GltfNodeTransform3d,
    skin: Option<gltf::Skin<'_>>,
    node_transforms: &[Option<GltfNodeTransform3d>],
) {
    let material_id = primitive.material().index().map(|index| index as u32);
    let Some(positions) = read_gltf_positions(buffers, &primitive) else {
        return;
    };
    let base_index = vertices.len();
    if let Some(skinned_positions) = skin.and_then(|skin| {
        skin_gltf_positions(buffers, &primitive, &positions, skin, node_transforms)
    }) {
        vertices.extend(skinned_positions);
    } else {
        vertices.extend(positions.into_iter().map(|position| {
            transform.transform_point(Vec3::new(position[0], position[1], position[2]))
        }));
    }
    if let Some(indices) = read_gltf_indices(buffers, &primitive) {
        for chunk in indices.chunks_exact(3) {
            push_imported_triangle(
                triangles,
                vertices,
                [
                    base_index + chunk[0] as usize,
                    base_index + chunk[1] as usize,
                    base_index + chunk[2] as usize,
                ],
                material_id,
            );
        }
    } else {
        let count = vertices.len() - base_index;
        for chunk_start in (0..count).step_by(3) {
            if chunk_start + 2 >= count {
                break;
            }
            push_imported_triangle(
                triangles,
                vertices,
                [
                    base_index + chunk_start,
                    base_index + chunk_start + 1,
                    base_index + chunk_start + 2,
                ],
                material_id,
            );
        }
    }
}

fn read_gltf_indices(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive<'_>,
) -> Option<Vec<u32>> {
    let accessor = primitive.indices()?;
    if accessor.dimensions() != Dimensions::Scalar {
        return None;
    }
    let view = accessor.view()?;
    let buffer = buffers.get(view.buffer().index())?.0.as_slice();
    let data_type = accessor.data_type();
    let stride = view
        .stride()
        .unwrap_or(data_type.size())
        .max(data_type.size());
    let start = view.offset().checked_add(accessor.offset())?;
    let mut result = Vec::with_capacity(accessor.count());

    for index in 0..accessor.count() {
        let offset = start.checked_add(index.checked_mul(stride)?)?;
        let value = match data_type {
            DataType::U8 => *buffer.get(offset)? as u32,
            DataType::U16 => {
                u16::from_le_bytes(buffer.get(offset..offset + 2)?.try_into().ok()?) as u32
            }
            DataType::U32 => u32::from_le_bytes(buffer.get(offset..offset + 4)?.try_into().ok()?),
            _ => return None,
        };
        result.push(value);
    }

    Some(result)
}

fn read_gltf_positions(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive<'_>,
) -> Option<Vec<[f32; 3]>> {
    let accessor = primitive.get(&Semantic::Positions)?;
    if accessor.dimensions() != Dimensions::Vec3 {
        return None;
    }
    let view = accessor.view()?;
    let buffer = buffers.get(view.buffer().index())?.0.as_slice();
    let stride = view
        .stride()
        .unwrap_or(accessor.size())
        .max(accessor.size());
    let start = view.offset().checked_add(accessor.offset())?;
    let data_type = accessor.data_type();
    let normalized = accessor.normalized();
    let mut result = Vec::with_capacity(accessor.count());

    for index in 0..accessor.count() {
        let element_start = start.checked_add(index.checked_mul(stride)?)?;
        let component_size = data_type.size();
        let x = read_gltf_component_as_f32(buffer, element_start, data_type, normalized)?;
        let y = read_gltf_component_as_f32(
            buffer,
            element_start.checked_add(component_size)?,
            data_type,
            normalized,
        )?;
        let z = read_gltf_component_as_f32(
            buffer,
            element_start.checked_add(component_size * 2)?,
            data_type,
            normalized,
        )?;
        result.push([x, y, z]);
    }

    Some(result)
}

fn skin_gltf_positions(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive<'_>,
    positions: &[[f32; 3]],
    skin: gltf::Skin<'_>,
    node_transforms: &[Option<GltfNodeTransform3d>],
) -> Option<Vec<Vec3>> {
    let joints = read_gltf_joints(buffers, primitive)?;
    let weights = read_gltf_weights(buffers, primitive)?;
    if joints.len() != positions.len() || weights.len() != positions.len() {
        return None;
    }

    let inverse_bind_matrices = read_gltf_inverse_bind_matrices(buffers, &skin);
    let skin_joints = skin.joints().collect::<Vec<_>>();
    if skin_joints.is_empty() {
        return None;
    }
    let joint_matrices = skin_joints
        .iter()
        .enumerate()
        .map(|(index, joint)| {
            let joint_global = node_transforms
                .get(joint.index())
                .and_then(|transform| *transform)
                .unwrap_or_else(|| GltfNodeTransform3d::from_node(joint));
            let inverse_bind = inverse_bind_matrices
                .get(index)
                .copied()
                .unwrap_or(GltfNodeTransform3d::IDENTITY);
            joint_global.then(inverse_bind)
        })
        .collect::<Vec<_>>();

    let mut skinned = Vec::with_capacity(positions.len());
    for (index, position) in positions.iter().enumerate() {
        let point = Vec3::new(position[0], position[1], position[2]);
        let mut out = Vec3::ZERO;
        let mut total_weight = 0.0;
        for influence in 0..4 {
            let weight = weights[index][influence].max(0.0);
            if weight <= f32::EPSILON {
                continue;
            }
            let joint_index = joints[index][influence] as usize;
            let Some(joint_matrix) = joint_matrices.get(joint_index).copied() else {
                continue;
            };
            let transformed = joint_matrix.transform_point(point);
            out.x += transformed.x * weight;
            out.y += transformed.y * weight;
            out.z += transformed.z * weight;
            total_weight += weight;
        }
        if total_weight > f32::EPSILON {
            skinned.push(Vec3::new(
                out.x / total_weight,
                out.y / total_weight,
                out.z / total_weight,
            ));
        } else {
            skinned.push(point);
        }
    }

    Some(skinned)
}

fn read_gltf_joints(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive<'_>,
) -> Option<Vec<[u16; 4]>> {
    let accessor = primitive.get(&Semantic::Joints(0))?;
    if accessor.dimensions() != Dimensions::Vec4 {
        return None;
    }
    let view = accessor.view()?;
    let buffer = buffers.get(view.buffer().index())?.0.as_slice();
    let stride = view
        .stride()
        .unwrap_or(accessor.size())
        .max(accessor.size());
    let start = view.offset().checked_add(accessor.offset())?;
    let data_type = accessor.data_type();
    let component_size = data_type.size();
    let mut result = Vec::with_capacity(accessor.count());

    for index in 0..accessor.count() {
        let element_start = start.checked_add(index.checked_mul(stride)?)?;
        let mut value = [0u16; 4];
        for (component, slot) in value.iter_mut().enumerate() {
            let offset = element_start.checked_add(component.checked_mul(component_size)?)?;
            *slot = match data_type {
                DataType::U8 => *buffer.get(offset)? as u16,
                DataType::U16 => {
                    u16::from_le_bytes(buffer.get(offset..offset + 2)?.try_into().ok()?)
                }
                _ => return None,
            };
        }
        result.push(value);
    }

    Some(result)
}

fn read_gltf_weights(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive<'_>,
) -> Option<Vec<[f32; 4]>> {
    let accessor = primitive.get(&Semantic::Weights(0))?;
    if accessor.dimensions() != Dimensions::Vec4 {
        return None;
    }
    let view = accessor.view()?;
    let buffer = buffers.get(view.buffer().index())?.0.as_slice();
    let stride = view
        .stride()
        .unwrap_or(accessor.size())
        .max(accessor.size());
    let start = view.offset().checked_add(accessor.offset())?;
    let data_type = accessor.data_type();
    let component_size = data_type.size();
    let normalized = accessor.normalized() || matches!(data_type, DataType::U8 | DataType::U16);
    let mut result = Vec::with_capacity(accessor.count());

    for index in 0..accessor.count() {
        let element_start = start.checked_add(index.checked_mul(stride)?)?;
        let mut value = [0.0f32; 4];
        for (component, slot) in value.iter_mut().enumerate() {
            let offset = element_start.checked_add(component.checked_mul(component_size)?)?;
            *slot = read_gltf_component_as_f32(buffer, offset, data_type, normalized)?;
        }
        result.push(value);
    }

    Some(result)
}

fn read_gltf_inverse_bind_matrices(
    buffers: &[gltf::buffer::Data],
    skin: &gltf::Skin<'_>,
) -> Vec<GltfNodeTransform3d> {
    let Some(accessor) = skin.inverse_bind_matrices() else {
        return Vec::new();
    };
    if accessor.dimensions() != Dimensions::Mat4 || accessor.data_type() != DataType::F32 {
        return Vec::new();
    }
    let Some(view) = accessor.view() else {
        return Vec::new();
    };
    let Some(buffer) = buffers
        .get(view.buffer().index())
        .map(|buffer| buffer.0.as_slice())
    else {
        return Vec::new();
    };
    let stride = view
        .stride()
        .unwrap_or(accessor.size())
        .max(accessor.size());
    let Some(start) = view.offset().checked_add(accessor.offset()) else {
        return Vec::new();
    };
    let mut result = Vec::with_capacity(accessor.count());

    for index in 0..accessor.count() {
        let Some(element_start) = start.checked_add(index.saturating_mul(stride)) else {
            break;
        };
        let mut column_major = [0.0f32; 16];
        let mut valid = true;
        for (component, slot) in column_major.iter_mut().enumerate() {
            let offset = element_start + component * 4;
            let Some(bytes) = buffer.get(offset..offset + 4) else {
                valid = false;
                break;
            };
            let Ok(bytes) = bytes.try_into() else {
                valid = false;
                break;
            };
            *slot = f32::from_le_bytes(bytes);
        }
        if valid {
            result.push(GltfNodeTransform3d::from_column_major_matrix(column_major));
        }
    }

    result
}

fn read_gltf_component_as_f32(
    bytes: &[u8],
    offset: usize,
    data_type: DataType,
    normalized: bool,
) -> Option<f32> {
    match data_type {
        DataType::I8 => {
            let value = *bytes.get(offset)? as i8;
            Some(if normalized {
                (value as f32 / 127.0).max(-1.0)
            } else {
                value as f32
            })
        }
        DataType::U8 => {
            let value = *bytes.get(offset)?;
            Some(if normalized {
                value as f32 / 255.0
            } else {
                value as f32
            })
        }
        DataType::I16 => {
            let value = i16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?);
            Some(if normalized {
                (value as f32 / 32767.0).max(-1.0)
            } else {
                value as f32
            })
        }
        DataType::U16 => {
            let value = u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?);
            Some(if normalized {
                value as f32 / 65535.0
            } else {
                value as f32
            })
        }
        DataType::U32 => {
            let value = u32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
            Some(value as f32)
        }
        DataType::F32 => {
            let value = f32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
            Some(value)
        }
    }
}

fn import_gltf_for_geometry(
    path: &std::path::Path,
) -> Result<(gltf::Document, Vec<gltf::buffer::Data>, Vec<GltfImageData>), gltf::Error> {
    match gltf::import(path) {
        Ok((document, buffers, images)) => Ok((document, buffers, images)),
        Err(error) if gltf_error_is_quantization_extension_validation(&error) => {
            let file = std::fs::File::open(path)?;
            let reader = std::io::BufReader::new(file);
            let gltf = gltf::Gltf::from_reader_without_validation(reader)?;
            let buffers = gltf
                .blob
                .map(|blob| vec![gltf::buffer::Data(blob)])
                .unwrap_or_default();
            Ok((gltf.document, buffers, Vec::new()))
        }
        Err(error) => Err(error),
    }
}

fn collect_inferred_black_mass_material_ids(
    document: &gltf::Document,
    images: &[GltfImageData],
) -> Vec<u32> {
    let mut material_ids = Vec::new();
    for material in document.materials() {
        let Some(material_id) = material.index().map(|index| index as u32) else {
            continue;
        };
        let pbr = material.pbr_metallic_roughness();
        let base = pbr.base_color_factor();
        let texture_average = pbr
            .base_color_texture()
            .and_then(|info| images.get(info.texture().source().index()))
            .and_then(|image| average_gltf_base_color_texture(image, base));
        let inferred_black_mass = if let Some((average_color, dark_pixel_ratio)) = texture_average {
            texture_average_is_black_mass(color_luminance(average_color), dark_pixel_ratio)
        } else {
            let color = [base[0], base[1], base[2], base[3]];
            color_luminance(color) <= NPR_BLACK_MASS_SOLID_LUMA_THRESHOLD && color[3] > 0.25
        };
        if inferred_black_mass {
            material_ids.push(material_id);
        }
    }
    material_ids
}

fn average_gltf_base_color_texture(
    image: &GltfImageData,
    base_color_factor: [f32; 4],
) -> Option<([f32; 4], f32)> {
    let sample_count = (image.width as usize).checked_mul(image.height as usize)?;
    if sample_count == 0 {
        return None;
    }
    let mut rgb_sum = [0.0f32; 3];
    let mut alpha_sum = 0.0f32;
    let mut dark_pixels = 0usize;
    for index in 0..sample_count {
        let [r, g, b, a] = gltf_image_pixel_rgba(image, index)?;
        let color = [
            (r * base_color_factor[0]).clamp(0.0, 1.0),
            (g * base_color_factor[1]).clamp(0.0, 1.0),
            (b * base_color_factor[2]).clamp(0.0, 1.0),
            (a * base_color_factor[3]).clamp(0.0, 1.0),
        ];
        rgb_sum[0] += color[0];
        rgb_sum[1] += color[1];
        rgb_sum[2] += color[2];
        alpha_sum += color[3];
        if color_luminance(color) <= NPR_BLACK_MASS_TEXTURE_DARK_LUMA && color[3] > 0.25 {
            dark_pixels += 1;
        }
    }
    let inv_count = 1.0 / sample_count as f32;
    Some((
        [
            rgb_sum[0] * inv_count,
            rgb_sum[1] * inv_count,
            rgb_sum[2] * inv_count,
            alpha_sum * inv_count,
        ],
        dark_pixels as f32 * inv_count,
    ))
}

fn gltf_image_pixel_rgba(image: &GltfImageData, index: usize) -> Option<[f32; 4]> {
    match image.format {
        GltfImageFormat::R8 => {
            let r = *image.pixels.get(index)? as f32 / 255.0;
            Some([r, r, r, 1.0])
        }
        GltfImageFormat::R8G8 => {
            let offset = index.checked_mul(2)?;
            let r = *image.pixels.get(offset)? as f32 / 255.0;
            let a = *image.pixels.get(offset + 1)? as f32 / 255.0;
            Some([r, r, r, a])
        }
        GltfImageFormat::R8G8B8 => {
            let offset = index.checked_mul(3)?;
            Some([
                *image.pixels.get(offset)? as f32 / 255.0,
                *image.pixels.get(offset + 1)? as f32 / 255.0,
                *image.pixels.get(offset + 2)? as f32 / 255.0,
                1.0,
            ])
        }
        GltfImageFormat::R8G8B8A8 => {
            let offset = index.checked_mul(4)?;
            Some([
                *image.pixels.get(offset)? as f32 / 255.0,
                *image.pixels.get(offset + 1)? as f32 / 255.0,
                *image.pixels.get(offset + 2)? as f32 / 255.0,
                *image.pixels.get(offset + 3)? as f32 / 255.0,
            ])
        }
        GltfImageFormat::R16 => {
            let r = read_u16_normalized(&image.pixels, index.checked_mul(2)?)?;
            Some([r, r, r, 1.0])
        }
        GltfImageFormat::R16G16 => {
            let offset = index.checked_mul(4)?;
            let r = read_u16_normalized(&image.pixels, offset)?;
            let a = read_u16_normalized(&image.pixels, offset + 2)?;
            Some([r, r, r, a])
        }
        GltfImageFormat::R16G16B16 => {
            let offset = index.checked_mul(6)?;
            Some([
                read_u16_normalized(&image.pixels, offset)?,
                read_u16_normalized(&image.pixels, offset + 2)?,
                read_u16_normalized(&image.pixels, offset + 4)?,
                1.0,
            ])
        }
        GltfImageFormat::R16G16B16A16 => {
            let offset = index.checked_mul(8)?;
            Some([
                read_u16_normalized(&image.pixels, offset)?,
                read_u16_normalized(&image.pixels, offset + 2)?,
                read_u16_normalized(&image.pixels, offset + 4)?,
                read_u16_normalized(&image.pixels, offset + 6)?,
            ])
        }
        GltfImageFormat::R32G32B32FLOAT => {
            let offset = index.checked_mul(12)?;
            Some([
                read_f32_channel(&image.pixels, offset)?,
                read_f32_channel(&image.pixels, offset + 4)?,
                read_f32_channel(&image.pixels, offset + 8)?,
                1.0,
            ])
        }
        GltfImageFormat::R32G32B32A32FLOAT => {
            let offset = index.checked_mul(16)?;
            Some([
                read_f32_channel(&image.pixels, offset)?,
                read_f32_channel(&image.pixels, offset + 4)?,
                read_f32_channel(&image.pixels, offset + 8)?,
                read_f32_channel(&image.pixels, offset + 12)?,
            ])
        }
    }
}

fn read_u16_normalized(bytes: &[u8], offset: usize) -> Option<f32> {
    Some(u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?) as f32 / 65535.0)
}

fn read_f32_channel(bytes: &[u8], offset: usize) -> Option<f32> {
    Some(f32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?).clamp(0.0, 1.0))
}

fn color_luminance(color: [f32; 4]) -> f32 {
    color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722
}

fn texture_average_is_black_mass(luminance: f32, dark_pixel_ratio: f32) -> bool {
    luminance <= NPR_BLACK_MASS_TEXTURE_AVERAGE_LUMA_THRESHOLD
        && dark_pixel_ratio >= NPR_BLACK_MASS_TEXTURE_DARK_RATIO_THRESHOLD
}

fn gltf_error_is_quantization_extension_validation(error: &gltf::Error) -> bool {
    matches!(error, gltf::Error::Validation(_))
        && format!("{error:?}").contains("KHR_mesh_quantization")
}

fn collect_gltf_scene_node_transforms(
    document: &gltf::Document,
) -> Vec<Option<GltfNodeTransform3d>> {
    let mut transforms = vec![None; document.nodes().count()];
    if let Some(scene) = document
        .default_scene()
        .or_else(|| document.scenes().next())
    {
        for node in scene.nodes() {
            collect_gltf_node_transforms(node, GltfNodeTransform3d::IDENTITY, &mut transforms);
        }
    }
    transforms
}

fn collect_gltf_node_transforms(
    node: gltf::Node<'_>,
    parent_transform: GltfNodeTransform3d,
    transforms: &mut [Option<GltfNodeTransform3d>],
) {
    let node_transform = parent_transform.then(GltfNodeTransform3d::from_node(&node));
    if let Some(slot) = transforms.get_mut(node.index()) {
        *slot = Some(node_transform);
    }
    for child in node.children() {
        collect_gltf_node_transforms(child, node_transform, transforms);
    }
}

#[derive(Clone, Copy)]
struct GltfNodeTransform3d {
    matrix: [f32; 16],
}

impl GltfNodeTransform3d {
    const IDENTITY: Self = Self {
        matrix: [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    fn from_node(node: &gltf::Node<'_>) -> Self {
        let (translation, rotation, scale) = node.transform().decomposed();
        let [mut x, mut y, mut z, mut w] = rotation;
        let length = (x * x + y * y + z * z + w * w).sqrt();
        if length > f32::EPSILON {
            x /= length;
            y /= length;
            z /= length;
            w /= length;
        } else {
            x = 0.0;
            y = 0.0;
            z = 0.0;
            w = 1.0;
        }

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;
        let [sx, sy, sz] = scale;

        Self {
            matrix: [
                (1.0 - 2.0 * (yy + zz)) * sx,
                (2.0 * (xy - wz)) * sy,
                (2.0 * (xz + wy)) * sz,
                translation[0],
                (2.0 * (xy + wz)) * sx,
                (1.0 - 2.0 * (xx + zz)) * sy,
                (2.0 * (yz - wx)) * sz,
                translation[1],
                (2.0 * (xz - wy)) * sx,
                (2.0 * (yz + wx)) * sy,
                (1.0 - 2.0 * (xx + yy)) * sz,
                translation[2],
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    fn from_column_major_matrix(matrix: [f32; 16]) -> Self {
        Self {
            matrix: [
                matrix[0], matrix[4], matrix[8], matrix[12], //
                matrix[1], matrix[5], matrix[9], matrix[13], //
                matrix[2], matrix[6], matrix[10], matrix[14], //
                matrix[3], matrix[7], matrix[11], matrix[15],
            ],
        }
    }

    fn then(self, child: Self) -> Self {
        let mut out = [0.0; 16];
        for row in 0..4 {
            for col in 0..4 {
                out[row * 4 + col] = self.matrix[row * 4] * child.matrix[col]
                    + self.matrix[row * 4 + 1] * child.matrix[4 + col]
                    + self.matrix[row * 4 + 2] * child.matrix[8 + col]
                    + self.matrix[row * 4 + 3] * child.matrix[12 + col];
            }
        }
        Self { matrix: out }
    }

    fn transform_point(self, point: Vec3) -> Vec3 {
        Vec3::new(
            self.matrix[0] * point.x
                + self.matrix[1] * point.y
                + self.matrix[2] * point.z
                + self.matrix[3],
            self.matrix[4] * point.x
                + self.matrix[5] * point.y
                + self.matrix[6] * point.z
                + self.matrix[7],
            self.matrix[8] * point.x
                + self.matrix[9] * point.y
                + self.matrix[10] * point.z
                + self.matrix[11],
        )
    }
}

fn push_imported_triangle(
    triangles: &mut Vec<MeshTriangle3d>,
    vertices: &[Vec3],
    indices: [usize; 3],
    material_id: Option<u32>,
) {
    let normal = normalize(cross(
        sub(vertices[indices[1]], vertices[indices[0]]),
        sub(vertices[indices[2]], vertices[indices[0]]),
    ));
    if normal == Vec3::ZERO {
        return;
    }
    triangles.push(MeshTriangle3d {
        indices,
        normal,
        material_id,
    });
}

fn normalize_geometry(vertices: &mut [Vec3]) {
    if vertices.is_empty() {
        return;
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for vertex in vertices.iter().copied() {
        min.x = min.x.min(vertex.x);
        min.y = min.y.min(vertex.y);
        min.z = min.z.min(vertex.z);
        max.x = max.x.max(vertex.x);
        max.y = max.y.max(vertex.y);
        max.z = max.z.max(vertex.z);
    }
    let center = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
    let scale = if extent <= f32::EPSILON {
        1.0
    } else {
        1.8 / extent
    };
    for vertex in vertices {
        *vertex = Vec3::new(
            (vertex.x - center.x) * scale,
            (vertex.y - center.y) * scale,
            (vertex.z - center.z) * scale,
        );
    }
}

fn rebuild_triangle_normals(triangles: &mut [MeshTriangle3d], vertices: &[Vec3]) {
    for triangle in triangles {
        triangle.normal = normalize(cross(
            sub(vertices[triangle.indices[1]], vertices[triangle.indices[0]]),
            sub(vertices[triangle.indices[2]], vertices[triangle.indices[0]]),
        ));
    }
}

type WeldedVertexKey3d = (i64, i64, i64);
type WeldedEdgeKey3d = (WeldedVertexKey3d, WeldedVertexKey3d);

#[derive(Debug, Clone)]
struct MeshEdgeAccumulator3d {
    a: usize,
    b: usize,
    faces: Vec<usize>,
}

pub(crate) fn build_edges(vertices: &[Vec3], triangles: &[MeshTriangle3d]) -> Vec<MeshEdge3d> {
    let mut edge_faces = BTreeMap::<WeldedEdgeKey3d, MeshEdgeAccumulator3d>::new();
    for (face_index, triangle) in triangles.iter().enumerate() {
        let [a, b, c] = triangle.indices;
        for (left, right) in [(a, b), (b, c), (c, a)] {
            let left_key = welded_vertex_key(vertices[left]);
            let right_key = welded_vertex_key(vertices[right]);
            let key = if left_key <= right_key {
                (left_key, right_key)
            } else {
                (right_key, left_key)
            };
            edge_faces
                .entry(key)
                .or_insert_with(|| {
                    let (a, b) = if left <= right {
                        (left, right)
                    } else {
                        (right, left)
                    };
                    MeshEdgeAccumulator3d {
                        a,
                        b,
                        faces: Vec::new(),
                    }
                })
                .faces
                .push(face_index);
        }
    }
    edge_faces
        .into_iter()
        .map(|(_, edge)| {
            let faces = edge.faces;
            let material_seam = faces.len() == 2
                && triangles[faces[0]].material_id != triangles[faces[1]].material_id;
            MeshEdge3d {
                edge_id: stable_mesh_edge_id(edge.a, edge.b, &faces),
                a: edge.a,
                b: edge.b,
                faces,
                material_seam,
            }
        })
        .collect()
}

pub(crate) fn welded_vertex_key(position: Vec3) -> WeldedVertexKey3d {
    (
        (position.x / NPR_EDGE_WELD_EPSILON).round() as i64,
        (position.y / NPR_EDGE_WELD_EPSILON).round() as i64,
        (position.z / NPR_EDGE_WELD_EPSILON).round() as i64,
    )
}

pub(crate) fn cube_geometry() -> CachedMeshGeometry3d {
    let vertices = vec![
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ];
    let face_triangles = [
        [0usize, 2usize, 1usize],
        [0usize, 3usize, 2usize],
        [4usize, 5usize, 6usize],
        [4usize, 6usize, 7usize],
        [0usize, 1usize, 5usize],
        [0usize, 5usize, 4usize],
        [2usize, 3usize, 7usize],
        [2usize, 7usize, 6usize],
        [1usize, 2usize, 6usize],
        [1usize, 6usize, 5usize],
        [3usize, 0usize, 4usize],
        [3usize, 4usize, 7usize],
    ];
    let mut triangles = Vec::new();
    for indices in face_triangles {
        push_imported_triangle(&mut triangles, &vertices, indices, None);
    }
    let edges = build_edges(&vertices, &triangles);
    CachedMeshGeometry3d {
        vertices,
        triangles,
        edges,
        inferred_black_mass_material_ids: Vec::new(),
    }
}

fn stable_mesh_edge_id(a: usize, b: usize, faces: &[usize]) -> u64 {
    let first_face = faces.first().copied().unwrap_or_default() as u64;
    let second_face = faces.get(1).copied().unwrap_or_default() as u64;
    ((a as u64) << 32)
        ^ (b as u64)
        ^ first_face.wrapping_mul(0x9E37_79B9)
        ^ second_face.wrapping_mul(0x85EB_CA77)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_riders_quantized_positions_without_cube_fallback() {
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();
        let path = workspace_root.join("mods/playground-npr/source-models/khronos/Riders.glb");
        if !path.exists() {
            return;
        }

        let (document, buffers, _) =
            import_gltf_for_geometry(&path).expect("Riders GLB should import for geometry");
        let primitive = document
            .meshes()
            .next()
            .and_then(|mesh| mesh.primitives().next())
            .expect("Riders GLB should contain a mesh primitive");
        let positions = read_gltf_positions(&buffers, &primitive)
            .expect("quantized Riders POSITION accessor should decode");
        let indices =
            read_gltf_indices(&buffers, &primitive).expect("Riders index accessor should decode");

        assert!(
            positions.len() > 100_000,
            "Riders should decode real character geometry, not cube fallback"
        );
        assert!(
            indices.len() > 1_000_000,
            "Riders should decode indexed triangles without gltf utility panics"
        );
    }

    #[test]
    fn averages_dark_gltf_texture_for_black_mass_inference() {
        let image = GltfImageData {
            pixels: vec![
                0, 0, 0, 255, //
                20, 20, 20, 255, //
                255, 255, 255, 255, //
                255, 255, 255, 255,
            ],
            format: GltfImageFormat::R8G8B8A8,
            width: 2,
            height: 2,
        };

        let (average, dark_ratio) = average_gltf_base_color_texture(&image, [1.0, 1.0, 1.0, 1.0])
            .expect("test image should average");

        assert!(average[0] > 0.5);
        assert_eq!(dark_ratio, 0.5);
        assert!(
            !texture_average_is_black_mass(color_luminance(average), dark_ratio),
            "mixed atlases should not turn a whole material into black mass"
        );
    }

    #[test]
    fn fully_dark_gltf_texture_is_black_mass_candidate() {
        let image = GltfImageData {
            pixels: vec![
                0, 0, 0, 255, //
                20, 20, 20, 255, //
                10, 10, 10, 255, //
                15, 15, 15, 255,
            ],
            format: GltfImageFormat::R8G8B8A8,
            width: 2,
            height: 2,
        };

        let (average, dark_ratio) = average_gltf_base_color_texture(&image, [1.0, 1.0, 1.0, 1.0])
            .expect("test image should average");

        assert!(texture_average_is_black_mass(
            color_luminance(average),
            dark_ratio
        ));
    }
}
