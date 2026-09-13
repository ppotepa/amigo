use amigo_math::Vec3;

use crate::renderer::{
    CachedMeshGeometry3d, NprFaceVisibilityBuffer, ProjectedPoint, Viewport,
    projected_triangle_is_sane,
};

pub(crate) fn build_npr_face_visibility_buffer(
    geometry: &CachedMeshGeometry3d,
    projected_vertices: &[Option<ProjectedPoint>],
    viewport: &Viewport,
    face_front: &[bool],
    max_visibility_dimension_px: f32,
) -> NprFaceVisibilityBuffer {
    let size = viewport.size();
    let max_dimension = size.x.max(size.y).max(1.0);
    let target_dimension = max_visibility_dimension_px.clamp(128.0, 4096.0);
    let scale = (target_dimension / max_dimension).min(1.0);
    let width = (size.x * scale).round().max(8.0) as usize;
    let height = (size.y * scale).round().max(8.0) as usize;
    let mut depth = vec![f32::INFINITY; width * height];
    let mut face_id = vec![usize::MAX; width * height];
    let mut face_visible = vec![false; geometry.triangle_count()];

    for (face_index, triangle) in geometry.triangles().iter().enumerate() {
        if !face_front.get(face_index).copied().unwrap_or(false) {
            continue;
        }
        let Some(a) = projected_vertices
            .get(triangle.indices[0])
            .and_then(|point| *point)
        else {
            continue;
        };
        let Some(b) = projected_vertices
            .get(triangle.indices[1])
            .and_then(|point| *point)
        else {
            continue;
        };
        let Some(c) = projected_vertices
            .get(triangle.indices[2])
            .and_then(|point| *point)
        else {
            continue;
        };
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }

        let a = npr_projected_point_to_buffer(a, width, height);
        let b = npr_projected_point_to_buffer(b, width, height);
        let c = npr_projected_point_to_buffer(c, width, height);
        let area = npr_edge_function(a.x, a.y, b.x, b.y, c.x, c.y);
        if area.abs() <= f32::EPSILON {
            continue;
        }

        let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
        let max_x = a.x.max(b.x).max(c.x).ceil().min((width - 1) as f32) as usize;
        let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as usize;
        let max_y = a.y.max(b.y).max(c.y).ceil().min((height - 1) as f32) as usize;
        if min_x > max_x || min_y > max_y {
            continue;
        }

        let w0_step_x = c.y - b.y;
        let w1_step_x = a.y - c.y;
        let w2_step_x = b.y - a.y;
        let w0_step_y = -(c.x - b.x);
        let w1_step_y = -(a.x - c.x);
        let w2_step_y = -(b.x - a.x);
        let start_px = min_x as f32 + 0.5;
        let start_py = min_y as f32 + 0.5;
        let row_start_w0 = npr_edge_function(b.x, b.y, c.x, c.y, start_px, start_py);
        let row_start_w1 = npr_edge_function(c.x, c.y, a.x, a.y, start_px, start_py);
        let row_start_w2 = npr_edge_function(a.x, a.y, b.x, b.y, start_px, start_py);
        let inv_area = 1.0 / area;

        for y in min_y..=max_y {
            let row_offset = (y - min_y) as f32;
            let mut w0 = row_start_w0 + row_offset * w0_step_y;
            let mut w1 = row_start_w1 + row_offset * w1_step_y;
            let mut w2 = row_start_w2 + row_offset * w2_step_y;
            for x in min_x..=max_x {
                let inside = if area >= 0.0 {
                    w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5
                } else {
                    w0 <= 1e-5 && w1 <= 1e-5 && w2 <= 1e-5
                };
                if !inside {
                    w0 += w0_step_x;
                    w1 += w1_step_x;
                    w2 += w2_step_x;
                    continue;
                }

                let l0 = w0 * inv_area;
                let l1 = w1 * inv_area;
                let l2 = w2 * inv_area;
                let sample_depth = l0 * a.z + l1 * b.z + l2 * c.z;
                let index = y * width + x;
                if sample_depth < depth[index] {
                    depth[index] = sample_depth;
                    face_id[index] = face_index;
                }
                w0 += w0_step_x;
                w1 += w1_step_x;
                w2 += w2_step_x;
            }
        }
    }

    for face in face_id.iter().copied().filter(|face| *face != usize::MAX) {
        if let Some(visible) = face_visible.get_mut(face) {
            *visible = true;
        }
    }

    NprFaceVisibilityBuffer {
        width,
        height,
        face_id,
        face_visible,
    }
}

fn npr_projected_point_to_buffer(point: ProjectedPoint, width: usize, height: usize) -> Vec3 {
    Vec3::new(
        (point.position.x * 0.5 + 0.5) * width as f32,
        (1.0 - (point.position.y * 0.5 + 0.5)) * height as f32,
        point.depth,
    )
}

fn npr_edge_function(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}
