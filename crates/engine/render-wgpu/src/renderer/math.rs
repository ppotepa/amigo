use crate::renderer::*;

pub(crate) fn project_point(
    point: Vec3,
    camera: Transform3,
    viewport: Viewport,
) -> Option<ProjectedPoint> {
    project_point_with_camera(point, camera, viewport, 60.0, 0.05, 1000.0)
}

pub(crate) fn project_point_with_camera(
    point: Vec3,
    camera: Transform3,
    viewport: Viewport,
    fov_y_degrees: f32,
    near_clip: f32,
    far_clip: f32,
) -> Option<ProjectedPoint> {
    let relative = sub(point, camera.translation);
    let camera_space = rotate_inverse(relative, camera.rotation_euler);
    let depth = -camera_space.z;

    if depth <= near_clip.max(0.001) || depth >= far_clip.max(near_clip + 0.001) {
        return None;
    }

    let focal = 1.0 / (fov_y_degrees.clamp(20.0, 120.0).to_radians() * 0.5).tan();
    let x = (camera_space.x * focal / viewport.aspect) / depth;
    let y = (camera_space.y * focal) / depth;

    Some(ProjectedPoint {
        position: Vec2::new(x, y),
        depth,
    })
}

pub(crate) fn ndc_from_screen(point: Vec2, viewport: &Viewport) -> Vec2 {
    Vec2::new(
        point.x / viewport.half_width,
        point.y / viewport.half_height,
    )
}

pub(crate) fn projected_triangle_is_sane(points: [Vec2; 3]) -> bool {
    points.iter().all(|point| {
        point.x.is_finite() && point.y.is_finite() && point.x.abs() < 8.0 && point.y.abs() < 8.0
    })
}

pub(crate) fn screen_segment_length_px(start: Vec2, end: Vec2, viewport: &Viewport) -> f32 {
    let dx = (end.x - start.x) * viewport.half_width;
    let dy = (end.y - start.y) * viewport.half_height;
    (dx * dx + dy * dy).sqrt()
}

pub(crate) fn ndc_from_world_2d(point: Vec2, camera: Transform2, viewport: &Viewport) -> Vec2 {
    let relative = Vec2::new(
        point.x - camera.translation.x,
        point.y - camera.translation.y,
    );
    let sin = camera.rotation_radians.sin();
    let cos = camera.rotation_radians.cos();
    let camera_space = Vec2::new(
        relative.x * cos + relative.y * sin,
        -relative.x * sin + relative.y * cos,
    );
    let zoomed = Vec2::new(
        camera_space.x * camera.scale.x.max(0.001),
        camera_space.y * camera.scale.y.max(0.001),
    );
    ndc_from_screen(zoomed, viewport)
}

pub(crate) fn ndc_from_world_2d_snapped(
    point: Vec2,
    camera: Transform2,
    viewport: &Viewport,
) -> Vec2 {
    let relative = Vec2::new(
        (point.x - camera.translation.x).round(),
        (point.y - camera.translation.y).round(),
    );
    let sin = camera.rotation_radians.sin();
    let cos = camera.rotation_radians.cos();
    let camera_space = Vec2::new(
        relative.x * cos + relative.y * sin,
        -relative.x * sin + relative.y * cos,
    );
    let zoomed = Vec2::new(
        camera_space.x * camera.scale.x.max(0.001),
        camera_space.y * camera.scale.y.max(0.001),
    );
    ndc_from_screen(zoomed, viewport)
}

pub(crate) fn ndc_from_ui_screen(point: Vec2, viewport: &Viewport) -> Vec2 {
    Vec2::new(
        point.x / viewport.half_width - 1.0,
        1.0 - point.y / viewport.half_height,
    )
}

pub(crate) fn push_quad(
    vertices: &mut Vec<ColorVertex>,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    color: ColorRgba,
) {
    push_triangle(vertices, [a, b, c], color);
    push_triangle(vertices, [a, c, d], color);
}

pub(crate) fn push_textured_quad(
    vertices: &mut Vec<TextureVertex>,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    uv: TextureUvRect,
    color: ColorRgba,
) {
    let bottom_left = Vec2::new(uv.u0, uv.v1);
    let bottom_right = Vec2::new(uv.u1, uv.v1);
    let top_right = Vec2::new(uv.u1, uv.v0);
    let top_left = Vec2::new(uv.u0, uv.v0);
    vertices.push(TextureVertex::new(a, bottom_left, color));
    vertices.push(TextureVertex::new(b, bottom_right, color));
    vertices.push(TextureVertex::new(c, top_right, color));
    vertices.push(TextureVertex::new(a, bottom_left, color));
    vertices.push(TextureVertex::new(c, top_right, color));
    vertices.push(TextureVertex::new(d, top_left, color));
}

pub(crate) fn push_triangle(vertices: &mut Vec<ColorVertex>, points: [Vec2; 3], color: ColorRgba) {
    vertices.push(ColorVertex::new(points[0], color));
    vertices.push(ColorVertex::new(points[1], color));
    vertices.push(ColorVertex::new(points[2], color));
}

