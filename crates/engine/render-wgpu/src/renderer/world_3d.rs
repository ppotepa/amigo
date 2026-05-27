use crate::renderer::*;

pub(crate) fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    light_settings: amigo_render_api::Light3dRenderSettings,
    transform: Transform3,
    base_color: ColorRgba,
    render_order: i32,
) {
    let corners = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ]
    .map(|point| transform_point_3d(point, transform));
    let faces = [
        [[0usize, 2usize, 1usize], [0usize, 3usize, 2usize]],
        [[4usize, 5usize, 6usize], [4usize, 6usize, 7usize]],
        [[0usize, 1usize, 5usize], [0usize, 5usize, 4usize]],
        [[2usize, 3usize, 7usize], [2usize, 7usize, 6usize]],
        [[1usize, 2usize, 6usize], [1usize, 6usize, 5usize]],
        [[3usize, 0usize, 4usize], [3usize, 4usize, 7usize]],
    ];

    for face_triangles in faces {
        for [a, b, c] in face_triangles {
            let world = [corners[a], corners[b], corners[c]];
            let projected = [
                project_point_with_camera(
                    world[0],
                    camera,
                    *viewport,
                    camera_settings.fov_y_degrees,
                    camera_settings.near_clip,
                    camera_settings.far_clip,
                ),
                project_point_with_camera(
                    world[1],
                    camera,
                    *viewport,
                    camera_settings.fov_y_degrees,
                    camera_settings.near_clip,
                    camera_settings.far_clip,
                ),
                project_point_with_camera(
                    world[2],
                    camera,
                    *viewport,
                    camera_settings.fov_y_degrees,
                    camera_settings.near_clip,
                    camera_settings.far_clip,
                ),
            ];
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
            let light_dir = normalize(Vec3::new(
                -light_settings.direction.x,
                -light_settings.direction.y,
                -light_settings.direction.z,
            ));
            let lit = dot(normal, light_dir).max(0.0) * light_settings.intensity.max(0.0);
            let brightness: f32 = (light_settings.ambient.max(0.0) + lit).clamp(0.0, 1.25);
            let shaded = force_opaque(modulate_color(base_color, brightness));
            triangles.push(ProjectedTriangle {
                points: [a.position, b.position, c.position],
                color: multiply_color(shaded, light_settings.color),
                depth: (a.depth + b.depth + c.depth) / 3.0,
                render_order,
            });
        }
    }
}

fn triangle_center(points: [Vec3; 3]) -> Vec3 {
    Vec3::new(
        (points[0].x + points[1].x + points[2].x) / 3.0,
        (points[0].y + points[1].y + points[2].y) / 3.0,
        (points[0].z + points[1].z + points[2].z) / 3.0,
    )
}

fn projected_triangle_is_sane(points: [Vec2; 3]) -> bool {
    points.iter().all(|point| {
        point.x.is_finite() && point.y.is_finite() && point.x.abs() < 8.0 && point.y.abs() < 8.0
    })
}

fn force_opaque(color: ColorRgba) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, 1.0)
}

pub(crate) fn append_text_3d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    content: &str,
    transform: Transform3,
    size: f32,
    color: ColorRgba,
) {
    let pixel_size = (size * 0.18).max(0.05);
    let advance = 6.0 * pixel_size;
    let text_width = content.chars().count() as f32 * advance;
    let start_x = -text_width * 0.5;
    let start_y = -3.5 * pixel_size;

    for (index, ch) in content.chars().enumerate() {
        let rows = glyph_rows(ch);
        let glyph_origin_x = start_x + index as f32 * advance;
        for (row_index, row_bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if row_bits & (1 << (4 - column)) == 0 {
                    continue;
                }

                let min = Vec3::new(
                    glyph_origin_x + column as f32 * pixel_size,
                    start_y + (6 - row_index) as f32 * pixel_size,
                    0.0,
                );
                let max = Vec3::new(min.x + pixel_size, min.y + pixel_size, 0.0);
                let quad = [
                    transform_point_3d(min, transform),
                    transform_point_3d(Vec3::new(max.x, min.y, 0.0), transform),
                    transform_point_3d(max, transform),
                    transform_point_3d(Vec3::new(min.x, max.y, 0.0), transform),
                ];
                let [Some(a), Some(b), Some(c), Some(d)] = quad.map(|point| {
                    project_point(point, camera, *viewport).map(|projected| projected.position)
                }) else {
                    continue;
                };
                push_quad(vertices, a, b, c, d, color);
            }
        }
    }
}
