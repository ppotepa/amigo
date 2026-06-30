use std::collections::BTreeMap;

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

fn test_npr_path_with_edges(id: u64, edges: Vec<u64>, points: &[(f32, f32)]) -> NprStrokePath {
    let mut path = test_npr_path(id, points);
    path.sorted_source_edges = sorted_npr_source_edges(&edges);
    path.source_edges = edges;
    path
}

#[test]
fn npr_temporal_history_retains_path_for_hysteresis_window() {
    let mut history = BTreeMap::new();
    let settings = amigo_render_api::NprLineSettings3d {
        visibility_hysteresis_frames: 3,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let first = stabilize_npr_paths_for_entity(
        &mut history,
        1,
        "entity",
        &settings,
        vec![test_npr_path(77, &[(-0.2, 0.0), (0.2, 0.0)])],
    );
    let retained = stabilize_npr_paths_for_entity(&mut history, 2, "entity", &settings, vec![]);
    let retained_again =
        stabilize_npr_paths_for_entity(&mut history, 3, "entity", &settings, vec![]);
    let dropped = stabilize_npr_paths_for_entity(&mut history, 4, "entity", &settings, vec![]);

    assert_eq!(first.len(), 1);
    assert_eq!(retained.len(), 1);
    assert_eq!(retained_again.len(), 1);
    assert!(dropped.is_empty());
}

#[test]
fn npr_temporal_history_blends_returning_path_points() {
    let mut history = BTreeMap::new();
    let settings = amigo_render_api::NprLineSettings3d {
        temporal_stability: 0.9,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let _ = stabilize_npr_paths_for_entity(
        &mut history,
        1,
        "entity",
        &settings,
        vec![test_npr_path(99, &[(-0.2, 0.0), (0.2, 0.0)])],
    );
    let blended = stabilize_npr_paths_for_entity(
        &mut history,
        2,
        "entity",
        &settings,
        vec![test_npr_path(99, &[(-0.2, 0.1), (0.2, 0.1)])],
    );

    assert_eq!(blended.len(), 1);
    assert!(blended[0].points[0].y > 0.0);
    assert!(blended[0].points[0].y < 0.1);
}

#[test]
fn npr_temporal_history_matches_changed_path_id_by_source_overlap() {
    let mut history = BTreeMap::new();
    let settings = amigo_render_api::NprLineSettings3d {
        temporal_stability: 0.9,
        visibility_hysteresis_frames: 4,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    let _ = stabilize_npr_paths_for_entity(
        &mut history,
        1,
        "entity",
        &settings,
        vec![test_npr_path_with_edges(
            100,
            vec![10, 20, 30],
            &[(-0.2, 0.0), (0.2, 0.0)],
        )],
    );
    let blended = stabilize_npr_paths_for_entity(
        &mut history,
        2,
        "entity",
        &settings,
        vec![test_npr_path_with_edges(
            101,
            vec![20, 30, 40],
            &[(-0.2, 0.08), (0.2, 0.08)],
        )],
    );

    assert_eq!(blended.len(), 1);
    assert_eq!(history.len(), 1);
    assert!(blended[0].points[0].y > 0.0);
    assert!(blended[0].points[0].y < 0.08);
}