pub(crate) fn transform_point_2d(point: Vec2, transform: Transform2) -> Vec2 {
    let scaled = Vec2::new(point.x * transform.scale.x, point.y * transform.scale.y);
    let sin = transform.rotation_radians.sin();
    let cos = transform.rotation_radians.cos();
    let rotated = Vec2::new(
        scaled.x * cos - scaled.y * sin,
        scaled.x * sin + scaled.y * cos,
    );
    Vec2::new(
        rotated.x + transform.translation.x,
        rotated.y + transform.translation.y,
    )
}

pub(crate) fn transform_point_3d(point: Vec3, transform: Transform3) -> Vec3 {
    let scaled = Vec3::new(
        point.x * transform.scale.x,
        point.y * transform.scale.y,
        point.z * transform.scale.z,
    );
    let rotated_x = rotate_x(scaled, transform.rotation_euler.x);
    let rotated_y = rotate_y(rotated_x, transform.rotation_euler.y);
    let rotated_z = rotate_z(rotated_y, transform.rotation_euler.z);
    Vec3::new(
        rotated_z.x + transform.translation.x,
        rotated_z.y + transform.translation.y,
        rotated_z.z + transform.translation.z,
    )
}

fn rotate_inverse(point: Vec3, rotation: Vec3) -> Vec3 {
    let around_z = rotate_z(point, -rotation.z);
    let around_y = rotate_y(around_z, -rotation.y);
    rotate_x(around_y, -rotation.x)
}

pub(crate) fn rotate_x(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x,
        point.y * cos - point.z * sin,
        point.y * sin + point.z * cos,
    )
}

pub(crate) fn rotate_y(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x * cos + point.z * sin,
        point.y,
        -point.x * sin + point.z * cos,
    )
}

pub(crate) fn rotate_z(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x * cos - point.y * sin,
        point.x * sin + point.y * cos,
        point.z,
    )
}

pub(crate) fn sprite_color(asset_key: &str) -> ColorRgba {
    if asset_key.contains("square") {
        ColorRgba::new(0.18, 0.74, 1.0, 1.0)
    } else {
        ColorRgba::new(0.46, 0.78, 0.54, 1.0)
    }
}

pub(crate) fn mesh_color(asset_key: &str) -> ColorRgba {
    if asset_key.contains("cube") {
        ColorRgba::new(0.92, 0.46, 0.18, 1.0)
    } else {
        ColorRgba::new(0.68, 0.7, 0.92, 1.0)
    }
}

pub(crate) fn modulate_color(color: ColorRgba, factor: f32) -> ColorRgba {
    ColorRgba::new(
        color.r * factor,
        color.g * factor,
        color.b * factor,
        color.a,
    )
}

pub(crate) fn blend_colors(base: ColorRgba, accent: ColorRgba) -> ColorRgba {
    ColorRgba::new(
        (base.r * 0.45 + accent.r * 0.55).clamp(0.0, 1.0),
        (base.g * 0.45 + accent.g * 0.55).clamp(0.0, 1.0),
        (base.b * 0.45 + accent.b * 0.55).clamp(0.0, 1.0),
        base.a,
    )
}

pub(crate) fn multiply_color(left: ColorRgba, right: ColorRgba) -> ColorRgba {
    ColorRgba::new(
        (left.r * right.r).clamp(0.0, 1.0),
        (left.g * right.g).clamp(0.0, 1.0),
        (left.b * right.b).clamp(0.0, 1.0),
        left.a * right.a,
    )
}

pub(crate) fn spritesheet_frame_color(frame: u32) -> ColorRgba {
    match frame % 8 {
        0 => ColorRgba::new(0.95, 0.36, 0.28, 1.0),
        1 => ColorRgba::new(0.95, 0.6, 0.22, 1.0),
        2 => ColorRgba::new(0.93, 0.82, 0.24, 1.0),
        3 => ColorRgba::new(0.36, 0.82, 0.42, 1.0),
        4 => ColorRgba::new(0.22, 0.72, 0.92, 1.0),
        5 => ColorRgba::new(0.34, 0.48, 0.95, 1.0),
        6 => ColorRgba::new(0.66, 0.34, 0.95, 1.0),
        _ => ColorRgba::new(0.92, 0.3, 0.72, 1.0),
    }
}

pub(crate) fn sub(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x - right.x, left.y - right.y, left.z - right.z)
}

pub(crate) fn cross(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(
        left.y * right.z - left.z * right.y,
        left.z * right.x - left.x * right.z,
        left.x * right.y - left.y * right.x,
    )
}

pub(crate) fn dot(left: Vec3, right: Vec3) -> f32 {
    left.x * right.x + left.y * right.y + left.z * right.z
}

pub(crate) fn normalize(value: Vec3) -> Vec3 {
    let length = dot(value, value).sqrt();
    if length <= f32::EPSILON {
        Vec3::ZERO
    } else {
        Vec3::new(value.x / length, value.y / length, value.z / length)
    }
}

pub(crate) fn triangle_center(points: [Vec3; 3]) -> Vec3 {
    Vec3::new(
        (points[0].x + points[1].x + points[2].x) / 3.0,
        (points[0].y + points[1].y + points[2].y) / 3.0,
        (points[0].z + points[1].z + points[2].z) / 3.0,
    )
}
