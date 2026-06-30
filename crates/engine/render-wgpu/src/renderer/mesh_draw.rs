use amigo_math::{ColorRgba, Transform3, Vec2, Vec3};

use crate::renderer::{
    CachedMeshGeometry3d, ColorVertex, NprFaceVisibilityBuffer, ProjectedPoint, ProjectedTriangle,
    Viewport, build_npr_face_visibility_buffer, cross, dot, modulate_color, multiply_color,
    normalize, project_point_with_camera, projected_triangle_is_sane, sub, transform_point_3d,
    triangle_center,
};

pub(crate) fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    light_settings: amigo_render_api::Light3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    base_color: ColorRgba,
    render_order: i32,
    shading: amigo_render_api::Material3dShadingMode,
) {
    for triangle in &geometry.triangles {
        let world = triangle
            .indices
            .map(|index| transform_point_3d(geometry.vertices[index], transform));
        let projected = world.map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        });
        let [Some(a), Some(b), Some(c)] = projected else {
            continue;
        };
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        if dot(normal, sub(camera.translation, center)) <= 0.0 {
            continue;
        }
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }
        if !projected_triangle_overlaps_viewport([a.position, b.position, c.position]) {
            continue;
        }
        let shaded = match shading {
            amigo_render_api::Material3dShadingMode::Lit => {
                let light_dir = normalize(Vec3::new(
                    -light_settings.direction.x,
                    -light_settings.direction.y,
                    -light_settings.direction.z,
                ));
                let lit = dot(normal, light_dir).max(0.0) * light_settings.intensity.max(0.0);
                let brightness: f32 = (light_settings.ambient.max(0.0) + lit).clamp(0.0, 1.25);
                multiply_color(
                    force_opaque(modulate_color(base_color, brightness)),
                    light_settings.color,
                )
            }
            amigo_render_api::Material3dShadingMode::Unlit => force_opaque(base_color),
        };
        triangles.push(ProjectedTriangle {
            points: [a.position, b.position, c.position],
            color: shaded,
            depth: (a.depth + b.depth + c.depth) / 3.0,
            render_order,
        });
    }
}

pub(crate) fn append_mesh_black_mass_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    material_ids: &[u32],
    render_order: i32,
    visibility_max_dimension_px: f32,
) {
    if material_ids.is_empty() {
        return;
    }
    let black = ColorRgba::new(0.0, 0.0, 0.0, 1.0);
    let world_vertices: Vec<Vec3> = geometry
        .vertices
        .iter()
        .copied()
        .map(|vertex| transform_point_3d(vertex, transform))
        .collect();
    let projected_vertices: Vec<Option<ProjectedPoint>> = world_vertices
        .iter()
        .copied()
        .map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        })
        .collect();
    let face_front: Vec<bool> = geometry
        .triangles
        .iter()
        .map(|triangle| {
            let world = triangle.indices.map(|index| world_vertices[index]);
            let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
            let center = triangle_center(world);
            dot(normal, sub(camera.translation, center)) > 0.0
        })
        .collect();
    let black_mass_faces: Vec<bool> = geometry
        .triangles
        .iter()
        .map(|triangle| {
            triangle
                .material_id
                .is_some_and(|material_id| material_ids.contains(&material_id))
        })
        .collect();
    let occlusion_faces = vec![true; geometry.triangle_count()];
    let visibility = build_npr_face_visibility_buffer(
        geometry,
        &projected_vertices,
        viewport,
        &occlusion_faces,
        visibility_max_dimension_px,
    );
    for (face_index, triangle) in geometry.triangles.iter().enumerate() {
        let Some(material_id) = triangle.material_id else {
            continue;
        };
        if !material_ids.contains(&material_id) {
            continue;
        }
        if !face_front.get(face_index).copied().unwrap_or(false)
            || !visibility
                .face_visible
                .get(face_index)
                .copied()
                .unwrap_or(false)
        {
            continue;
        };
        let projected = triangle
            .indices
            .map(|index| projected_vertices.get(index).and_then(|point| *point));
        let [Some(a), Some(b), Some(c)] = projected else {
            continue;
        };
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }
        if !projected_triangle_overlaps_viewport([a.position, b.position, c.position]) {
            continue;
        }
        append_visible_black_mass_triangle(
            triangles,
            &visibility,
            &black_mass_faces,
            [a, b, c],
            black,
            render_order,
        );
    }
}

