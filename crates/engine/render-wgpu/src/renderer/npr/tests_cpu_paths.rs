use amigo_math::{Transform3, Vec2, Vec3};

use crate::renderer::*;

#[test]
fn npr_fragments_join_into_quantized_stroke_path() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let fragments = vec![
        NprLineFragment {
            source_edge_id: 11,
            kind: NprLineKind::Feature,
            p0: Vec2::new(-0.2, 0.0),
            p1: Vec2::new(0.0, 0.0),
            t0: 0.0,
            t1: 0.5,
            tangent0: Vec2::new(1.0, 0.0),
            tangent1: Vec2::new(1.0, 0.0),
            avg_depth: 1.0,
        },
        NprLineFragment {
            source_edge_id: 11,
            kind: NprLineKind::Feature,
            p0: Vec2::new(0.002, 0.001),
            p1: Vec2::new(0.2, 0.0),
            t0: 0.5,
            t1: 1.0,
            tangent0: Vec2::new(1.0, 0.0),
            tangent1: Vec2::new(1.0, 0.0),
            avg_depth: 1.0,
        },
    ];

    let paths = build_npr_stroke_paths(&fragments, &viewport, 2.5, 0.0);

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].kind, NprLineKind::Feature);
    assert_eq!(paths[0].points.len(), 3);
}

#[test]
fn npr_stable_path_id_uses_source_edges_not_projection() {
    let forward = stable_path_id(NprLineKind::Silhouette, &[11, 22, 33]);
    let reversed = stable_path_id(NprLineKind::Silhouette, &[33, 22, 11]);
    let crease = stable_path_id(NprLineKind::Crease, &[11, 22, 33]);

    assert_eq!(forward, reversed);
    assert_ne!(forward, crease);
}

#[test]
fn npr_dihedral_feature_edges_classify_as_crease_lines() {
    let settings = amigo_render_api::NprLineSettings3d {
        feature: true,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    assert_eq!(
        npr_line_kind_for_edge(&settings, false, false, true, false, false, false),
        Some(NprLineKind::Crease)
    );
}

#[test]
fn npr_material_seam_edges_classify_as_seam_lines() {
    let settings = amigo_render_api::NprLineSettings3d {
        feature: true,
        ..amigo_render_api::NprLineSettings3d::default()
    };

    assert_eq!(
        npr_line_kind_for_edge(&settings, false, false, false, true, false, false),
        Some(NprLineKind::Seam)
    );
}

#[test]
fn npr_ink_detail_material_edges_use_lower_cpu_length_threshold() {
    let settings = amigo_render_api::NprLineSettings3d {
        min_screen_length_px: 10.0,
        ink_detail_material_ids: vec![7],
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let edge = MeshEdge3d {
        edge_id: 1,
        a: 0,
        b: 1,
        faces: vec![0, 1],
        material_seam: false,
    };
    let triangles = vec![
        MeshTriangle3d {
            indices: [0, 1, 2],
            normal: Vec3::new(0.0, 0.0, 1.0),
            material_id: Some(7),
        },
        MeshTriangle3d {
            indices: [1, 3, 2],
            normal: Vec3::new(0.0, 0.0, 1.0),
            material_id: Some(2),
        },
    ];

    assert_eq!(
        npr_edge_min_screen_length_px(&settings, &edge, &triangles),
        5.5
    );
}

#[test]
fn npr_non_ink_detail_material_edges_keep_cpu_length_threshold() {
    let settings = amigo_render_api::NprLineSettings3d {
        min_screen_length_px: 10.0,
        ink_detail_material_ids: vec![7],
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let edge = MeshEdge3d {
        edge_id: 1,
        a: 0,
        b: 1,
        faces: vec![0],
        material_seam: false,
    };
    let triangles = vec![MeshTriangle3d {
        indices: [0, 1, 2],
        normal: Vec3::new(0.0, 0.0, 1.0),
        material_id: Some(2),
    }];

    assert_eq!(
        npr_edge_min_screen_length_px(&settings, &edge, &triangles),
        10.0
    );
}

#[test]
fn npr_path_simplification_removes_nearly_collinear_points() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let points = vec![
        Vec2::new(-0.5, 0.0),
        Vec2::new(0.0, 0.0005),
        Vec2::new(0.5, 0.0),
    ];

    let simplified = simplify_npr_path(&points, &viewport, 1.0);

    assert_eq!(simplified, vec![points[0], points[2]]);
}

#[test]
fn npr_visibility_fragments_require_owned_face_samples() {
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let edge = MeshEdge3d {
        edge_id: 11,
        a: 0,
        b: 1,
        faces: vec![0],
        material_seam: false,
    };
    let a = ProjectedPoint {
        position: Vec2::new(-0.25, 0.0),
        depth: 2.0,
    };
    let b = ProjectedPoint {
        position: Vec2::new(0.25, 0.0),
        depth: 2.0,
    };
    let owned_visibility = NprFaceVisibilityBuffer {
        width: 16,
        height: 16,
        face_id: vec![0; 16 * 16],
        face_visible: vec![true],
    };
    let hidden_visibility = NprFaceVisibilityBuffer {
        width: 16,
        height: 16,
        face_id: vec![1; 16 * 16],
        face_visible: vec![true, true],
    };

    let visible = visible_npr_fragments_for_edge(
        &owned_visibility,
        &edge,
        NprLineKind::Boundary,
        a,
        b,
        &viewport,
        1.0,
    );
    let hidden = visible_npr_fragments_for_edge(
        &hidden_visibility,
        &edge,
        NprLineKind::Boundary,
        a,
        b,
        &viewport,
        1.0,
    );

    assert_eq!(visible.len(), 1);
    assert!(hidden.is_empty());
}

#[test]
fn npr_lines_skip_back_facing_feature_edges() {
    let geometry = cube_geometry();
    let viewport = Viewport::from_dimensions(800.0, 600.0);
    let settings = amigo_render_api::NprLineSettings3d {
        boundary: false,
        silhouette: false,
        feature: true,
        feature_angle_degrees: 1.0,
        min_screen_length_px: 0.0,
        width_px: 2.0,
        pressure_jitter: 0.0,
        stroke_wobble_px: 0.0,
        overshoot_px: 0.0,
        dropout: 0.0,
        passes: 1,
        ..amigo_render_api::NprLineSettings3d::default()
    };
    let mut vertices = Vec::new();
    append_mesh_npr_line_vertices(
        &mut vertices,
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
    assert!(
        vertices.is_empty(),
        "feature-only pass should not draw back-facing cube edges"
    );

    let mut silhouette_vertices = Vec::new();
    append_mesh_npr_line_vertices(
        &mut silhouette_vertices,
        &viewport,
        Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        },
        amigo_render_api::Camera3dRenderSettings::default(),
        &geometry,
        Transform3::default(),
        &amigo_render_api::NprLineSettings3d {
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
        },
    );
    assert!(!silhouette_vertices.is_empty());
}
