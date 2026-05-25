use super::*;

mod emergency_overlay_tests {
    use super::*;

    #[test]
    fn emergency_overlay_lines_keep_latest_five() {
        let mut renderer_lines = Vec::new();
        for index in 0..8 {
            emergency_overlay::push_emergency_overlay_line(
                &mut renderer_lines,
                WgpuEmergencyOverlayLevel::Error,
                format!("error {index}"),
            );
        }

        let lines = emergency_overlay::emergency_overlay_lines(&[], &renderer_lines);
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0].message, "error 3");
        assert_eq!(lines[4].message, "error 7");
    }

    #[test]
    fn emergency_overlay_deduplicates_adjacent_messages() {
        let mut lines = Vec::new();
        emergency_overlay::push_emergency_overlay_line(
            &mut lines,
            WgpuEmergencyOverlayLevel::Warning,
            "same warning".to_owned(),
        );
        emergency_overlay::push_emergency_overlay_line(
            &mut lines,
            WgpuEmergencyOverlayLevel::Warning,
            "same warning".to_owned(),
        );

        assert_eq!(lines.len(), 1);
    }
}

mod focus_blur_layer_plan_tests {
    use super::*;

    fn layer(id: &str, mode: amigo_render_api::RenderDepthMode2d) -> RenderLayer2dCommand {
        RenderLayer2dCommand {
            source_mod: "test-mod".to_owned(),
            id: id.to_owned(),
            label: Some(id.to_owned()),
            order: 0.0,
            visible: true,
            opacity: 1.0,
            depth: amigo_render_api::RenderDepth2d {
                mode,
                distance_m: None,
                z_depth: 0.5,
                blur_scale: 1.0,
            },
            optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
        }
    }

    #[test]
    fn focus_blur_plan_prefers_explicit_render_depth_over_implicit_affected_layers() {
        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d {
                affected_layers: vec!["background.city".to_owned()],
                ..Default::default()
            },
            &[
                layer(
                    "background.city",
                    amigo_render_api::RenderDepthMode2d::DepthMap,
                ),
                layer(
                    "weather.rain.front",
                    amigo_render_api::RenderDepthMode2d::Overlay,
                ),
            ],
            None,
        );

        assert!(plan.has_explicit_render_depth);
        assert!(plan.implicit_affected_layers.is_none());
        assert!(plan.overlay_layers.contains("weather.rain.front"));
    }

    #[test]
    fn focus_blur_plan_splits_depth_map_z_depth_and_overlay_layers() {
        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d::default(),
            &[
                layer(
                    "background.city",
                    amigo_render_api::RenderDepthMode2d::DepthMap,
                ),
                RenderLayer2dCommand {
                    depth: amigo_render_api::RenderDepth2d {
                        mode: amigo_render_api::RenderDepthMode2d::ZDepth,
                        distance_m: None,
                        z_depth: 0.34,
                        blur_scale: 0.12,
                    },
                    ..layer(
                        "weather.rain.near",
                        amigo_render_api::RenderDepthMode2d::ZDepth,
                    )
                },
                layer("ui", amigo_render_api::RenderDepthMode2d::Overlay),
            ],
            None,
        );

        assert!(plan.depth_map_layers.contains("background.city"));
        assert_eq!(plan.z_depth_layers.len(), 1);
        assert_eq!(plan.z_depth_layers[0].layer_id, "weather.rain.near");
        assert_eq!(plan.z_depth_layers[0].z_depth, 0.34);
        assert_eq!(plan.z_depth_layers[0].blur_scale, 0.12);
        assert!(plan.overlay_layers.contains("ui"));
    }

    #[test]
    fn focus_blur_plan_preserves_z_depth_layer_order() {
        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d::default(),
            &[
                RenderLayer2dCommand {
                    order: -18.0,
                    depth: amigo_render_api::RenderDepth2d {
                        mode: amigo_render_api::RenderDepthMode2d::ZDepth,
                        distance_m: None,
                        z_depth: 0.68,
                        blur_scale: 0.45,
                    },
                    ..layer(
                        "weather.rain.far",
                        amigo_render_api::RenderDepthMode2d::ZDepth,
                    )
                },
                RenderLayer2dCommand {
                    order: -16.0,
                    depth: amigo_render_api::RenderDepth2d {
                        mode: amigo_render_api::RenderDepthMode2d::ZDepth,
                        distance_m: None,
                        z_depth: 0.52,
                        blur_scale: 0.25,
                    },
                    ..layer(
                        "weather.rain.mid",
                        amigo_render_api::RenderDepthMode2d::ZDepth,
                    )
                },
                RenderLayer2dCommand {
                    order: -14.0,
                    depth: amigo_render_api::RenderDepth2d {
                        mode: amigo_render_api::RenderDepthMode2d::ZDepth,
                        distance_m: None,
                        z_depth: 0.34,
                        blur_scale: 0.12,
                    },
                    ..layer(
                        "weather.rain.near",
                        amigo_render_api::RenderDepthMode2d::ZDepth,
                    )
                },
            ],
            None,
        );

        assert_eq!(
            plan.z_depth_layers
                .iter()
                .map(|layer| layer.layer_id.as_str())
                .collect::<Vec<_>>(),
            vec!["weather.rain.far", "weather.rain.mid", "weather.rain.near"]
        );
    }

    #[test]
    fn focus_blur_plan_treats_distance_layers_as_constant_z_depth_layers() {
        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d::default(),
            &[RenderLayer2dCommand {
                depth: amigo_render_api::RenderDepth2d {
                    mode: amigo_render_api::RenderDepthMode2d::Distance,
                    distance_m: Some(75.0),
                    z_depth: 0.41,
                    blur_scale: 0.25,
                },
                ..layer(
                    "weather.rain.mid",
                    amigo_render_api::RenderDepthMode2d::Distance,
                )
            }],
            None,
        );

        assert_eq!(plan.z_depth_layers.len(), 1);
        assert_eq!(plan.z_depth_layers[0].layer_id, "weather.rain.mid");
        assert_eq!(plan.z_depth_layers[0].z_depth, 0.41);
    }

    #[test]
    fn focus_blur_plan_treats_infinity_as_far_z_depth() {
        let plan = build_focus_blur_layer_plan(
            amigo_render_api::FocusBlur2d::default(),
            &[layer("sky", amigo_render_api::RenderDepthMode2d::Infinity)],
            None,
        );
        assert_eq!(plan.z_depth_layers.len(), 1);
        assert_eq!(plan.z_depth_layers[0].z_depth, 0.0);
    }
}

