use amigo_math::Vec2;

use crate::renderer::{
    ColorVertex, NprBrushSample, NprCachedStrokePlan, NprLineKind, NprStableBrushPath,
    NprStrokeFrameStats3d, NprStrokeGesture, NprStrokePath, NprStrokeSegmentVertex,
    NprToolDynamics, Viewport, append_npr_stroke_strip_segments,
    append_npr_stroke_strip_vertices, build_empty_npr_cached_stroke_plan,
    build_npr_cached_stroke_plan, npr_stroke_strip_sample, screen_segment_length_px,
};

use super::{
    NprLineCandidateTraits, resolve_npr_brush_profile_with_traits,
    resolve_npr_gesture_role_profile_with_traits, resolve_npr_kind_style_with_traits,
};

const NPR_BRUSH_RESAMPLE_SPACING_PX: f32 = 2.5;

pub(crate) fn build_npr_stable_brush_path(
    path: &NprStrokePath,
    viewport: &Viewport,
) -> NprStableBrushPath {
    if path.points.len() < 2 {
        return NprStableBrushPath {
            path_id: path.path_id,
            samples: path
                .points
                .iter()
                .copied()
                .map(|point| NprBrushSample {
                    point,
                    arc_length_px: 0.0,
                })
                .collect(),
            length_px: 0.0,
        };
    }

    let estimated_samples = path
        .arc_lengths_px
        .last()
        .copied()
        .map(|length| (length / NPR_BRUSH_RESAMPLE_SPACING_PX).ceil() as usize + 1)
        .unwrap_or(path.points.len())
        .max(path.points.len());
    let mut samples = Vec::with_capacity(estimated_samples);
    let mut total = 0.0;
    samples.push(NprBrushSample {
        point: path.points[0],
        arc_length_px: 0.0,
    });

    for index in 1..path.points.len() {
        let start = path.points[index - 1];
        let end = path.points[index];
        let segment_length = screen_segment_length_px(start, end, viewport);
        if segment_length <= f32::EPSILON {
            continue;
        }

        let steps = (segment_length / NPR_BRUSH_RESAMPLE_SPACING_PX).floor() as usize;
        for step in 1..=steps {
            let local_t =
                (step as f32 * NPR_BRUSH_RESAMPLE_SPACING_PX / segment_length).clamp(0.0, 1.0);
            if local_t >= 1.0 {
                continue;
            }
            samples.push(NprBrushSample {
                point: Vec2::new(
                    start.x + (end.x - start.x) * local_t,
                    start.y + (end.y - start.y) * local_t,
                ),
                arc_length_px: total + segment_length * local_t,
            });
        }

        total += segment_length;
        samples.push(NprBrushSample {
            point: end,
            arc_length_px: total,
        });
    }

    NprStableBrushPath {
        path_id: path.path_id,
        samples,
        length_px: total.max(1.0),
    }
}

