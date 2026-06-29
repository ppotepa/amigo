use amigo_math::{ColorRgba, Vec2};

use crate::renderer::{
    ColorVertex, NprStableBrushPath, NprStrokeGesture, NprStrokePassPlan, NprStrokeRail,
    NprStrokeSegmentVertex, NprStrokeStripSample, Viewport, npr_alpha_pressure_multiplier,
    npr_depth_alpha_multiplier, npr_distance_width_multiplier, npr_pressure_multiplier,
};

use super::{coherent_signed_noise_1d, resolve_npr_brush_profile};

pub(crate) fn npr_endpoint_lock(
    t: f32,
    path_length_px: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let start_t =
        (settings.endpoint_lock_start_px.max(0.0) / path_length_px.max(1.0)).clamp(0.0, 0.45);
    let end_t = (settings.endpoint_lock_end_px.max(0.0) / path_length_px.max(1.0)).clamp(0.0, 0.45);
    if start_t > 0.0 && t <= start_t {
        (t / start_t).clamp(0.0, 1.0)
    } else if end_t > 0.0 && t >= 1.0 - end_t {
        ((1.0 - t) / end_t).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

pub(crate) fn npr_pass_offset_px(
    path_seed: u64,
    arc_t: f32,
    settings: &amigo_render_api::NprLineSettings3d,
    pass: u8,
) -> f32 {
    if settings.pass_offset_px <= 0.0 {
        return 0.0;
    }
    coherent_signed_noise_1d(
        settings.seed,
        path_seed,
        pass as u64,
        arc_t * 12.0 + pass as f32,
        631,
    ) * settings.pass_offset_px
}

pub(crate) fn npr_taper_multiplier(t: f32, taper: f32) -> f32 {
    let endpoint_weight = (t.min(1.0 - t) * 2.0).clamp(0.0, 1.0);
    1.0 - taper.clamp(0.0, 1.0) * (1.0 - endpoint_weight.max(0.35))
}

pub(crate) fn humanize_npr_brush_sample(
    brush_path: &NprStableBrushPath,
    index: usize,
    settings: &amigo_render_api::NprLineSettings3d,
    pass: u8,
    wobble_px: f32,
    viewport: &Viewport,
) -> Vec2 {
    let brush = resolve_npr_brush_profile(settings);
    let micro_wobble_px = settings.micro_wobble_px
        * settings.humanization
        * brush.path_wobble_multiplier
        * brush.micro_wobble_multiplier;
    if wobble_px <= 0.0 && micro_wobble_px <= 0.0 {
        return brush_path.samples[index].point;
    }

    let prev = brush_path.samples[index.saturating_sub(1)].point;
    let next = brush_path.samples[(index + 1).min(brush_path.samples.len() - 1)].point;
    let tx = (next.x - prev.x) * viewport.half_width;
    let ty = (next.y - prev.y) * viewport.half_height;
    let length = (tx * tx + ty * ty).sqrt();
    if length <= f32::EPSILON {
        return brush_path.samples[index].point;
    }

    let normal = Vec2::new(-ty / length, tx / length);
    let point = brush_path.samples[index].point;
    let arc_t = brush_path.samples[index].arc_length_px / brush_path.length_px;
    let endpoint_lock = npr_endpoint_lock(arc_t, brush_path.length_px, settings);
    let primary = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.stroke_wobble_frequency.max(0.01) * 100.0,
        919,
    );
    let drift = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.stroke_wobble_frequency.max(0.01) * 37.0 + 3.7,
        977,
    );
    let micro = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.micro_wobble_frequency.max(0.01) * 100.0 + 13.0,
        991,
    );
    let tangent_scale =
        settings.local_angular_drift_degrees.to_radians().sin() * settings.humanization;
    let px = point.x * viewport.half_width
        + normal.x * primary * wobble_px * endpoint_lock
        + normal.x * micro * micro_wobble_px * endpoint_lock
        + (tx / length) * drift * wobble_px * tangent_scale * endpoint_lock;
    let py = point.y * viewport.half_height
        + normal.y * primary * wobble_px * endpoint_lock
        + normal.y * micro * micro_wobble_px * endpoint_lock
        + (ty / length) * drift * wobble_px * tangent_scale * endpoint_lock;
    Vec2::new(px / viewport.half_width, py / viewport.half_height)
}

pub(crate) fn npr_stroke_strip_sample(
    brush_path: &NprStableBrushPath,
    point_index: usize,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
    pass: NprStrokePassPlan,
    distance_t: f32,
    viewport: &Viewport,
) -> NprStrokeStripSample {
    let brush = resolve_npr_brush_profile(settings);
    let point = humanize_npr_brush_sample(
        brush_path,
        point_index,
        settings,
        pass.pass_index,
        pass.wobble_px,
        viewport,
    );
    let width_noise = coherent_signed_noise_1d(
        settings.seed,
        gesture.path_seed,
        pass.pass_index as u64,
        distance_t * settings.stroke_wobble_frequency.max(0.01) * 100.0 + 7.0,
        503,
    );
    let width_px = (gesture.dynamics.base_width_px
        * pass.width_multiplier
        * npr_pressure_multiplier(distance_t, settings)
        * npr_taper_multiplier(distance_t, gesture.style.taper)
        * npr_distance_width_multiplier(gesture.importance, settings)
        + width_noise * settings.pressure_jitter * brush.pressure_jitter_multiplier)
        .max(0.25);
    let pass_offset = npr_pass_offset_px(brush_path.path_id, distance_t, settings, pass.pass_index);
    let color = ColorRgba::new(
        pass.color.r,
        pass.color.g,
        pass.color.b,
        (pass.color.a
            * npr_alpha_pressure_multiplier(distance_t, settings)
            * brush.alpha_multiplier
            * npr_depth_alpha_multiplier(gesture.importance, settings))
        .clamp(0.0, 1.0),
    );

    NprStrokeStripSample {
        point,
        width_px,
        offset_px: pass_offset,
        overshoot_px: pass.overshoot_px,
        color,
    }
}

