use amigo_math::{ColorRgba, Vec2};

use crate::renderer::{
    ColorVertex, NprStableBrushPath, NprStrokeGesture, NprStrokePassPlan, NprStrokeRail,
    NprLineKind, NprStrokeSegmentVertex, NprStrokeStripSample, Viewport, npr_alpha_pressure_multiplier,
    npr_depth_alpha_multiplier, npr_distance_width_multiplier,
    npr_preferred_stroke_length_px_with_traits, npr_pressure_multiplier,
};

use super::{NprLineCandidateTraits, coherent_signed_noise_1d, resolve_npr_brush_profile_with_traits};

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

fn brush_angle_response(
    brush: super::NprResolvedBrushProfile3d,
    tangent: Vec2,
) -> f32 {
    let tangent_length = (tangent.x * tangent.x + tangent.y * tangent.y).sqrt();
    if tangent_length <= f32::EPSILON || brush.angle_influence <= f32::EPSILON {
        return 1.0;
    }
    let direction = Vec2::new(tangent.x / tangent_length, tangent.y / tangent_length);
    let brush_angle = brush.angle_bias_radians;
    let brush_axis = Vec2::new(brush_angle.cos(), brush_angle.sin());
    let alignment = (direction.x * brush_axis.x + direction.y * brush_axis.y).abs();
    let nib_factor = match brush.tip {
        amigo_render_api::NprBrushTip3d::Round => 0.94 + (1.0 - alignment) * 0.12,
        amigo_render_api::NprBrushTip3d::Flat => 0.72 + (1.0 - alignment) * 0.56,
        amigo_render_api::NprBrushTip3d::GPen => 0.80 + (1.0 - alignment) * 0.40,
        amigo_render_api::NprBrushTip3d::MaruPen => 0.90 + (1.0 - alignment) * 0.16,
        amigo_render_api::NprBrushTip3d::DryBrush => 0.76 + (1.0 - alignment) * 0.48,
    };
    1.0 + (nib_factor - 1.0) * brush.angle_influence.clamp(0.0, 1.0)
}

pub(crate) fn humanize_npr_brush_sample(
    brush_path: &NprStableBrushPath,
    index: usize,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
    pass: u8,
    wobble_px: f32,
    viewport: &Viewport,
) -> Vec2 {
    let traits = NprLineCandidateTraits {
        technical_detail: gesture.technical_detail,
        material_detail: gesture.material_detail,
        material_seam: gesture.material_seam,
    };
    let brush = resolve_npr_brush_profile_with_traits(gesture.kind, traits, settings);
    let micro_wobble_px = settings.micro_wobble_px
        * settings.humanization
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

    let mut normal = Vec2::new(-ty / length, tx / length);
    if brush.angle_bias_radians.abs() > f32::EPSILON {
        let cos_theta = brush.angle_bias_radians.cos();
        let sin_theta = brush.angle_bias_radians.sin();
        normal = Vec2::new(
            normal.x * cos_theta - normal.y * sin_theta,
            normal.x * sin_theta + normal.y * cos_theta,
        );
    }
    let point = brush_path.samples[index].point;
    let arc_t = brush_path.samples[index].arc_length_px / brush_path.length_px;
    let endpoint_lock = npr_endpoint_lock(arc_t, brush_path.length_px, settings)
        * brush.path_adherence_multiplier.clamp(0.35, 1.0);
    let primary = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.stroke_wobble_frequency.max(0.01) * 100.0,
        919,
    );
    let hand_arc = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.stroke_wobble_frequency.max(0.01) * 18.0 + 1.7,
        887,
    );
    let bow_seed = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        0.5,
        1061,
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
        settings.local_angular_drift_degrees.to_radians().sin()
            * settings.humanization
            * brush.tangent_drift_multiplier
            * gesture.role.tangent_drift_multiplier;
    let preferred_length_px =
        npr_preferred_stroke_length_px_with_traits(gesture.kind, traits, settings).max(24.0);
    let hand_arc_scale = ((brush_path.length_px / preferred_length_px)
        .clamp(0.35, 1.15)
        * wobble_px
        * 0.65
        * brush.hand_arc_multiplier
        * gesture.role.hand_arc_multiplier)
        .max(0.0);
    let bow_scale = npr_seeded_feature_bow_scale(
        gesture,
        brush_path.length_px,
        preferred_length_px,
        wobble_px,
        brush.hand_arc_multiplier,
    );
    let seeded_bow = (arc_t * std::f32::consts::PI).sin() * bow_seed * bow_scale;
    let detail_crispness = if brush_path.length_px < preferred_length_px {
        1.0
    } else {
        0.82
    } * brush.detail_crispness_multiplier
        * gesture.role.detail_crispness;
    let px = point.x * viewport.half_width
        + normal.x * hand_arc * hand_arc_scale * endpoint_lock
        + normal.x * seeded_bow * endpoint_lock
        + normal.x * primary * wobble_px * detail_crispness * endpoint_lock
        + normal.x * micro * micro_wobble_px * endpoint_lock
        + (tx / length) * drift * wobble_px * tangent_scale * endpoint_lock;
    let py = point.y * viewport.half_height
        + normal.y * hand_arc * hand_arc_scale * endpoint_lock
        + normal.y * seeded_bow * endpoint_lock
        + normal.y * primary * wobble_px * detail_crispness * endpoint_lock
        + normal.y * micro * micro_wobble_px * endpoint_lock
        + (ty / length) * drift * wobble_px * tangent_scale * endpoint_lock;
    Vec2::new(px / viewport.half_width, py / viewport.half_height)
}