pub(crate) fn append_mesh_black_tone_hatching_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    material_ids: &[u32],
    hatching: amigo_render_api::NprBlackToneHatching3d,
    seed: u64,
    visibility_max_dimension_px: f32,
) {
    let hatching = hatching.normalized();
    if !hatching.enabled || material_ids.is_empty() || hatching.max_strokes == 0 {
        return;
    }

    let world_vertices: Vec<Vec3> = geometry
        .vertices
        .iter()
        .copied()
        .map(|vertex| transform_point_3d(vertex, transform))
        .collect();
    let projected_vertices: Vec<Option<ProjectedPoint>> = world_vertices
        .iter()
        .copied()
        .map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        })
        .collect();
    let black_mass_faces: Vec<bool> = geometry
        .triangles
        .iter()
        .map(|triangle| {
            triangle
                .material_id
                .is_some_and(|material_id| material_ids.contains(&material_id))
        })
        .collect();
    let occlusion_faces = vec![true; geometry.triangle_count()];
    let visibility = build_npr_face_visibility_buffer(
        geometry,
        &projected_vertices,
        viewport,
        &occlusion_faces,
        visibility_max_dimension_px,
    );
    append_surface_tone_hatching_from_visible_faces(
        vertices,
        viewport,
        &visibility,
        geometry,
        &world_vertices,
        &projected_vertices,
        &black_mass_faces,
        hatching,
        seed,
        camera,
    );
}

fn append_visible_black_mass_triangle(
    triangles: &mut Vec<ProjectedTriangle>,
    visibility: &NprFaceVisibilityBuffer,
    black_mass_faces: &[bool],
    points: [ProjectedPoint; 3],
    color: ColorRgba,
    render_order: i32,
) {
    let subdivisions = black_mass_triangle_subdivisions(points, visibility);
    for i in 0..subdivisions {
        for j in 0..(subdivisions - i) {
            let a = black_mass_barycentric_grid_point(i, j, subdivisions);
            let b = black_mass_barycentric_grid_point(i + 1, j, subdivisions);
            let c = black_mass_barycentric_grid_point(i, j + 1, subdivisions);
            append_visible_black_mass_subtriangle(
                triangles,
                visibility,
                black_mass_faces,
                points,
                [a, b, c],
                color,
                render_order,
            );
            if i + j + 1 < subdivisions {
                let d = black_mass_barycentric_grid_point(i + 1, j + 1, subdivisions);
                append_visible_black_mass_subtriangle(
                    triangles,
                    visibility,
                    black_mass_faces,
                    points,
                    [b, d, c],
                    color,
                    render_order,
                );
            }
        }
    }
}

