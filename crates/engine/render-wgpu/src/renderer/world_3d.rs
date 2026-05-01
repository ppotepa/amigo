use crate::renderer::*;

pub(crate) fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    transform: Transform3,
    base_color: ColorRgba,
) {
    let corners = [
        Vec3::new(-0.75, -0.75, -0.75),
        Vec3::new(0.75, -0.75, -0.75),
        Vec3::new(0.75, 0.75, -0.75),
        Vec3::new(-0.75, 0.75, -0.75),
        Vec3::new(-0.75, -0.75, 0.75),
        Vec3::new(0.75, -0.75, 0.75),
        Vec3::new(0.75, 0.75, 0.75),
        Vec3::new(-0.75, 0.75, 0.75),
    ]
    .map(|point| transform_point_3d(point, transform));
    let faces = [
        (
            [[0usize, 1usize, 2usize], [0usize, 2usize, 3usize]],
            ColorRgba::new(0.88, 0.34, 0.22, 1.0),
        ),
        (
            [[4usize, 5usize, 6usize], [4usize, 6usize, 7usize]],
            ColorRgba::new(0.22, 0.72, 0.96, 1.0),
        ),
        (
            [[0usize, 1usize, 5usize], [0usize, 5usize, 4usize]],
            ColorRgba::new(0.94, 0.84, 0.28, 1.0),
        ),
        (
            [[2usize, 3usize, 7usize], [2usize, 7usize, 6usize]],
            ColorRgba::new(0.32, 0.82, 0.54, 1.0),
        ),
        (
            [[1usize, 2usize, 6usize], [1usize, 6usize, 5usize]],
            ColorRgba::new(0.82, 0.42, 0.94, 1.0),
        ),
        (
            [[3usize, 0usize, 4usize], [3usize, 4usize, 7usize]],
            ColorRgba::new(0.96, 0.58, 0.18, 1.0),
        ),
    ];

    for (face_triangles, face_tint) in faces {
        for [a, b, c] in face_triangles {
            let world = [corners[a], corners[b], corners[c]];
            let projected = [
                project_point(world[0], camera, *viewport),
                project_point(world[1], camera, *viewport),
                project_point(world[2], camera, *viewport),
            ];
            let [Some(a), Some(b), Some(c)] = projected else {
                continue;
            };
            let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
            let light_dir = normalize(Vec3::new(0.35, 0.7, 0.6));
            let brightness: f32 = (0.25 + 0.75 * dot(normal, light_dir).max(0.0)).clamp(0.0, 1.0);
            triangles.push(ProjectedTriangle {
                points: [a.position, b.position, c.position],
                color: modulate_color(blend_colors(base_color, face_tint), brightness),
                depth: (a.depth + b.depth + c.depth) / 3.0,
            });
        }
    }
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