fn npr_seeded_feature_bow_scale(
    gesture: NprStrokeGesture,
    path_length_px: f32,
    preferred_length_px: f32,
    wobble_px: f32,
    brush_hand_arc_multiplier: f32,
) -> f32 {
    if !matches!(
        gesture.kind,
        NprLineKind::Feature | NprLineKind::Crease | NprLineKind::Seam
    ) {
        return 0.0;
    }
    if path_length_px < 18.0 {
        return 0.0;
    }

    let length_factor = (path_length_px / preferred_length_px.max(24.0)).clamp(0.35, 1.35);
    let detail_factor = if gesture.kind == NprLineKind::Feature {
        1.0
    } else {
        0.72
    };
    (wobble_px.max(0.18)
        * 1.15
        * length_factor
        * detail_factor
        * brush_hand_arc_multiplier
        * gesture.role.hand_arc_multiplier)
        .clamp(0.0, 2.4)
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
    let brush = resolve_npr_brush_profile_with_traits(
        gesture.kind,
        NprLineCandidateTraits {
            technical_detail: gesture.technical_detail,
            material_detail: gesture.material_detail,
            material_seam: gesture.material_seam,
        },
        settings,
    );
    let point = humanize_npr_brush_sample(
        brush_path,
        point_index,
        settings,
        gesture,
        pass.pass_index,
        pass.wobble_px,
        viewport,
    );
    let taper = (gesture.style.taper * gesture.role.taper_multiplier * brush.taper_multiplier)
        .clamp(0.0, 1.5);
    let prev = brush_path.samples[point_index.saturating_sub(1)].point;
    let next = brush_path.samples[(point_index + 1).min(brush_path.samples.len() - 1)].point;
    let tangent = Vec2::new(
        (next.x - prev.x) * viewport.half_width,
        (next.y - prev.y) * viewport.half_height,
    );
    let angle_response = brush_angle_response(brush, tangent);
    let width_noise = coherent_signed_noise_1d(
        settings.seed,
        gesture.path_seed,
        pass.pass_index as u64,
        distance_t * settings.stroke_wobble_frequency.max(0.01) * 100.0 + 7.0,
        503,
    );
    let width_px = (gesture.dynamics.base_width_px
        * pass.width_multiplier
        * npr_pressure_multiplier(distance_t, settings, brush)
        * npr_taper_multiplier(distance_t, taper)
        * npr_distance_width_multiplier(gesture.importance, settings)
        * angle_response
        + width_noise * settings.pressure_jitter * brush.pressure_jitter_multiplier)
        .max(0.25);
    let pass_offset = npr_pass_offset_px(brush_path.path_id, distance_t, settings, pass.pass_index);
    let color = ColorRgba::new(
        pass.color.r,
        pass.color.g,
        pass.color.b,
        (pass.color.a
            * npr_alpha_pressure_multiplier(distance_t, brush)
            * brush.alpha_multiplier
            * gesture.role.alpha_multiplier
            * npr_depth_alpha_multiplier(gesture.importance, settings)
            * (1.0 + (angle_response - 1.0) * 0.22))
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