fn append_surface_tone_hatching_from_visible_faces(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    visibility: &NprFaceVisibilityBuffer,
    geometry: &CachedMeshGeometry3d,
    world_vertices: &[Vec3],
    projected_vertices: &[Option<ProjectedPoint>],
    black_mass_faces: &[bool],
    hatching: amigo_render_api::NprBlackToneHatching3d,
    seed: u64,
    camera: Transform3,
) {
    if visibility.width < 3 || visibility.height < 3 || hatching.density <= 0.0 {
        return;
    }
    let color = ColorRgba::new(0.0, 0.0, 0.0, hatching.alpha);
    let mut emitted = 0_u32;

    for (face_index, triangle) in geometry.triangles.iter().enumerate() {
        if emitted >= hatching.max_strokes {
            return;
        }
        if black_mass_faces.get(face_index).copied().unwrap_or(false)
            || !visibility
                .face_visible
                .get(face_index)
                .copied()
                .unwrap_or(false)
        {
            continue;
        }
        let world = triangle.indices.map(|index| world_vertices[index]);
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        if dot(normal, sub(camera.translation, center)) <= 0.0 {
            continue;
        }
        let tone = black_tone_surface_tone(normal, camera, center);
        let tone_weight = black_tone_weight(tone, hatching.tone_threshold, hatching.tone_softness);
        if tone_weight <= 0.0 {
            continue;
        }

        let projected = triangle
            .indices
            .map(|index| projected_vertices.get(index).and_then(|point| *point));
        let [Some(a), Some(b), Some(c)] = projected else {
            continue;
        };
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }
        let screen_points = [
            ndc_to_screen_px(a.position, viewport),
            ndc_to_screen_px(b.position, viewport),
            ndc_to_screen_px(c.position, viewport),
        ];
        let Some(bounds) = screen_triangle_bounds(screen_points, viewport) else {
            continue;
        };
        let step = hatching.spacing_px.max(2.0);
        let start_x = (bounds.0.x / step).floor() as i32;
        let end_x = (bounds.1.x / step).ceil() as i32;
        let start_y = (bounds.0.y / step).floor() as i32;
        let end_y = (bounds.1.y / step).ceil() as i32;

        for grid_y in start_y..=end_y {
            for grid_x in start_x..=end_x {
                if emitted >= hatching.max_strokes {
                    return;
                }
                let hash = black_tone_hash_unit(
                    face_index as u32 ^ grid_x.max(0) as u32,
                    grid_y.max(0) as u32,
                    seed,
                );
                if hash > hatching.density * tone_weight {
                    continue;
                }
                let anchor = Vec2::new((grid_x as f32 + 0.5) * step, (grid_y as f32 + 0.5) * step);
                if !point_in_screen_triangle(anchor, screen_points)
                    || !black_tone_screen_px_is_visible_face(
                        visibility, viewport, anchor, face_index,
                    )
                {
                    continue;
                }
                let jitter = (black_tone_hash_unit(
                    face_index as u32 ^ grid_x.max(0) as u32 ^ 0x8f1b,
                    grid_y.max(0) as u32 ^ 0x5d23,
                    seed,
                ) * 2.0
                    - 1.0)
                    * hatching.angle_jitter_degrees;
                let direction = Vec2::new(
                    (hatching.angle_degrees + jitter).to_radians().cos(),
                    (hatching.angle_degrees + jitter).to_radians().sin(),
                );
                let length = hatching.length_px
                    * (0.65
                        + black_tone_hash_unit(
                            face_index as u32 ^ grid_x.max(0) as u32 ^ 0x4c2d,
                            grid_y.max(0) as u32 ^ 0x31e7,
                            seed,
                        ) * 0.70)
                    * (0.55 + tone_weight * 0.45);
                let half = length * 0.5;
                let start = Vec2::new(anchor.x - direction.x * half, anchor.y - direction.y * half);
                let end = Vec2::new(anchor.x + direction.x * half, anchor.y + direction.y * half);
                let Some((start, end)) = clip_black_tone_hatch_to_visible_face(
                    visibility,
                    black_mass_faces,
                    viewport,
                    start,
                    end,
                    hatching.surface_clip_samples,
                ) else {
                    continue;
                };
                append_screen_line_quad(vertices, viewport, start, end, hatching.width_px, color);
                emitted += 1;
            }
        }
    }
}

fn append_visible_black_mass_subtriangle(
    triangles: &mut Vec<ProjectedTriangle>,
    visibility: &NprFaceVisibilityBuffer,
    black_mass_faces: &[bool],
    source: [ProjectedPoint; 3],
    barycentric: [[f32; 3]; 3],
    color: ColorRgba,
    render_order: i32,
) {
    let points = barycentric.map(|barycentric| interpolate_projected_point(source, barycentric));
    if !projected_triangle_is_sane([points[0].position, points[1].position, points[2].position]) {
        return;
    }
    if !black_mass_subtriangle_visible(visibility, black_mass_faces, points) {
        return;
    }
    triangles.push(ProjectedTriangle {
        points: [points[0].position, points[1].position, points[2].position],
        color,
        depth: (points[0].depth + points[1].depth + points[2].depth) / 3.0,
        render_order,
    });
}

