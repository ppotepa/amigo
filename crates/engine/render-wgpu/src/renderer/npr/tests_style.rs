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
        source_edges: vec![id],
        sorted_source_edges: vec![id],
        arc_lengths_px: npr_path_arc_lengths(&points, &viewport),
        importance: 1.0,
        closed: false,
        points,
    }
}

fn y_span(vertices: &[ColorVertex]) -> f32 {
    let min_y = vertices
        .iter()
        .map(|vertex| vertex.position[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = vertices
        .iter()
        .map(|vertex| vertex.position[1])
        .fold(f32::NEG_INFINITY, f32::max);
    max_y - min_y
}

#[test]
fn npr_straightness_controls_humanized_path_deviation() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
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
    let mut loose_vertices = Vec::new();
    let mut straight_vertices = Vec::new();

    append_npr_styled_path_vertices(
        &mut loose_vertices,
        None,
        &viewport,
        &path,
        &loose,
        None,
        &mut NprStrokeFrameStats3d::default(),
    );
    append_npr_styled_path_vertices(
        &mut straight_vertices,
        None,
        &viewport,
        &path,
        &straight,
        None,
        &mut NprStrokeFrameStats3d::default(),
    );

    let loose_span = y_span(&loose_vertices);
    let straight_span = y_span(&straight_vertices);
    assert!(
        loose_span > straight_span,
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

    let ink = crate::renderer::resolve_npr_brush_profile(&ink);
    let pencil = crate::renderer::resolve_npr_brush_profile(&pencil);
    let brush = crate::renderer::resolve_npr_brush_profile(&brush);
    let technical = crate::renderer::resolve_npr_brush_profile(&technical);

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

    let profile = crate::renderer::resolve_npr_brush_profile(&settings);

    assert_eq!(profile.search_multiplier, 0.0);
    assert_eq!(profile.dropout_multiplier, 0.0);
    assert!(profile.alpha_multiplier < 0.5);
    assert!(profile.width_multiplier > 1.5);
}
