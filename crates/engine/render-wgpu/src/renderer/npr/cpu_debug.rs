use amigo_math::{ColorRgba, Vec2};

use crate::renderer::{
    ColorVertex, NprDebugOverlay3d, NprLineKind, NprStrokePassKind, NprStrokePath, Viewport,
    build_npr_dropout_mask, build_npr_stable_brush_path, build_npr_stroke_gesture,
    build_npr_stroke_pass_plan, deterministic_noise, npr_cpu_tessellation_profile,
    npr_stroke_strip_sample, push_quad, screen_segment_length_px,
};

use super::types::NprRejectedTechnicalCandidate;

pub(crate) fn append_npr_rejected_technical_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    rejected: &[NprRejectedTechnicalCandidate],
) {
    for candidate in rejected {
        append_npr_debug_segment(
            vertices,
            viewport,
            candidate.p0,
            candidate.p1,
            npr_rejected_technical_debug_color(*candidate),
            1.5,
        );
    }
}

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
                npr_path_topology_debug_color(path),
                npr_path_debug_width_px(path, viewport),
            );
        }
        NprDebugOverlay3d::CandidateImportance => {
            append_npr_debug_polyline(
                vertices,
                viewport,
                &path.points,
                npr_candidate_importance_debug_color(path),
                2.5,
            );
        }
        NprDebugOverlay3d::TechnicalSelection => {
            append_npr_debug_polyline(
                vertices,
                viewport,
                &path.points,
                npr_technical_selection_debug_color(path, viewport),
                2.25,
            );
        }
        NprDebugOverlay3d::StrokeLengthBucket => {
            append_npr_debug_polyline(
                vertices,
                viewport,
                &path.points,
                npr_length_bucket_debug_color(path, viewport),
                npr_path_debug_width_px(path, viewport),
            );
        }
        NprDebugOverlay3d::SourceEdgeCount => {
            append_npr_debug_polyline(
                vertices,
                viewport,
                &path.points,
                npr_source_edge_count_debug_color(path),
                npr_source_edge_count_debug_width_px(path),
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
    let brush_path =
        build_npr_stable_brush_path(path, viewport, npr_cpu_tessellation_profile(settings));
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
    let brush_path =
        build_npr_stable_brush_path(path, viewport, npr_cpu_tessellation_profile(settings));
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

fn npr_path_topology_debug_color(path: &NprStrokePath) -> ColorRgba {
    let base = npr_path_id_debug_color(path.path_id);
    let source_edge_factor = (path.source_edges.len() as f32 / 10.0).clamp(0.0, 1.0);
    ColorRgba::new(
        (base.r * 0.45 + source_edge_factor * 0.55).clamp(0.0, 1.0),
        base.g,
        (base.b * 0.65 + (1.0 - source_edge_factor) * 0.35).clamp(0.0, 1.0),
        base.a,
    )
}

fn npr_path_debug_width_px(path: &NprStrokePath, viewport: &Viewport) -> f32 {
    let length_px = npr_path_length_px(path, viewport);
    if length_px >= 96.0 {
        3.5
    } else if length_px >= 42.0 {
        2.5
    } else {
        1.5
    }
}

fn npr_candidate_importance_debug_color(path: &NprStrokePath) -> ColorRgba {
    let importance = path.candidate_importance.clamp(0.0, 1.0);
    let source_edge_factor = (path.source_edges.len() as f32 / 8.0).clamp(0.0, 1.0);
    ColorRgba::new(
        (1.0 - importance) * 0.95,
        (importance * 0.85 + source_edge_factor * 0.15).clamp(0.0, 1.0),
        (0.25 + source_edge_factor * 0.65).clamp(0.0, 1.0),
        0.92,
    )
}

fn npr_technical_selection_debug_color(path: &NprStrokePath, viewport: &Viewport) -> ColorRgba {
    let length_px = npr_path_length_px(path, viewport);
    let chain_factor = (path.source_edges.len() as f32 / 6.0).clamp(0.0, 1.0);
    match path.kind {
        NprLineKind::Silhouette | NprLineKind::Boundary => ColorRgba::new(0.12, 0.92, 0.20, 0.95),
        NprLineKind::Contact => ColorRgba::new(0.10, 0.70, 1.0, 0.95),
        NprLineKind::Crease | NprLineKind::Seam | NprLineKind::Feature => {
            let kept = if path.technical_detail {
                path.candidate_importance.clamp(0.0, 1.0) * 0.55
                    + (length_px / 72.0).clamp(0.0, 1.0) * 0.25
                    + chain_factor * 0.20
            } else {
                0.25
            };
            ColorRgba::new(1.0 - kept * 0.35, kept, 0.12, 0.92)
        }
    }
}

fn npr_length_bucket_debug_color(path: &NprStrokePath, viewport: &Viewport) -> ColorRgba {
    let length_px = npr_path_length_px(path, viewport);
    if length_px >= 96.0 {
        ColorRgba::new(0.12, 0.92, 0.25, 0.95)
    } else if length_px >= 48.0 {
        ColorRgba::new(1.0, 0.82, 0.10, 0.95)
    } else {
        ColorRgba::new(1.0, 0.28, 0.18, 0.95)
    }
}

fn npr_source_edge_count_debug_color(path: &NprStrokePath) -> ColorRgba {
    let source_edges = path.source_edges.len() as f32;
    let normalized = (source_edges / 10.0).clamp(0.0, 1.0);
    ColorRgba::new(
        (0.10 + normalized * 0.25).clamp(0.0, 1.0),
        (0.22 + normalized * 0.78).clamp(0.0, 1.0),
        (1.0 - normalized * 0.78).clamp(0.15, 1.0),
        0.94,
    )
}

fn npr_source_edge_count_debug_width_px(path: &NprStrokePath) -> f32 {
    match path.source_edges.len() {
        0..=1 => 1.5,
        2..=3 => 2.25,
        4..=7 => 3.0,
        _ => 3.75,
    }
}

fn npr_rejected_technical_debug_color(candidate: NprRejectedTechnicalCandidate) -> ColorRgba {
    let importance = candidate.candidate_importance.clamp(0.0, 1.0);
    let kind_bias = match candidate.kind {
        NprLineKind::Crease => 0.12f32,
        NprLineKind::Seam => 0.08f32,
        NprLineKind::Feature => 0.16f32,
        _ => 0.0f32,
    };
    let source_bias = ((candidate.source_edge_id % 13) as f32 / 13.0) * 0.10;
    ColorRgba::new(
        1.0,
        (0.10f32 + importance * 0.25f32 + source_bias).clamp(0.0, 1.0),
        (0.08f32 + kind_bias).clamp(0.0, 1.0),
        0.82,
    )
}

fn npr_path_length_px(path: &NprStrokePath, viewport: &Viewport) -> f32 {
    path.points
        .windows(2)
        .map(|segment| screen_segment_length_px(segment[0], segment[1], viewport))
        .sum::<f32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_feature_path() -> NprStrokePath {
        NprStrokePath {
            path_id: 90,
            kind: NprLineKind::Feature,
            candidate_importance: 1.0,
            technical_detail: true,
            material_detail: false,
            material_seam: false,
            points: vec![Vec2::new(-0.50, 0.0), Vec2::new(0.50, 0.0)],
            source_edges: vec![1, 2, 3],
            sorted_source_edges: vec![1, 2, 3],
            arc_lengths_px: vec![0.0, 400.0],
            importance: 0.8,
            closed: false,
        }
    }

    #[test]
    fn width_alpha_debug_uses_preset_tessellation_profile() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = long_feature_path();
        let mut dense = amigo_render_api::NprLineSettings3d::default();
        dense.cpu_strategy_profile.tessellation.resample_spacing_px = 4.0;
        let mut coarse = dense.clone();
        coarse.cpu_strategy_profile.tessellation.resample_spacing_px = 64.0;
        let mut dense_vertices = Vec::new();
        let mut coarse_vertices = Vec::new();

        append_npr_debug_path_vertices(
            &mut dense_vertices,
            &viewport,
            &path,
            &dense,
            NprDebugOverlay3d::WidthAlpha,
        );
        append_npr_debug_path_vertices(
            &mut coarse_vertices,
            &viewport,
            &path,
            &coarse,
            NprDebugOverlay3d::WidthAlpha,
        );

        assert!(dense_vertices.len() > coarse_vertices.len() * 4);
    }
}
