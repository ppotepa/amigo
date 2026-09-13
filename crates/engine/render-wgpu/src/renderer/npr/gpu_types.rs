use amigo_math::Vec3;

use crate::renderer::{CachedMeshGeometry3d, MeshEdge3d, MeshTriangle3d};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprVertex3d {
    pub position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprTriangle3d {
    pub indices: [u32; 4],
    pub normal: [f32; 4],
    pub material_id: u32,
    pub _pad0: [u32; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprEdge3d {
    pub a: u32,
    pub b: u32,
    pub face0: u32,
    pub face1: u32,
    pub face_count: u32,
    pub material_seam: u32,
    pub edge_id: u32,
    pub next_a: u32,
    pub next_b: u32,
    pub degree_a: u32,
    pub degree_b: u32,
    pub alt_next_a: u32,
    pub alt_next_b: u32,
    pub _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprProjectedVertex3d {
    pub ndc_depth: [f32; 4],
    pub screen: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprVisibleSegment3d {
    pub start: [f32; 4],
    pub end: [f32; 4],
    pub kind_edge: [u32; 4],
    pub metrics: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprPathLink3d {
    pub owner_edge: u32,
    pub start_next: u32,
    pub end_next: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprEndpointEntry3d {
    pub edge_index: u32,
    pub flags: u32,
    pub next_plus_one: u32,
    pub kind: u32,
    pub bin: [i32; 2],
    pub endpoint_vertex: u32,
    pub _pad0: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprPathSegment3d {
    pub start: [f32; 4],
    pub end: [f32; 4],
    pub path: [u32; 4],
    pub metrics: [f32; 4],
    pub style_metrics: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprPathState3d {
    pub owner_segment: u32,
    pub path_id: u32,
    pub kind: u32,
    pub flags: u32,
    pub segment_count: u32,
    pub _pad0: [u32; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprAggregatedPath3d {
    pub start: [f32; 4],
    pub end: [f32; 4],
    pub control: [f32; 4],
    pub path: [u32; 4],
    pub metrics: [f32; 4],
    pub style_metrics: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuNprFrameUniforms3d {
    pub model_translation: [f32; 4],
    pub model_rotation: [f32; 4],
    pub model_scale: [f32; 4],
    pub camera_translation: [f32; 4],
    pub camera_rotation: [f32; 4],
    pub viewport_half: [f32; 4],
    pub params0: [f32; 4],
    pub params1: [f32; 4],
    pub params2: [f32; 4],
    pub params3: [f32; 4],
    pub params4: [f32; 4],
    pub params5: [f32; 4],
    pub params6: [f32; 4],
    pub params7: [f32; 4],
    pub params8: [f32; 4],
    pub params9: [f32; 4],
    pub params10: [f32; 4],
    pub params11: [f32; 4],
    pub params12: [f32; 4],
    pub params13: [f32; 4],
    pub params14: [f32; 4],
    pub params15: [f32; 4],
    pub params16: [f32; 4],
    pub params17: [f32; 4],
    pub params18: [f32; 4],
    pub params19: [f32; 4],
    pub params20: [f32; 4],
    pub params21: [f32; 4],
    pub params22: [f32; 4],
    pub params23: [f32; 4],
    pub params24: [f32; 4],
    pub params25: [f32; 4],
    pub params26: [f32; 4],
    pub params27: [f32; 4],
    pub params28: [f32; 4],
    pub params29: [f32; 4],
    pub params30: [f32; 4],
    pub params31: [f32; 4],
    pub params32: [f32; 4],
    pub params33: [f32; 4],
    pub params34: [f32; 4],
    pub params35: [f32; 4],
    pub params36: [f32; 4],
    pub params37: [f32; 4],
    pub params38: [f32; 4],
    pub params39: [f32; 4],
    pub params40: [f32; 4],
    pub params41: [f32; 4],
    pub params42: [f32; 4],
    pub params43: [f32; 4],
    pub params44: [f32; 4],
    pub params45: [f32; 4],
    pub params46: [f32; 4],
    pub params47: [f32; 4],
    pub params48: [f32; 4],
    pub params49: [f32; 4],
    pub params50: [f32; 4],
    pub params51: [f32; 4],
    pub params52: [f32; 4],
    pub params53: [f32; 4],
    pub params54: [f32; 4],
    pub ink_color: [f32; 4],
    pub seed: [u32; 4],
    pub pipeline0: [u32; 4],
    pub pipeline1: [u32; 4],
    pub material_roles0: [u32; 4],
}

pub(crate) fn gpu_vertices_from_geometry(geometry: &CachedMeshGeometry3d) -> Vec<GpuNprVertex3d> {
    geometry
        .vertices()
        .iter()
        .map(|position| GpuNprVertex3d {
            position: [position.x, position.y, position.z, 1.0],
        })
        .collect()
}

pub(crate) fn gpu_triangles_from_geometry(
    geometry: &CachedMeshGeometry3d,
) -> Vec<GpuNprTriangle3d> {
    geometry
        .triangles()
        .iter()
        .map(gpu_triangle_from_mesh_triangle)
        .collect()
}

pub(crate) fn gpu_edges_from_geometry(geometry: &CachedMeshGeometry3d) -> Vec<GpuNprEdge3d> {
    let edges = geometry.edges();
    let mut vertex_edges = vec![Vec::<usize>::new(); geometry.vertices().len()];
    for (edge_index, edge) in edges.iter().enumerate() {
        vertex_edges[edge.a].push(edge_index);
        vertex_edges[edge.b].push(edge_index);
    }

    edges
        .iter()
        .enumerate()
        .map(|(edge_index, edge)| {
            gpu_edge_from_mesh_edge(geometry.vertices(), edges, &vertex_edges, edge_index, edge)
        })
        .collect()
}

fn gpu_triangle_from_mesh_triangle(triangle: &MeshTriangle3d) -> GpuNprTriangle3d {
    GpuNprTriangle3d {
        indices: [
            triangle.indices[0] as u32,
            triangle.indices[1] as u32,
            triangle.indices[2] as u32,
            0,
        ],
        normal: vec3_to_gpu4(triangle.normal),
        material_id: triangle.material_id.unwrap_or(u32::MAX),
        _pad0: [0; 7],
    }
}

pub(crate) fn gpu_edge_from_mesh_edge(
    vertices: &[Vec3],
    edges: &[MeshEdge3d],
    vertex_edges: &[Vec<usize>],
    edge_index: usize,
    edge: &MeshEdge3d,
) -> GpuNprEdge3d {
    let face0 = edge.faces.first().copied().unwrap_or(usize::MAX) as u32;
    let face1 = edge.faces.get(1).copied().unwrap_or(usize::MAX) as u32;
    let next_a = best_edge_continuations(vertices, edges, vertex_edges, edge_index, edge.a, edge.b);
    let next_b = best_edge_continuations(vertices, edges, vertex_edges, edge_index, edge.b, edge.a);
    GpuNprEdge3d {
        a: edge.a as u32,
        b: edge.b as u32,
        face0,
        face1,
        face_count: edge.faces.len().min(2) as u32,
        material_seam: edge.material_seam as u32,
        edge_id: edge.edge_id as u32,
        next_a: next_a[0].map(|index| index as u32).unwrap_or(u32::MAX),
        next_b: next_b[0].map(|index| index as u32).unwrap_or(u32::MAX),
        degree_a: vertex_edges[edge.a]
            .len()
            .saturating_sub(1)
            .min(u32::MAX as usize) as u32,
        degree_b: vertex_edges[edge.b]
            .len()
            .saturating_sub(1)
            .min(u32::MAX as usize) as u32,
        alt_next_a: next_a[1].map(|index| index as u32).unwrap_or(u32::MAX),
        alt_next_b: next_b[1].map(|index| index as u32).unwrap_or(u32::MAX),
        _pad0: [0; 3],
    }
}

fn best_edge_continuations(
    vertices: &[Vec3],
    edges: &[MeshEdge3d],
    vertex_edges: &[Vec<usize>],
    current_edge_index: usize,
    shared_vertex: usize,
    current_other_vertex: usize,
) -> [Option<usize>; 2] {
    let current_in = normalize_or_zero(Vec3::new(
        vertices[shared_vertex].x - vertices[current_other_vertex].x,
        vertices[shared_vertex].y - vertices[current_other_vertex].y,
        vertices[shared_vertex].z - vertices[current_other_vertex].z,
    ));
    if current_in == Vec3::ZERO {
        return [None, None];
    }

    let mut best: Option<(usize, f32)> = None;
    let mut second: Option<(usize, f32)> = None;
    for &candidate_index in &vertex_edges[shared_vertex] {
        if candidate_index == current_edge_index {
            continue;
        }
        let candidate = &edges[candidate_index];
        let candidate_other = if candidate.a == shared_vertex {
            candidate.b
        } else if candidate.b == shared_vertex {
            candidate.a
        } else {
            continue;
        };
        let candidate_out = normalize_or_zero(Vec3::new(
            vertices[candidate_other].x - vertices[shared_vertex].x,
            vertices[candidate_other].y - vertices[shared_vertex].y,
            vertices[candidate_other].z - vertices[shared_vertex].z,
        ));
        if candidate_out == Vec3::ZERO {
            continue;
        }
        let score = current_in.x * candidate_out.x
            + current_in.y * candidate_out.y
            + current_in.z * candidate_out.z;
        if score < 0.2 {
            continue;
        }
        match best {
            Some((_, best_score)) if score > best_score => {
                second = best;
                best = Some((candidate_index, score));
            }
            Some(_) => match second {
                Some((_, second_score)) if score <= second_score => {}
                _ => second = Some((candidate_index, score)),
            },
            None => {
                best = Some((candidate_index, score));
            }
        }
    }
    [best.map(|(index, _)| index), second.map(|(index, _)| index)]
}

fn normalize_or_zero(value: Vec3) -> Vec3 {
    let length_sq = value.x * value.x + value.y * value.y + value.z * value.z;
    if length_sq <= f32::EPSILON {
        return Vec3::ZERO;
    }
    let inv_length = length_sq.sqrt().recip();
    Vec3::new(
        value.x * inv_length,
        value.y * inv_length,
        value.z * inv_length,
    )
}

fn vec3_to_gpu4(value: Vec3) -> [f32; 4] {
    [value.x, value.y, value.z, 0.0]
}

#[allow(dead_code)]
fn normalized_depth(depth: f32, camera_settings: amigo_render_api::Camera3dRenderSettings) -> f32 {
    ((depth - camera_settings.near_clip)
        / (camera_settings.far_clip - camera_settings.near_clip).max(0.0001))
    .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        GpuNprEdge3d, GpuNprPathSegment3d, GpuNprPathState3d, GpuNprTriangle3d,
        GpuNprVisibleSegment3d,
    };

    #[test]
    fn gpu_npr_storage_structs_match_wgsl_array_stride() {
        assert_eq!(std::mem::size_of::<GpuNprTriangle3d>(), 64);
        assert_eq!(std::mem::size_of::<GpuNprEdge3d>(), 64);
        assert_eq!(std::mem::size_of::<GpuNprVisibleSegment3d>(), 64);
        assert_eq!(std::mem::size_of::<GpuNprPathSegment3d>(), 80);
        assert_eq!(std::mem::size_of::<GpuNprPathState3d>(), 48);
    }
}
