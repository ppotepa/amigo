use amigo_math::Vec2;

use crate::renderer::*;

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

#[test]
fn npr_straightness_controls_humanized_path_deviation() {
    let path = test_npr_path(808, &[(-0.55, 0.0), (-0.15, 0.0), (0.2, 0.0), (0.55, 0.0)]);
    let loose = amigo_render_api::NprLineSettings3d {
        straightness: 0.0,
        stroke_wobble_px: 1.2,
        micro_wobble_px: 0.35,
        humanization: 1.0,
        passes: 1,
        search_line_count: 0,
        dropout: 0.0,
        pressure_jitter: 0.0,
        seed: 909,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let straight = amigo_render_api::NprLineSettings3d {
        straightness: 1.0,
        ..loose.clone()
    };
    let loose_brush = resolve_npr_brush_profile(NprLineKind::Silhouette, &loose);
    let straight_brush = resolve_npr_brush_profile(NprLineKind::Silhouette, &straight);
    let loose_gesture = build_npr_stroke_gesture(&path, &loose);
    let straight_gesture = build_npr_stroke_gesture(&path, &straight);

    assert!(
        loose_brush.path_wobble_multiplier > straight_brush.path_wobble_multiplier
            && loose_gesture.dynamics.base_wobble_px > straight_gesture.dynamics.base_wobble_px,
        "lower straightness should increase gesture deviation"
    );
}

#[test]
fn npr_alpha_pressure_curve_fades_stroke_along_arc_length() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let path = test_npr_path(909, &[(-0.6, 0.0), (-0.2, 0.0), (0.2, 0.0), (0.6, 0.0)]);
    let settings = amigo_render_api::NprLineSettings3d {
        alpha_pressure_curve: [1.0, 1.0, 0.25, 0.2],
        stroke_wobble_px: 0.0,
        micro_wobble_px: 0.0,
        pressure_jitter: 0.0,
        dropout: 0.0,
        passes: 1,
        search_line_count: 0,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let mut vertices = Vec::new();

    append_npr_styled_path_vertices(
        &mut vertices,
        None,
        &viewport,
        &path,
        &settings,
        None,
        &mut NprStrokeFrameStats3d::default(),
    );

    let max_alpha = vertices
        .iter()
        .map(|vertex| vertex.color[3])
        .fold(0.0, f32::max);
    let min_alpha = vertices
        .iter()
        .map(|vertex| vertex.color[3])
        .fold(1.0, f32::min);
    assert!(min_alpha < max_alpha * 0.35);
}

#[test]
fn npr_depth_alpha_modulates_stroke_opacity() {
    let settings = amigo_render_api::NprLineSettings3d {
        depth_alpha: 0.5,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let near = npr_depth_alpha_multiplier(1.2, &settings);
    let far = npr_depth_alpha_multiplier(0.2, &settings);

    assert!(near > far);
}

#[test]
fn npr_stroke_tool_profiles_change_drawing_dynamics() {
    let ink = amigo_render_api::NprLineSettings3d {
        stroke_tool: amigo_render_api::NprStrokeTool3d::InkPen,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let pencil = amigo_render_api::NprLineSettings3d {
        stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let brush = amigo_render_api::NprLineSettings3d {
        stroke_tool: amigo_render_api::NprStrokeTool3d::Brush,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let technical = amigo_render_api::NprLineSettings3d {
        stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let ink = crate::renderer::resolve_npr_brush_profile(NprLineKind::Silhouette, &ink);
    let pencil = crate::renderer::resolve_npr_brush_profile(NprLineKind::Silhouette, &pencil);
    let brush = crate::renderer::resolve_npr_brush_profile(NprLineKind::Silhouette, &brush);
    let technical = crate::renderer::resolve_npr_brush_profile(NprLineKind::Silhouette, &technical);

    assert!(pencil.search_multiplier > ink.search_multiplier);
    assert_eq!(technical.search_multiplier, 0.0);
    assert!(pencil.dropout_multiplier > ink.dropout_multiplier);
    assert_eq!(technical.dropout_multiplier, 0.0);
    assert!(brush.width_multiplier > ink.width_multiplier);
    assert!(pencil.alpha_multiplier < ink.alpha_multiplier);
    assert!(technical.pressure_jitter_multiplier < ink.pressure_jitter_multiplier);
}

#[test]
fn npr_brush_profile_author_scalars_override_tool_dynamics() {
    let settings = amigo_render_api::NprLineSettings3d {
        stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
        tool_search_multiplier: 0.0,
        tool_dropout_multiplier: 0.0,
        tool_alpha_multiplier: 0.5,
        tool_width_multiplier: 2.0,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let profile = crate::renderer::resolve_npr_brush_profile(NprLineKind::Silhouette, &settings);

    assert_eq!(profile.search_multiplier, 0.0);
    assert_eq!(profile.dropout_multiplier, 0.0);
    assert!(profile.alpha_multiplier < 0.5);
    assert!(profile.width_multiplier > 1.5);
}

#[test]
fn npr_line_family_brush_tip_and_role_override_feature_stroke_behavior() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let points = vec![
        Vec2::new(-0.3, 0.0),
        Vec2::new(0.0, 0.02),
        Vec2::new(0.28, 0.01),
    ];
    let settings = amigo_render_api::NprLineSettings3d {
        pipeline: amigo_render_api::NprPipelineStrategies3d {
            candidate_strategy: amigo_render_api::NprCandidateStrategy3d::CharacterSemantic,
            path_strategy: amigo_render_api::NprPathStrategy3d::StableStrokedPaths,
            stroke_strategy: amigo_render_api::NprStrokeStrategy3d::ConfidentMangaInk,
            budget_strategy: amigo_render_api::NprBudgetStrategy3d::CharacterReadability,
            temporal_strategy: amigo_render_api::NprTemporalStrategy3d::StableArcLength,
            ..amigo_render_api::NprPipelineStrategies3d::default()
        },
        brush_profiles: std::collections::BTreeMap::from([(
            "detail_pen".to_string(),
            amigo_render_api::NprBrushProfile3d {
                tool: Some(amigo_render_api::NprStrokeTool3d::TechnicalPen),
                tip: Some(amigo_render_api::NprBrushTip3d::MaruPen),
                ..amigo_render_api::NprBrushProfile3d::default()
            },
        )]),
        line_families: vec![amigo_render_api::NprLineFamily3d {
            id: "detail_ink".to_string(),
            role: Some(amigo_render_api::NprLineFamilyRole3d::DetailInk),
            sources: vec![amigo_render_api::NprLineSource3d::Feature],
            brush: Some("detail_pen".to_string()),
            ..amigo_render_api::NprLineFamily3d::default()
        }],
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let path = NprStrokePath {
        path_id: 404,
        kind: NprLineKind::Feature,
        candidate_importance: 1.0,
        technical_detail: true,
        material_detail: true,
        material_seam: false,
        source_edges: vec![1, 2],
        sorted_source_edges: vec![1, 2],
        arc_lengths_px: npr_path_arc_lengths(&points, &viewport),
        importance: 0.92,
        closed: false,
        points,
    };

    let brush = resolve_npr_brush_profile(NprLineKind::Feature, &settings);
    let gesture = build_npr_stroke_gesture(&path, &settings);

    assert_eq!(brush.tip, amigo_render_api::NprBrushTip3d::MaruPen);
    assert!(gesture.role.overshoot_multiplier < 0.3);
    assert!(gesture.role.detail_crispness > 1.1);
}