fn interpolate_projected_point(
    points: [ProjectedPoint; 3],
    barycentric: [f32; 3],
) -> ProjectedPoint {
    ProjectedPoint {
        position: Vec2::new(
            points[0].position.x * barycentric[0]
                + points[1].position.x * barycentric[1]
                + points[2].position.x * barycentric[2],
            points[0].position.y * barycentric[0]
                + points[1].position.y * barycentric[1]
                + points[2].position.y * barycentric[2],
        ),
        depth: points[0].depth * barycentric[0]
            + points[1].depth * barycentric[1]
            + points[2].depth * barycentric[2],
    }
}

fn black_mass_barycentric_grid_point(i: usize, j: usize, subdivisions: usize) -> [f32; 3] {
    let inv = 1.0 / subdivisions.max(1) as f32;
    let b = i as f32 * inv;
    let c = j as f32 * inv;
    [1.0 - b - c, b, c]
}

fn black_mass_triangle_subdivisions(
    points: [ProjectedPoint; 3],
    visibility: &NprFaceVisibilityBuffer,
) -> usize {
    let a = black_mass_point_to_visibility_px(points[0].position, visibility);
    let b = black_mass_point_to_visibility_px(points[1].position, visibility);
    let c = black_mass_point_to_visibility_px(points[2].position, visibility);
    let max_edge = black_mass_distance_px(a, b)
        .max(black_mass_distance_px(b, c))
        .max(black_mass_distance_px(c, a));
    ((max_edge / 6.0).ceil() as usize).clamp(1, 24)
}

fn black_mass_point_to_visibility_px(point: Vec2, visibility: &NprFaceVisibilityBuffer) -> Vec2 {
    Vec2::new(
        (point.x * 0.5 + 0.5) * visibility.width as f32,
        (1.0 - (point.y * 0.5 + 0.5)) * visibility.height as f32,
    )
}

fn black_mass_distance_px(left: Vec2, right: Vec2) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    (dx * dx + dy * dy).sqrt()
}

