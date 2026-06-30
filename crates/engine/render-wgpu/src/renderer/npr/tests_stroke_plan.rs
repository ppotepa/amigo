use std::collections::BTreeMap;

use amigo_math::{Transform3, Vec2, Vec3};

use crate::renderer::*;

fn vertex_signature(vertices: &[ColorVertex]) -> Vec<[i32; 6]> {
    vertices
        .iter()
        .map(|vertex| {
            [
                (vertex.position[0] * 10_000.0).round() as i32,
                (vertex.position[1] * 10_000.0).round() as i32,
                (vertex.color[0] * 10_000.0).round() as i32,
                (vertex.color[1] * 10_000.0).round() as i32,
                (vertex.color[2] * 10_000.0).round() as i32,
                (vertex.color[3] * 10_000.0).round() as i32,
            ]
        })
        .collect()
}

fn test_npr_path(id: u64, points: &[(f32, f32)]) -> NprStrokePath {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let points = points
        .iter()
        .map(|(x, y)| Vec2::new(*x, *y))
        .collect::<Vec<_>>();
    NprStrokePath {
        path_id: id,
        kind: NprLineKind::Silhouette,
        candidate_importance: 1.0,
        technical_detail: false,
        material_detail: false,
        material_seam: false,
        source_edges: vec![id],
        sorted_source_edges: vec![id],
        arc_lengths_px: npr_path_arc_lengths(&points, &viewport),
        importance: 1.0,
        closed: false,
        points,
    }
}

fn test_npr_path_with_kind(id: u64, kind: NprLineKind, points: &[(f32, f32)]) -> NprStrokePath {
    let mut path = test_npr_path(id, points);
    path.kind = kind;
    path
}

#[test]
fn npr_stable_brush_path_resamples_by_arc_length() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let path = test_npr_path(913, &[(-0.5, 0.0), (0.5, 0.0)]);

    let brush_path = build_npr_stable_brush_path(&path, &viewport);

    assert!(brush_path.samples.len() > path.points.len());
    assert_eq!(brush_path.samples[0].point, path.points[0]);
    assert_eq!(
        brush_path.samples.last().expect("last sample").point,
        path.points[1]
    );
    assert!(
        brush_path
            .samples
            .windows(2)
            .all(|window| window[1].arc_length_px >= window[0].arc_length_px)
    );
}

