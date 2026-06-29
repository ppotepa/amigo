use amigo_math::{ColorRgba, Vec2};

use crate::renderer::{
    ColorVertex, NprDebugOverlay3d, NprLineKind, NprStrokePassKind, NprStrokePath, Viewport,
    build_npr_dropout_mask, build_npr_stable_brush_path, build_npr_stroke_gesture,
    build_npr_stroke_pass_plan, deterministic_noise, npr_stroke_strip_sample, push_quad,
    screen_segment_length_px,
};

pub(crate) fn append_npr_debug_path_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    overlay: NprDebugOverlay3d,
) {
    match overlay {
        NprDebugOverlay3d::LineKinds => {
            append_npr_debug_polyline(
                vertices,
                viewport,
                &path.points,
                npr_line_kind_debug_color(path.kind),
                2.0,
            );
        }
        NprDebugOverlay3d::RawPaths => {
            append_npr_debug_polyline(
                vertices,
                viewport,
                &path.points,
                npr_path_id_debug_color(path.path_id),
                1.5,
            );
        }
        NprDebugOverlay3d::Dropout => {
            append_npr_dropout_debug_vertices(vertices, viewport, path, settings);
        }
        NprDebugOverlay3d::WidthAlpha => {
            append_npr_width_alpha_debug_vertices(vertices, viewport, path, settings);
        }
    }
}

fn append_npr_dropout_debug_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    if path.points.len() < 2 {
        return;
    }

    let gesture = build_npr_stroke_gesture(path, settings);
    let brush_path = build_npr_stable_brush_path(path, viewport);
    let passes = build_npr_stroke_pass_plan(path, settings, gesture);
    let dropout = build_npr_dropout_mask(gesture, settings, &passes);
    let Some(primary) = passes
        .iter()
        .copied()
        .find(|pass| pass.kind == NprStrokePassKind::Primary)
    else {
        return;
    };

    for point_index in 1..brush_path.samples.len() {
        let segment_t0 = brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
        let segment_t1 = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
        let segment_length = screen_segment_length_px(
            brush_path.samples[point_index - 1].point,
            brush_path.samples[point_index].point,
            viewport,
        );
        if dropout.keeps_segment(primary, segment_t0, segment_t1, segment_length) {
            continue;
        }
        append_npr_debug_segment(
            vertices,
            viewport,
            brush_path.samples[point_index - 1].point,
            brush_path.samples[point_index].point,
            ColorRgba::new(1.0, 0.12, 0.05, 0.95),
            4.0,
        );
    }
}

fn append_npr_width_alpha_debug_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    if path.points.len() < 2 {
        return;
    }

    let gesture = build_npr_stroke_gesture(path, settings);
    let brush_path = build_npr_stable_brush_path(path, viewport);
    let Some(primary) = build_npr_stroke_pass_plan(path, settings, gesture)
        .into_iter()
        .find(|pass| pass.kind == NprStrokePassKind::Primary)
    else {
        return;
    };

    for point_index in 1..brush_path.samples.len() {
        let t0 = brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
        let t1 = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
        let sample0 = npr_stroke_strip_sample(
            &brush_path,
            point_index - 1,
            settings,
            gesture,
            primary,
            t0,
            viewport,
        );
        let sample1 = npr_stroke_strip_sample(
            &brush_path,
            point_index,
            settings,
            gesture,
            primary,
            t1,
            viewport,
        );
        let width01 = ((sample0.width_px + sample1.width_px) * 0.5 / 8.0).clamp(0.0, 1.0);
        let alpha01 = ((sample0.color.a + sample1.color.a) * 0.5).clamp(0.0, 1.0);
        append_npr_debug_segment(
            vertices,
            viewport,
            sample0.point,
            sample1.point,
            ColorRgba::new(width01, alpha01, 1.0 - width01, 0.9),
            3.0,
        );
    }
}

fn append_npr_debug_polyline(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    points: &[Vec2],
    color: ColorRgba,
    width_px: f32,
) {
    for window in points.windows(2) {
        append_npr_debug_segment(vertices, viewport, window[0], window[1], color, width_px);
    }
}

fn append_npr_debug_segment(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    a: Vec2,
    b: Vec2,
    color: ColorRgba,
    width_px: f32,
) {
    let delta_px = Vec2::new(
        (b.x - a.x) * viewport.half_width,
        (b.y - a.y) * viewport.half_height,
    );
    let length = (delta_px.x * delta_px.x + delta_px.y * delta_px.y).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let normal_px = Vec2::new(-delta_px.y / length, delta_px.x / length);
    let half = width_px.max(0.5) * 0.5;
    let offset = Vec2::new(
        normal_px.x * half / viewport.half_width,
        normal_px.y * half / viewport.half_height,
    );
    push_quad(
        vertices,
        Vec2::new(a.x + offset.x, a.y + offset.y),
        Vec2::new(b.x + offset.x, b.y + offset.y),
        Vec2::new(b.x - offset.x, b.y - offset.y),
        Vec2::new(a.x - offset.x, a.y - offset.y),
        color,
    );
}

fn npr_line_kind_debug_color(kind: NprLineKind) -> ColorRgba {
    match kind {
        NprLineKind::Boundary => ColorRgba::new(0.15, 0.45, 1.0, 0.95),
        NprLineKind::Silhouette => ColorRgba::new(1.0, 0.88, 0.1, 0.95),
        NprLineKind::Crease => ColorRgba::new(1.0, 0.22, 0.12, 0.9),
        NprLineKind::Seam => ColorRgba::new(0.2, 1.0, 0.45, 0.9),
        NprLineKind::Feature => ColorRgba::new(0.85, 0.35, 1.0, 0.9),
        NprLineKind::Contact => ColorRgba::new(0.05, 0.05, 0.05, 0.95),
    }
}

fn npr_path_id_debug_color(path_id: u64) -> ColorRgba {
    let r = deterministic_noise(path_id, 11, 0, 0);
    let g = deterministic_noise(path_id, 23, 0, 0);
    let b = deterministic_noise(path_id, 37, 0, 0);
    ColorRgba::new(0.25 + r * 0.75, 0.25 + g * 0.75, 0.25 + b * 0.75, 0.88)
}