fn black_mass_subtriangle_visible(
    visibility: &NprFaceVisibilityBuffer,
    black_mass_faces: &[bool],
    points: [ProjectedPoint; 3],
) -> bool {
    let centroid = interpolate_projected_point(points, [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    if !black_mass_sample_visible_surface(visibility, black_mass_faces, centroid.position) {
        return false;
    }

    let edge_samples = [
        interpolate_projected_point(points, [0.5, 0.5, 0.0]),
        interpolate_projected_point(points, [0.0, 0.5, 0.5]),
        interpolate_projected_point(points, [0.5, 0.0, 0.5]),
    ];
    let visible_samples = edge_samples
        .into_iter()
        .filter(|point| {
            black_mass_sample_visible_surface(visibility, black_mass_faces, point.position)
        })
        .count();
    visible_samples >= 2
}

fn black_mass_sample_visible_surface(
    visibility: &NprFaceVisibilityBuffer,
    black_mass_faces: &[bool],
    point: Vec2,
) -> bool {
    let x = ((point.x * 0.5 + 0.5) * visibility.width as f32).floor() as isize;
    let y = ((1.0 - (point.y * 0.5 + 0.5)) * visibility.height as f32).floor() as isize;
    if x < 0 || y < 0 || x >= visibility.width as isize || y >= visibility.height as isize {
        return false;
    }
    let face = visibility.face_id[y as usize * visibility.width + x as usize];
    face != usize::MAX && black_mass_faces.get(face).copied().unwrap_or(false)
}

fn black_tone_surface_tone(normal: Vec3, camera: Transform3, center: Vec3) -> f32 {
    let light_dir = normalize(Vec3::new(-0.35, 0.78, 0.52));
    let view_dir = normalize(sub(camera.translation, center));
    let lit = dot(normal, light_dir).max(0.0);
    let rim = 1.0 - dot(normal, view_dir).abs().clamp(0.0, 1.0);
    (1.0 - lit + rim * 0.20).clamp(0.0, 1.0)
}

fn black_tone_weight(tone: f32, threshold: f32, softness: f32) -> f32 {
    ((tone - threshold) / softness.max(0.001)).clamp(0.0, 1.0)
}

fn clip_black_tone_hatch_to_visible_face(
    visibility: &NprFaceVisibilityBuffer,
    black_mass_faces: &[bool],
    viewport: &Viewport,
    start: Vec2,
    end: Vec2,
    samples: u8,
) -> Option<(Vec2, Vec2)> {
    let mut first = None;
    let mut last = None;
    for index in 0..=samples {
        let t = index as f32 / samples.max(1) as f32;
        let point = Vec2::new(
            start.x + (end.x - start.x) * t,
            start.y + (end.y - start.y) * t,
        );
        if black_tone_screen_px_is_visible_nonblack_surface(
            visibility,
            black_mass_faces,
            viewport,
            point,
        ) {
            first.get_or_insert(point);
            last = Some(point);
        } else if first.is_some() {
            break;
        }
    }

    let first = first?;
    let last = last?;
    let dx = last.x - first.x;
    let dy = last.y - first.y;
    ((dx * dx + dy * dy).sqrt() >= 2.0).then_some((first, last))
}

fn black_tone_screen_px_is_visible_nonblack_surface(
    visibility: &NprFaceVisibilityBuffer,
    black_mass_faces: &[bool],
    viewport: &Viewport,
    point: Vec2,
) -> bool {
    black_tone_screen_px_face(visibility, viewport, point)
        .is_some_and(|face| !black_mass_faces.get(face).copied().unwrap_or(false))
}

fn black_tone_screen_px_is_visible_face(
    visibility: &NprFaceVisibilityBuffer,
    viewport: &Viewport,
    point: Vec2,
    face_index: usize,
) -> bool {
    black_tone_screen_px_face(visibility, viewport, point) == Some(face_index)
}

fn black_tone_screen_px_face(
    visibility: &NprFaceVisibilityBuffer,
    viewport: &Viewport,
    point: Vec2,
) -> Option<usize> {
    let size = viewport.size();
    let x = (point.x / size.x.max(1.0) * visibility.width as f32).floor() as isize;
    let y = (point.y / size.y.max(1.0) * visibility.height as f32).floor() as isize;
    if x < 0 || y < 0 || x >= visibility.width as isize || y >= visibility.height as isize {
        return None;
    }
    let face = visibility.face_id[y as usize * visibility.width + x as usize];
    (face != usize::MAX).then_some(face)
}

fn ndc_to_screen_px(point: Vec2, viewport: &Viewport) -> Vec2 {
    let size = viewport.size();
    Vec2::new(
        (point.x * 0.5 + 0.5) * size.x.max(1.0),
        (1.0 - (point.y * 0.5 + 0.5)) * size.y.max(1.0),
    )
}

fn screen_triangle_bounds(points: [Vec2; 3], viewport: &Viewport) -> Option<(Vec2, Vec2)> {
    let size = viewport.size();
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min)
        .max(0.0);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(size.x.max(1.0));
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min)
        .max(0.0);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(size.y.max(1.0));
    (max_x > min_x && max_y > min_y).then_some((Vec2::new(min_x, min_y), Vec2::new(max_x, max_y)))
}

fn point_in_screen_triangle(point: Vec2, triangle: [Vec2; 3]) -> bool {
    let area = edge_function(triangle[0], triangle[1], triangle[2]);
    if area.abs() <= f32::EPSILON {
        return false;
    }
    let w0 = edge_function(triangle[1], triangle[2], point);
    let w1 = edge_function(triangle[2], triangle[0], point);
    let w2 = edge_function(triangle[0], triangle[1], point);
    if area > 0.0 {
        w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
    } else {
        w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
    }
}