#[test]
fn npr_stroke_pass_plan_does_not_search_duplicate_silhouettes() {
    let silhouette = test_npr_path(910, &[(-0.5, 0.0), (0.5, 0.0)]);
    let feature = test_npr_path_with_kind(911, NprLineKind::Crease, &[(-0.5, 0.0), (0.5, 0.0)]);
    let settings = amigo_render_api::NprLineSettings3d {
        style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
        stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
        passes: 1,
        search_line_count: 2,
        gpu_realtime_tuning: amigo_render_api::NprGpuRealtimeTuning3d {
            search_enabled: true,
            ..amigo_render_api::NprGpuRealtimeTuning3d::default()
        },
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let silhouette_plan = build_npr_stroke_pass_plan(
        &silhouette,
        &settings,
        build_npr_stroke_gesture(&silhouette, &settings),
    );
    let feature_plan = build_npr_stroke_pass_plan(
        &feature,
        &settings,
        build_npr_stroke_gesture(&feature, &settings),
    );

    assert_eq!(silhouette_plan.len(), 1);
    assert!(
        silhouette_plan
            .iter()
            .all(|pass| pass.kind == NprStrokePassKind::Primary)
    );
    assert_eq!(feature_plan.len(), 4);
    assert_eq!(
        feature_plan
            .iter()
            .filter(|pass| pass.kind == NprStrokePassKind::Search)
            .count(),
        3
    );
}

#[test]
fn npr_sparse_character_hatching_adds_short_cpu_hatch_pass_for_feature_lines() {
    let settings = amigo_render_api::NprLineSettings3d {
        pipeline: amigo_render_api::NprPipelineStrategies3d {
            candidate_strategy: amigo_render_api::NprCandidateStrategy3d::CharacterSemantic,
            stroke_strategy: amigo_render_api::NprStrokeStrategy3d::AkiraInk,
            hatching_strategy: amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching,
            budget_strategy: amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority,
            ..amigo_render_api::NprPipelineStrategies3d::default()
        },
        passes: 1,
        search_line_count: 0,
        width_px: 2.0,
        seed: 101084,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let hatch_plan = (900..1100)
        .find_map(|path_id| {
            let path = test_npr_path_with_kind(
                path_id,
                NprLineKind::Feature,
                &[(-0.03, 0.0), (0.03, 0.0)],
            );
            let plan = build_npr_stroke_pass_plan(
                &path,
                &settings,
                build_npr_stroke_gesture(&path, &settings),
            );
            plan.into_iter()
                .find(|pass| pass.kind == NprStrokePassKind::Hatch)
        })
        .expect("sparse hatching should deterministically select some feature paths");

    assert!(hatch_plan.active_t0 >= 0.0);
    assert!(hatch_plan.active_t1 <= 1.0);
    assert!(hatch_plan.active_t1 - hatch_plan.active_t0 < 0.8);
    assert!(hatch_plan.color.a < settings.ink_color.a);
}

#[test]
fn npr_dropout_mask_protects_primary_silhouette_segments() {
    let path = test_npr_path(912, &[(-0.5, 0.0), (0.0, 0.0), (0.5, 0.0)]);
    let settings = amigo_render_api::NprLineSettings3d {
        dropout: 1.0,
        dropout_segment_min_px: 1.0,
        passes: 1,
        search_line_count: 0,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let gesture = build_npr_stroke_gesture(&path, &settings);
    let pass = build_npr_stroke_pass_plan(&path, &settings, gesture)[0];
    let dropout = build_npr_dropout_mask(gesture, &settings, &[pass]);

    assert!(dropout.keeps_segment(pass, 0.0, 0.5, 100.0));
    assert!(dropout.keeps_segment(pass, 0.5, 1.0, 100.0));
}

#[test]
fn npr_cached_stroke_plan_reuses_dropout_for_stable_path() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let path = test_npr_path_with_kind(
        920,
        NprLineKind::Crease,
        &[(-0.5, 0.0), (0.0, 0.0), (0.5, 0.0)],
    );
    let settings = amigo_render_api::NprLineSettings3d {
        dropout: 0.8,
        dropout_segment_min_px: 1.0,
        passes: 1,
        search_line_count: 0,
        seed: 99,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let mut vertices = Vec::new();
    let mut stats = NprStrokeFrameStats3d::default();
    let first_plan = append_npr_styled_path_vertices(
        &mut vertices,
        None,
        &viewport,
        &path,
        &settings,
        None,
        &mut stats,
    );
    let mut second_vertices = Vec::new();
    let mut second_stats = NprStrokeFrameStats3d::default();
    let second_plan = append_npr_styled_path_vertices(
        &mut second_vertices,
        None,
        &viewport,
        &path,
        &settings,
        Some(&first_plan),
        &mut second_stats,
    );

    assert_eq!(
        first_plan.settings_signature,
        second_plan.settings_signature
    );
    assert_eq!(first_plan.length_bucket_px, second_plan.length_bucket_px);
    assert_eq!(
        first_plan.dropout.intervals.len(),
        second_plan.dropout.intervals.len()
    );
    assert_eq!(
        vertex_signature(&vertices),
        vertex_signature(&second_vertices)
    );
}

#[test]
fn npr_backend_stats_report_cached_plan_hits_on_second_frame() {
    let geometry = cube_geometry();
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let settings = amigo_render_api::NprLineSettings3d {
        boundary: false,
        silhouette: true,
        feature: false,
        min_screen_length_px: 0.0,
        width_px: 2.0,
        pressure_jitter: 0.0,
        stroke_wobble_px: 0.0,
        overshoot_px: 0.0,
        dropout: 0.0,
        passes: 1,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let mut history = BTreeMap::new();
    let mut first_vertices = Vec::new();
    let first = append_mesh_npr_line_vertices_with_history_and_stats(
        &mut history,
        1,
        "cube",
        &mut first_vertices,
        None,
        &viewport,
        Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        },
        amigo_render_api::Camera3dRenderSettings::default(),
        &geometry,
        Transform3::default(),
        &settings,
    );
    let mut second_vertices = Vec::new();
    let second = append_mesh_npr_line_vertices_with_history_and_stats(
        &mut history,
        2,
        "cube",
        &mut second_vertices,
        None,
        &viewport,
        Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        },
        amigo_render_api::Camera3dRenderSettings::default(),
        &geometry,
        Transform3::default(),
        &settings,
    );

    assert!(first.cached_plan_misses > 0);
    assert_eq!(first.cached_plan_hits, 0);
    assert!(second.cached_plan_hits > 0);
    assert!(second.cached_plan_misses < first.cached_plan_misses);
}

#[test]
fn npr_cached_stroke_plan_invalidates_when_seed_changes() {
    let path = test_npr_path_with_kind(
        921,
        NprLineKind::Crease,
        &[(-0.5, 0.0), (0.0, 0.0), (0.5, 0.0)],
    );
    let settings = amigo_render_api::NprLineSettings3d {
        dropout: 0.8,
        dropout_segment_min_px: 1.0,
        passes: 1,
        search_line_count: 0,
        seed: 99,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let changed_seed = amigo_render_api::NprLineSettings3d {
        seed: 100,
        ..settings.clone()
    };
    let gesture = build_npr_stroke_gesture(&path, &settings);
    let cached = build_npr_cached_stroke_plan(&path, &settings, gesture);

    assert!(!cached.is_compatible(
        &changed_seed,
        build_npr_stroke_gesture(&path, &changed_seed)
    ));
}