mod camera_debug_view_tests {
    use super::visual_debug::{
        optical_debug_missing_color, optical_role_debug_color, should_bypass_for_camera_debug_view,
    };

    #[test]
    fn camera_after_dof_bypasses_later_camera_effects() {
        assert!(should_bypass_for_camera_debug_view(
            &amigo_render_api::CameraDebugView2d::parse("camera_after_dof"),
            "film_emulsion"
        ));
        assert!(should_bypass_for_camera_debug_view(
            &amigo_render_api::CameraDebugView2d::parse("camera_after_dof"),
            "color_ramp"
        ));
        assert!(!should_bypass_for_camera_debug_view(
            &amigo_render_api::CameraDebugView2d::parse("camera_after_dof"),
            "focus_blur"
        ));
    }

    #[test]
    fn camera_after_optics_bypasses_dof_and_later_effects() {
        assert!(should_bypass_for_camera_debug_view(
            &amigo_render_api::CameraDebugView2d::parse("camera_after_optics"),
            "focus_blur"
        ));
        assert!(should_bypass_for_camera_debug_view(
            &amigo_render_api::CameraDebugView2d::parse("camera_after_optics"),
            "film_emulsion"
        ));
        assert!(!should_bypass_for_camera_debug_view(
            &amigo_render_api::CameraDebugView2d::parse("camera_after_optics"),
            "camera_optics"
        ));
    }

    #[test]
    fn optical_role_debug_colors_are_distinct() {
        assert_ne!(
            optical_role_debug_color(amigo_2d_spatial::OpticalLayerRole2d::WorldSurface).r,
            optical_role_debug_color(amigo_2d_spatial::OpticalLayerRole2d::Overlay).r
        );
    }

    #[test]
    fn missing_source_debug_colors_are_non_black_for_real_missing_kinds() {
        assert_ne!(
            optical_debug_missing_color(amigo_render_api::VisualSourceKind2d::SceneNormal).b,
            0.0
        );
        assert_ne!(
            optical_debug_missing_color(amigo_render_api::VisualSourceKind2d::SceneWetness).g,
            0.0
        );
        assert_ne!(
            optical_debug_missing_color(amigo_render_api::VisualSourceKind2d::SceneEmissive).r,
            0.0
        );
        assert_ne!(
            optical_debug_missing_color(amigo_render_api::VisualSourceKind2d::LayerMask).r,
            0.0
        );
    }
}
