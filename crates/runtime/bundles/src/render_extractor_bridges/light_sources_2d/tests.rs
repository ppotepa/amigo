mod tests {
    use amigo_camera_optics_plugin::api::{
        CameraOpticalCandidateStatus2d, CameraOpticalResponse2d,
    };
    use amigo_render_api::{VisualSourceKind2d, VisualSourceOrigin2d, VisualSourceRef2d};

    use super::super::{
        collect_camera_optical_candidates_from_light_sources_2d, collect_light_sources_2d,
    };
    use crate::render_extractor_bridges::light_sources_2d::format_light_sources_2d;

    #[test]
    fn light_sources_summary_reports_emissive_visual_source() {
        let sources = collect_light_sources_2d(
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            Some(&amigo_render_api::CameraCaptureInput2d {
                depth_space: amigo_2d_spatial::DepthSpace2d::default(),
                color: VisualSourceRef2d::fallback(VisualSourceKind2d::SceneColor, "scene"),
                depth: None,
                layer_mask: None,
                normal: None,
                wetness: None,
                emissive: Some(VisualSourceRef2d::produced(
                    VisualSourceKind2d::SceneEmissive,
                    "scene_emissive",
                    VisualSourceOrigin2d::EmissiveBuffer,
                )),
                highlight: None,
                motion: None,
                layers: Vec::new(),
            }),
        );
        let summary = format_light_sources_2d(&sources);

        assert!(summary.contains("render.light.sources:"));
        assert!(summary.contains("emissive_visual_source"));
        assert!(summary.contains("scene_emissive"));
    }

    #[test]
    fn light_sources_collects_lightmap_channels() {
        let lightmap = amigo_light_2d_plugin::LightMap2dSourceCommand {
            source_mod: "test".to_owned(),
            entity_name: "neon-map".to_owned(),
            id: "neon-alley-lightmap".to_owned(),
            source: amigo_light_2d_plugin::LightMap2dSourceRef {
                kind: amigo_light_2d_plugin::LightMap2dSourceKind::LayeredImage2d,
                entity_name: "neon-map".to_owned(),
            },
            channels: vec![amigo_light_2d_plugin::LightMap2dChannel {
                id: "mid_neon".to_owned(),
                layers: vec!["club.mid".to_owned()],
            }],
        };

        let sources = collect_light_sources_2d(&[], &[], &[], &[lightmap], &[], &[], None);
        assert_eq!(sources.len(), 1);
        assert_eq!(
            sources[0].emitter_kind,
            amigo_render_api::LightEmitterKind2d::LightMapChannel
        );
        assert_eq!(sources[0].effective_intensity, Some(1.0));
        assert!(sources[0].reason.contains("mid_neon"));
    }

    #[test]
    fn light_sources_resolves_light_group_effective_intensity() {
        let global = amigo_light_2d_plugin::GlobalLight2dCommand {
            source_mod: "test".to_owned(),
            entity_name: "sky-light".to_owned(),
            id: "sky".to_owned(),
            color: amigo_math::ColorRgba::new(1.0, 0.9, 0.8, 1.0),
            intensity: 0.5,
        };
        let group = amigo_light_2d_plugin::LightGroup2dCommand {
            source_mod: "test".to_owned(),
            id: "street-neon".to_owned(),
            label: None,
            color: amigo_math::ColorRgba::new(1.0, 0.8, 0.6, 1.0),
            intensity: 2.0,
            render_contributions: amigo_render_api::RenderContributionSet::default(),
            camera_response: CameraOpticalResponse2d::default(),
            sources: vec![amigo_light_2d_plugin::LightGroup2dSourceCommand {
                kind: amigo_light_2d_plugin::LightGroup2dSourceKind::GlobalLight {
                    id: "sky".to_owned(),
                },
                response: 0.25,
            }],
        };

        let sources = collect_light_sources_2d(&[], &[], &[global], &[], &[group], &[], None);
        let group_source = sources
            .iter()
            .find(|source| source.emitter_kind == amigo_render_api::LightEmitterKind2d::LightGroup)
            .expect("light group source should be collected");
        assert_eq!(group_source.effective_intensity, Some(0.25));
        assert!(group_source.reason.contains("street-neon"));
    }

    #[test]
    fn camera_optical_candidates_report_light_group_lightmap_coverage() {
        let lightmap = amigo_light_2d_plugin::LightMap2dSourceCommand {
            source_mod: "test".to_owned(),
            entity_name: "neon-map".to_owned(),
            id: "neon-alley-lightmap".to_owned(),
            source: amigo_light_2d_plugin::LightMap2dSourceRef {
                kind: amigo_light_2d_plugin::LightMap2dSourceKind::LayeredImage2d,
                entity_name: "neon-map".to_owned(),
            },
            channels: vec![amigo_light_2d_plugin::LightMap2dChannel {
                id: "mid_neon".to_owned(),
                layers: vec!["club.mid".to_owned()],
            }],
        };
        let group = amigo_light_2d_plugin::LightGroup2dCommand {
            source_mod: "test".to_owned(),
            id: "neon.mid".to_owned(),
            label: None,
            color: amigo_math::ColorRgba::new(1.0, 0.2, 0.8, 1.0),
            intensity: 1.5,
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([
                (
                    amigo_render_api::render_contribution_roles::LIGHTING_EMIT,
                    true,
                ),
                (
                    amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
                    true,
                ),
                (
                    amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
                    true,
                ),
            ]),
            camera_response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 0.75,
                bloom: 0.45,
                ghosting: 0.22,
                ..CameraOpticalResponse2d::default()
            },
            sources: vec![amigo_light_2d_plugin::LightGroup2dSourceCommand {
                kind: amigo_light_2d_plugin::LightGroup2dSourceKind::LightMapChannel {
                    source: "neon-alley-lightmap".to_owned(),
                    channel: "mid_neon".to_owned(),
                },
                response: 1.0,
            }],
        };

        let light_sources =
            collect_light_sources_2d(&[], &[], &[], &[lightmap], &[group], &[], None);
        let candidates = collect_camera_optical_candidates_from_light_sources_2d(&light_sources);
        let summary = amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
            &candidates,
        );

        assert!(summary.contains("camera.optical.candidates:"));
        assert!(summary.contains("coverage=lightmap_channel"));
        assert!(summary.contains("source=neon-alley-lightmap"));
        assert!(summary.contains("channel=mid_neon"));
        assert!(summary.contains("intensity=1.500"));
        assert!(summary.contains("bloom:0.450"));
        assert!(summary.contains("status=active"));
        assert!(summary.contains("ghosting:0.220"));
        assert!(summary.contains("targets=scene_highlight,scene_emissive"));
        assert!(summary.contains("highlight_gain=1.125"));
        assert!(summary.contains("emissive_gain=1.125"));
    }

    #[test]
    fn camera_optical_candidate_unsupported_coverage_is_skipped() {
        let source = amigo_render_api::LightSource2dCommon {
            owner: "global-backed-group".to_owned(),
            component_kind: "LightGroup2D".to_owned(),
            emitter_kind: amigo_render_api::LightEmitterKind2d::LightGroup,
            emitter_id: Some("light_group:global:sky".to_owned()),
            render_layer: None,
            color_rgba: Some([1.0, 1.0, 1.0, 1.0]),
            intensity: Some(1.0),
            effective_intensity: Some(1.0),
            response: Some(1.0),
            camera_response: Some(CameraOpticalResponse2d {
                enabled: true,
                intensity: 1.0,
                glare: 1.0,
                bloom: 1.0,
                ..CameraOpticalResponse2d::default()
            }),
            bloom: None,

            radius_px: None,
            falloff: None,
            distance_m: None,
            z_depth: None,
            contributions: vec![
                amigo_render_api::LightContributionKind2d::CameraFxSource,
                amigo_render_api::LightContributionKind2d::BloomSource,
            ],
            status: amigo_render_api::LightSourceStatus2d::Active,
            reason: "light_group_active".to_owned(),
            position_px: None,
        };

        let candidates = collect_camera_optical_candidates_from_light_sources_2d(&[source]);
        let summary = amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
            &candidates,
        );

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].status,
            CameraOpticalCandidateStatus2d::Skipped
        );
        assert!(!candidates[0].is_active());
        assert_eq!(candidates[0].highlight_gain(), 0.0);
        assert_eq!(candidates[0].emissive_gain(), 0.0);
        assert!(summary.contains("status=skipped"));
        assert!(summary.contains("reason=camera_optical_coverage_unsupported"));
        assert!(summary.contains("targets="));
        assert!(summary.contains("highlight_gain=0.000"));
        assert!(summary.contains("emissive_gain=0.000"));
    }

    #[test]
    fn camera_optical_candidates_report_beacon_hotspot_coverage() {
        let beacon = amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand {
            entity_name: "beacon-a".to_owned(),
            render_layer: "foreground.lights".to_owned(),
            z_index: 0.0,
            center: amigo_math::Vec2::new(42.0, 84.0),
            color: amigo_math::ColorRgba::new(1.0, 0.2, 0.1, 1.0),
            intensity: 1.0,
            pulse: 1.0,
            core_radius_px: 8.0,
            halo_radius_px: 42.0,
            glow_strength: 0.6,
            rotation_radians: 0.0,
            beam_enabled: false,
            beam_length_px: 0.0,
            beam_width_degrees: 0.0,
            beam_strength: 0.0,
            aberration_px: 4.0,

            bloom: 0.5,
            camera_response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 0.8,
                bloom: 0.5,
                glare: 0.8,
                ghosting: 0.7,
                streaks: 0.18,
                chromatic_smear: 4.0 / 32.0,
                dirt_response: 0.4,
                halation: 0.5 * 0.35,
                threshold: 0.0,
            },
            distance_m: Some(2.0),
            z_depth: Some(0.75),
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([
                (
                    amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
                    true,
                ),
                (
                    amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
                    true,
                ),
            ]),
            viewport_fit: amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed,
            viewport_canvas_size: None,
        };

        let light_sources = collect_light_sources_2d(&[], &[beacon], &[], &[], &[], &[], None);
        let candidates = collect_camera_optical_candidates_from_light_sources_2d(&light_sources);
        let summary = amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
            &candidates,
        );

        assert!(summary.contains("component=BeaconLight2D"));
        assert!(summary.contains("coverage=hotspot"));
        assert!(summary.contains("entity=beacon-a"));
        assert!(summary.contains("radius_px=42.000"));
        assert!(summary.contains("position_px=42.000,84.000"));
        assert!(summary.contains("status=active"));
        assert!(summary.contains("ghosting:0.700"));
        assert!(summary.contains("streaks:0.180"));
        assert!(summary.contains("chromatic_smear:0.125"));
        assert!(summary.contains("dirt:0.400"));
    }
}
