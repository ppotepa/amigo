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
    fn npr_debug_overlay_maps_camera_debug_views() {
        assert_eq!(
            NprDebugOverlay3d::from_camera_debug_view(&amigo_render_api::CameraDebugView2d::new(
                "npr.line_kinds"
            )),
            Some(NprDebugOverlay3d::LineKinds)
        );
        assert_eq!(
            NprDebugOverlay3d::from_camera_debug_view(&amigo_render_api::CameraDebugView2d::new(
                "npr.dropout"
            )),
            Some(NprDebugOverlay3d::Dropout)
        );
        assert_eq!(
            NprDebugOverlay3d::from_camera_debug_view(
                &amigo_render_api::CameraDebugView2d::final_output()
            ),
            None
        );
    }

    #[test]
    fn npr_debug_overlay_emits_line_kind_vertices_without_changing_style_settings() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = test_npr_path_with_kind(
            17,
            NprLineKind::Crease,
            &[(-0.5, -0.25), (0.0, 0.1), (0.5, -0.25)],
        );
        let settings = amigo_render_api::NprLineSettings3d::default();
        let mut vertices = Vec::new();

        append_npr_debug_path_vertices(
            &mut vertices,
            &viewport,
            &path,
            &settings,
            NprDebugOverlay3d::LineKinds,
        );

        assert!(!vertices.is_empty());
        assert!(vertices.iter().any(|vertex| vertex.color[0] > 0.9));
    }

    #[test]
    fn npr_suggestive_edges_require_explicit_authoring() {
        let disabled = amigo_render_api::NprLineSettings3d {
            suggestive: false,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let enabled = amigo_render_api::NprLineSettings3d {
            suggestive: true,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_line_kind_for_edge(&disabled, false, false, false, false, true, false),
            None
        );
        assert_eq!(
            npr_line_kind_for_edge(&enabled, false, false, false, false, true, false),
            Some(NprLineKind::Feature)
        );
    }

    #[test]
    fn npr_contact_edges_require_explicit_authoring() {
        let disabled = amigo_render_api::NprLineSettings3d {
            contact: false,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let enabled = amigo_render_api::NprLineSettings3d {
            contact: true,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_line_kind_for_edge(&disabled, false, false, false, false, false, true),
            None
        );
        assert_eq!(
            npr_line_kind_for_edge(&enabled, false, false, false, false, false, true),
            Some(NprLineKind::Contact)
        );
    }

    #[test]
    fn npr_contact_authoring_reaches_backend_stats_and_vertices() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            boundary: true,
            silhouette: true,
            feature: true,
            suggestive: false,
            contact: true,
            contact_ground_y: 0.0,
            contact_threshold: 1.0,
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
        let mut history = BTreeMap::new();
        let stats = append_mesh_npr_line_vertices_with_history_and_stats(
            &mut history,
            1,
            "contact-cube",
            &mut vertices,
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

        assert!(
            stats.contact_paths > 0,
            "expected visible contact paths, got stats={stats:?}"
        );
        assert_eq!(stats.paths, stats.contact_paths);
        assert_eq!(stats.boundary_paths, 0);
        assert_eq!(stats.silhouette_paths, 0);
        assert_eq!(stats.crease_paths, 0);
        assert_eq!(stats.seam_paths, 0);
        assert_eq!(stats.feature_paths, 0);
        assert_eq!(stats.strip_vertices, vertices.len());
        assert!(!vertices.is_empty());
    }

    #[test]
    fn npr_line_segment_uses_pixel_width() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let mut vertices = Vec::new();
        append_npr_stroke_strip_vertices(
            &mut vertices,
            &viewport,
            &[
                NprStrokeStripSample {
                    point: Vec2::new(-0.5, 0.0),
                    width_px: 4.0,
                    offset_px: 0.0,
                    overshoot_px: 0.0,
                    color: ColorRgba::WHITE,
                },
                NprStrokeStripSample {
                    point: Vec2::new(0.5, 0.0),
                    width_px: 4.0,
                    offset_px: 0.0,
                    overshoot_px: 0.0,
                    color: ColorRgba::WHITE,
                },
            ],
        );
        assert_eq!(vertices.len(), 6);
        let height_ndc = (vertices[0].position[1] - vertices[2].position[1]).abs();
        assert!((height_ndc - (4.0 / viewport.half_height)).abs() < 0.0001);
    }

    #[test]
    fn npr_stroke_strip_connects_multiple_samples() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let mut vertices = Vec::new();
        let samples = [
            NprStrokeStripSample {
                point: Vec2::new(-0.5, 0.0),
                width_px: 4.0,
                offset_px: 0.0,
                overshoot_px: 0.0,
                color: ColorRgba::WHITE,
            },
            NprStrokeStripSample {
                point: Vec2::new(0.0, 0.1),
                width_px: 4.0,
                offset_px: 0.0,
                overshoot_px: 0.0,
                color: ColorRgba::WHITE,
            },
            NprStrokeStripSample {
                point: Vec2::new(0.5, 0.0),
                width_px: 4.0,
                offset_px: 0.0,
                overshoot_px: 0.0,
                color: ColorRgba::WHITE,
            },
        ];

        append_npr_stroke_strip_vertices(&mut vertices, &viewport, &samples);

        assert_eq!(vertices.len(), 12);
    }


    #[test]
    fn npr_line_generation_is_deterministic_for_same_seed() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            seed: 2222,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let camera = Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        };
        let mut first = Vec::new();
        let mut second = Vec::new();

        append_mesh_npr_line_vertices(
            &mut first,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );
        append_mesh_npr_line_vertices(
            &mut second,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );

        assert_eq!(vertex_signature(&first), vertex_signature(&second));
    }

    #[test]
    fn npr_backend_stats_report_generated_stroke_work() {
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
        let mut vertices = Vec::new();
        let mut history = BTreeMap::new();
        let stats = append_mesh_npr_line_vertices_with_history_and_stats(
            &mut history,
            1,
            "cube",
            &mut vertices,
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

        assert_eq!(stats.meshes, 1);
        assert!(stats.paths > 0);
        assert_eq!(stats.silhouette_paths, stats.paths);
        assert_eq!(stats.boundary_paths, 0);
        assert_eq!(stats.crease_paths, 0);
        assert_eq!(stats.seam_paths, 0);
        assert_eq!(stats.feature_paths, 0);
        assert!(stats.brush_samples >= stats.paths * 2);
        assert!(stats.primary_passes >= stats.paths);
        assert_eq!(stats.search_passes, 0);
        assert_eq!(stats.cached_plan_hits, 0);
        assert_eq!(stats.cached_plan_misses, stats.paths);
        assert_eq!(stats.strip_vertices, vertices.len());
        assert!(!vertices.is_empty());
    }

    #[test]
    fn npr_line_generation_changes_with_seed_without_dropping_paths() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let camera = Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        };
        let mut first = Vec::new();
        let mut second = Vec::new();

        append_mesh_npr_line_vertices(
            &mut first,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &amigo_render_api::NprLineSettings3d {
                seed: 1001,
                ..amigo_render_api::NprLineSettings3d::default()
            },
        );
        append_mesh_npr_line_vertices(
            &mut second,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &amigo_render_api::NprLineSettings3d {
                seed: 1002,
                ..amigo_render_api::NprLineSettings3d::default()
            },
        );

        assert!(!first.is_empty());
        assert_eq!(first.len(), second.len());
        assert_ne!(vertex_signature(&first), vertex_signature(&second));
    }