pub(crate) fn append_npr_stroke_strip_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    samples: &[NprStrokeStripSample],
) {
    if samples.len() < 2 {
        return;
    }

    let mut previous_rail = npr_stroke_rail(samples, viewport, 0);
    for index in 1..samples.len() {
        let Some(a) = previous_rail else {
            previous_rail = npr_stroke_rail(samples, viewport, index);
            continue;
        };
        let Some(b) = npr_stroke_rail(samples, viewport, index) else {
            continue;
        };
        let color = ColorRgba::new(
            (a.color.r + b.color.r) * 0.5,
            (a.color.g + b.color.g) * 0.5,
            (a.color.b + b.color.b) * 0.5,
            (a.color.a + b.color.a) * 0.5,
        );
        vertices.push(ColorVertex::new(a.left, color));
        vertices.push(ColorVertex::new(b.left, color));
        vertices.push(ColorVertex::new(b.right, color));
        vertices.push(ColorVertex::new(a.left, color));
        vertices.push(ColorVertex::new(b.right, color));
        vertices.push(ColorVertex::new(a.right, color));
        previous_rail = Some(b);
    }
}

pub(crate) fn append_npr_stroke_strip_segments(
    segments: &mut Vec<NprStrokeSegmentVertex>,
    viewport: &Viewport,
    samples: &[NprStrokeStripSample],
) {
    if samples.len() < 2 {
        return;
    }

    for index in 1..samples.len() {
        let start = samples[index - 1];
        let end = samples[index];
        let dx = (end.point.x - start.point.x) * viewport.half_width;
        let dy = (end.point.y - start.point.y) * viewport.half_height;
        if dx * dx + dy * dy <= f32::EPSILON {
            continue;
        }
        let color = ColorRgba::new(
            (start.color.r + end.color.r) * 0.5,
            (start.color.g + end.color.g) * 0.5,
            (start.color.b + end.color.b) * 0.5,
            (start.color.a + end.color.a) * 0.5,
        );
        segments.push(NprStrokeSegmentVertex {
            start: [
                start.point.x * viewport.half_width,
                start.point.y * viewport.half_height,
            ],
            end: [
                end.point.x * viewport.half_width,
                end.point.y * viewport.half_height,
            ],
            color: [color.r, color.g, color.b, color.a],
            width_px: (start.width_px + end.width_px) * 0.5,
            offset_px: (start.offset_px + end.offset_px) * 0.5,
            overshoot_start_px: if index == 1 { start.overshoot_px } else { 0.0 },
            overshoot_end_px: if index + 1 == samples.len() {
                end.overshoot_px
            } else {
                0.0
            },
            viewport_half: [viewport.half_width, viewport.half_height],
            end_width_px: end.width_px,
            end_alpha: end.color.a,
        });
    }
}

fn npr_stroke_rail(
    samples: &[NprStrokeStripSample],
    viewport: &Viewport,
    index: usize,
) -> Option<NprStrokeRail> {
    let sample = samples.get(index)?;
    let previous = samples[index.saturating_sub(1)].point;
    let next = samples[(index + 1).min(samples.len() - 1)].point;
    let tangent_px = Vec2::new(
        (next.x - previous.x) * viewport.half_width,
        (next.y - previous.y) * viewport.half_height,
    );
    let length = (tangent_px.x * tangent_px.x + tangent_px.y * tangent_px.y).sqrt();
    if length <= f32::EPSILON {
        return None;
    }

    let direction = Vec2::new(tangent_px.x / length, tangent_px.y / length);
    let normal = Vec2::new(-direction.y, direction.x);
    let endpoint_sign = if index == 0 {
        -1.0
    } else if index + 1 == samples.len() {
        1.0
    } else {
        0.0
    };
    let center_px = Vec2::new(
        sample.point.x * viewport.half_width
            + direction.x * sample.overshoot_px * endpoint_sign
            + normal.x * sample.offset_px,
        sample.point.y * viewport.half_height
            + direction.y * sample.overshoot_px * endpoint_sign
            + normal.y * sample.offset_px,
    );
    let half_width = sample.width_px * 0.5;
    let left = Vec2::new(
        (center_px.x + normal.x * half_width) / viewport.half_width,
        (center_px.y + normal.y * half_width) / viewport.half_height,
    );
    let right = Vec2::new(
        (center_px.x - normal.x * half_width) / viewport.half_width,
        (center_px.y - normal.y * half_width) / viewport.half_height,
    );
    Some(NprStrokeRail {
        left,
        right,
        color: sample.color,
    })
}