pub(crate) fn build_npr_stroke_gesture(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprStrokeGesture {
    let traits = NprLineCandidateTraits {
        technical_detail: path.technical_detail,
        material_detail: path.material_detail,
        material_seam: path.material_seam,
    };
    let style = resolve_npr_kind_style_with_traits(path.kind, traits, settings);
    let path_length_px = path.arc_lengths_px.last().copied().unwrap_or(0.0).max(1.0);
    let role = resolve_npr_gesture_role_profile_with_traits(path.kind, path_length_px, traits, settings);
    let brush = resolve_npr_brush_profile_with_traits(path.kind, traits, settings);
    let importance = if path.technical_detail {
        (path.importance * (0.82 + path.candidate_importance.clamp(0.0, 1.0) * 0.18))
            .clamp(0.45, 1.2)
    } else {
        path.importance.clamp(0.55, 1.35)
    };
    let dynamics = NprToolDynamics {
        base_width_px: settings.width_px
            * style.width_multiplier
            * importance
            * brush.width_multiplier,
        base_wobble_px: style.wobble_px * settings.humanization * brush.path_wobble_multiplier,
        effective_overshoot_px: if path.closed {
            0.0
        } else {
            brush
                .overshoot_px
                .unwrap_or(style.overshoot_px)
                * role.overshoot_multiplier
        },
        edge_complexity: path.source_edges.len().max(1) as f32,
        protected_silhouette: path.kind == NprLineKind::Silhouette && path.importance >= 0.9,
    };

    NprStrokeGesture {
        kind: path.kind,
        path_seed: path.path_id,
        path_length_px,
        importance,
        technical_detail: path.technical_detail,
        material_detail: path.material_detail,
        material_seam: path.material_seam,
        dynamics,
        style,
        role,
    }
}

pub(crate) fn append_npr_styled_path_vertices(
    vertices: &mut Vec<ColorVertex>,
    mut npr_stroke_segments: Option<&mut Vec<NprStrokeSegmentVertex>>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    cached_plan: Option<&NprCachedStrokePlan>,
    stats: &mut NprStrokeFrameStats3d,
) -> NprCachedStrokePlan {
    if path.points.len() < 2 {
        return build_empty_npr_cached_stroke_plan(settings);
    }

    let gesture = build_npr_stroke_gesture(path, settings);
    let brush_path = build_npr_stable_brush_path(path, viewport);
    if brush_path.samples.len() < 2 {
        return build_empty_npr_cached_stroke_plan(settings);
    }
    stats.brush_samples += brush_path.samples.len();
    let plan = if let Some(plan) = cached_plan.filter(|plan| plan.is_compatible(settings, gesture))
    {
        stats.cached_plan_hits += 1;
        plan.clone()
    } else {
        stats.cached_plan_misses += 1;
        build_npr_cached_stroke_plan(path, settings, gesture)
    };
    stats.dropout_intervals += plan.dropout.intervals.len();

    for pass in plan.passes.iter().copied() {
        stats.record_pass(pass);
        let mut strip_samples = Vec::with_capacity(brush_path.samples.len());
        for point_index in 1..brush_path.samples.len() {
            let segment_length = screen_segment_length_px(
                brush_path.samples[point_index - 1].point,
                brush_path.samples[point_index].point,
                viewport,
            );
            let segment_t0 =
                brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
            let segment_t1 = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
            if segment_t1 < pass.active_t0 || segment_t0 > pass.active_t1 {
                let before_vertices = vertices.len();
                if let Some(segments) = npr_stroke_segments.as_deref_mut() {
                    append_npr_stroke_strip_segments(segments, viewport, &strip_samples);
                    stats.strip_vertices += strip_samples.len().saturating_sub(1) * 6;
                } else {
                    append_npr_stroke_strip_vertices(vertices, viewport, &strip_samples);
                    stats.strip_vertices += vertices.len().saturating_sub(before_vertices);
                }
                strip_samples.clear();
                continue;
            }
            if !plan
                .dropout
                .keeps_segment(pass, segment_t0, segment_t1, segment_length)
            {
                let before_vertices = vertices.len();
                if let Some(segments) = npr_stroke_segments.as_deref_mut() {
                    append_npr_stroke_strip_segments(segments, viewport, &strip_samples);
                    stats.strip_vertices += strip_samples.len().saturating_sub(1) * 6;
                } else {
                    append_npr_stroke_strip_vertices(vertices, viewport, &strip_samples);
                    stats.strip_vertices += vertices.len().saturating_sub(before_vertices);
                }
                strip_samples.clear();
                continue;
            }

            if strip_samples.is_empty() {
                let distance_t =
                    brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
                strip_samples.push(npr_stroke_strip_sample(
                    &brush_path,
                    point_index - 1,
                    settings,
                    gesture,
                    pass,
                    distance_t,
                    viewport,
                ));
            }
            let distance_t = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
            strip_samples.push(npr_stroke_strip_sample(
                &brush_path,
                point_index,
                settings,
                gesture,
                pass,
                distance_t,
                viewport,
            ));
        }
        let before_vertices = vertices.len();
        if let Some(segments) = npr_stroke_segments.as_deref_mut() {
            append_npr_stroke_strip_segments(segments, viewport, &strip_samples);
            stats.strip_vertices += strip_samples.len().saturating_sub(1) * 6;
        } else {
            append_npr_stroke_strip_vertices(vertices, viewport, &strip_samples);
            stats.strip_vertices += vertices.len().saturating_sub(before_vertices);
        }
    }
    plan
}