fn edge_function(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn append_screen_line_quad(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    start_px: Vec2,
    end_px: Vec2,
    width_px: f32,
    color: ColorRgba,
) {
    let dx = end_px.x - start_px.x;
    let dy = end_px.y - start_px.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let half_width = width_px.max(0.1) * 0.5;
    let nx = -dy / length * half_width;
    let ny = dx / length * half_width;
    let a = screen_px_to_ndc(Vec2::new(start_px.x + nx, start_px.y + ny), viewport);
    let b = screen_px_to_ndc(Vec2::new(end_px.x + nx, end_px.y + ny), viewport);
    let c = screen_px_to_ndc(Vec2::new(end_px.x - nx, end_px.y - ny), viewport);
    let d = screen_px_to_ndc(Vec2::new(start_px.x - nx, start_px.y - ny), viewport);
    vertices.push(ColorVertex::new(a, color));
    vertices.push(ColorVertex::new(b, color));
    vertices.push(ColorVertex::new(c, color));
    vertices.push(ColorVertex::new(a, color));
    vertices.push(ColorVertex::new(c, color));
    vertices.push(ColorVertex::new(d, color));
}

fn screen_px_to_ndc(point: Vec2, viewport: &Viewport) -> Vec2 {
    let size = viewport.size();
    Vec2::new(
        point.x / size.x.max(1.0) * 2.0 - 1.0,
        1.0 - point.y / size.y.max(1.0) * 2.0,
    )
}

fn black_tone_hash_unit(x: u32, y: u32, seed: u64) -> f32 {
    let mut value = seed ^ ((x as u64) << 32) ^ y as u64;
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51afd7ed558ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ceb9fe1a85ec53);
    value ^= value >> 33;
    ((value & 0xffff_ffff) as f32) / u32::MAX as f32
}

fn projected_triangle_overlaps_viewport(points: [Vec2; 3]) -> bool {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    max_x >= -1.0 && min_x <= 1.0 && max_y >= -1.0 && min_y <= 1.0
}

fn force_opaque(color: ColorRgba) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_mass_sample_requires_visible_black_mass_surface() {
        let visibility = NprFaceVisibilityBuffer {
            width: 2,
            height: 2,
            face_id: vec![usize::MAX, usize::MAX, usize::MAX, 3],
            face_visible: vec![false; 4],
        };
        let black_mass_faces = vec![false, false, false, true];
        let skin_faces = vec![false, false, false, false];

        assert!(black_mass_sample_visible_surface(
            &visibility,
            &black_mass_faces,
            Vec2::new(0.0, 0.0)
        ));
        assert!(!black_mass_sample_visible_surface(
            &visibility,
            &skin_faces,
            Vec2::new(0.0, 0.0)
        ));
    }

    #[test]
    fn black_mass_triangle_is_clipped_when_skin_surface_owns_visibility() {
        let source = [
            ProjectedPoint {
                position: Vec2::new(-0.5, -0.5),
                depth: 0.4,
            },
            ProjectedPoint {
                position: Vec2::new(0.5, -0.5),
                depth: 0.4,
            },
            ProjectedPoint {
                position: Vec2::new(0.0, 0.5),
                depth: 0.4,
            },
        ];
        let owned_visibility = NprFaceVisibilityBuffer {
            width: 4,
            height: 4,
            face_id: vec![0; 16],
            face_visible: vec![true],
        };
        let occluded_visibility = NprFaceVisibilityBuffer {
            width: 4,
            height: 4,
            face_id: vec![1; 16],
            face_visible: vec![false, true],
        };
        let black_mass_face_mask = vec![true];
        let occluded_face_mask = vec![true, false];

        let mut visible = Vec::new();
        append_visible_black_mass_triangle(
            &mut visible,
            &owned_visibility,
            &black_mass_face_mask,
            source,
            ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            0,
        );
        assert!(!visible.is_empty());

        let mut occluded = Vec::new();
        append_visible_black_mass_triangle(
            &mut occluded,
            &occluded_visibility,
            &occluded_face_mask,
            source,
            ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            0,
        );
        assert!(occluded.is_empty());
    }

    #[test]
    fn black_tone_hatching_clips_to_visible_face_owner() {
        let visibility = NprFaceVisibilityBuffer {
            width: 4,
            height: 4,
            face_id: vec![0; 16],
            face_visible: vec![true, false],
        };
        let viewport = Viewport::from_dimensions(4.0, 4.0);

        assert!(
            clip_black_tone_hatch_to_visible_face(
                &visibility,
                &[false, false],
                &viewport,
                Vec2::new(1.0, 1.0),
                Vec2::new(3.0, 1.0),
                8,
            )
            .is_some()
        );
        assert!(
            clip_black_tone_hatch_to_visible_face(
                &visibility,
                &[true, false],
                &viewport,
                Vec2::new(1.0, 1.0),
                Vec2::new(3.0, 1.0),
                8,
            )
            .is_none()
        );
    }
}
