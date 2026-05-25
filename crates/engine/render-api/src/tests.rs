mod tests {
use super::{
    FrameCompositionPlan, FrameGraph, FrameGraphNodeKind, FrameResourceKind,
    PostFx2dId, PostFxHost2dId, PostFxPassPlan, PostFxPipelineKind, PostFxScope2d,
    RenderCompositionDiagnostics, RenderFeatureId, RenderExtractor, RenderExtractorRegistry,
    RenderFrameExtractor, RenderFrameExtractorRegistry, RenderFramePacket, RenderPassInput,
    RenderPassOutput, RenderPassPlan, WorldPassPlan,
};

fn sample_post_fx_node() -> FrameGraphNodeKind {
    FrameGraphNodeKind::PostFx {
        host_id: PostFxHost2dId::new("scene:test:visual2d"),
        effect_id: PostFx2dId::new("scene:test:visual2d:0:lens_droplets"),
        scope: PostFxScope2d::Frame,
        pipeline: PostFxPipelineKind::FrameGraph,
        feature_id: RenderFeatureId::new("lens_droplets"),
    }
}

    #[test]
    fn render_frame_packet_defaults_to_empty_overlay() {
        let packet = RenderFramePacket::<u32>::new();

        assert!(packet.overlay().is_empty());
    }

    #[test]
    fn render_frame_packet_preserves_overlay_order() {
        let packet = RenderFramePacket::with_overlay(vec![1_u32, 2, 3]);

        assert_eq!(packet.overlay(), &[1, 2, 3]);
        assert_eq!(packet.into_overlay(), vec![1, 2, 3]);
    }

    #[test]
    fn render_frame_packet_can_accumulate_overlay_items() {
        let mut packet = RenderFramePacket::new();
        packet.push_overlay(1_u32);
        packet.extend_overlay([2_u32, 3]);

        assert_eq!(packet.overlay(), &[1, 2, 3]);
    }

    #[test]
    fn render_extractor_registry_combines_registered_extractors() {
        struct StubExtractor(u32);

        impl RenderExtractor<(), u32> for StubExtractor {
            fn name(&self) -> &'static str {
                "stub"
            }

            fn extract(&self, _context: &(), packet: &mut RenderFramePacket<u32>) {
                packet.push_overlay(self.0);
            }
        }

        let mut registry = RenderExtractorRegistry::default();
        registry.register(StubExtractor(1));
        registry.register(StubExtractor(2));

        let packet = registry.extract_all(&());

        assert_eq!(packet.overlay(), &[1, 2]);
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
    }

    #[test]
    fn render_frame_extractor_registry_combines_registered_extractors() {
        #[derive(Default)]
        struct Packet {
            values: Vec<u32>,
        }

        struct StubExtractor(u32);

        impl RenderFrameExtractor<(), Packet> for StubExtractor {
            fn name(&self) -> &'static str {
                "stub-frame"
            }

            fn extract(&self, _context: &(), packet: &mut Packet) {
                packet.values.push(self.0);
            }
        }

        let mut registry = RenderFrameExtractorRegistry::new();
        registry.register(StubExtractor(7));
        registry.register(StubExtractor(9));

        let packet = registry.extract_all(&());

        assert_eq!(packet.values, vec![7, 9]);
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
    }

    #[test]
    fn composition_plan_detects_post_fx() {
        let plan = FrameCompositionPlan::single_main_view(vec![
            RenderPassPlan::World(WorldPassPlan {
                output: RenderPassOutput::WorldColor,
            }),
            RenderPassPlan::PostFx(PostFxPassPlan {
                host_id: PostFxHost2dId::new("scene:test:visual2d"),
                effect_id: PostFx2dId::new("scene:test:visual2d:0:lens_droplets"),
                scope: PostFxScope2d::Frame,
                pipeline: PostFxPipelineKind::FrameGraph,
                feature_id: RenderFeatureId::new("lens_droplets"),
                input: RenderPassInput::WorldColor,
                output: RenderPassOutput::PostFxColor,
            }),
        ]);

        assert!(plan.has_post_fx());
    }

    #[test]
    fn frame_graph_tracks_nodes_in_order() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
        let world = graph.add_resource(
            "world_color",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );

        graph.add_node("world", FrameGraphNodeKind::World, vec![], vec![world]);
        graph.add_node(
            "present",
            FrameGraphNodeKind::Present,
            vec![world],
            vec![surface],
        );

        assert_eq!(graph.node_labels(), vec!["world", "present"]);
    }

    #[test]
    fn frame_graph_node_kind_has_no_old_composite() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);

        graph.add_node(
            "present",
            FrameGraphNodeKind::Present,
            vec![surface],
            vec![surface],
        );

        assert_eq!(graph.node_labels(), vec!["present"]);
    }

    #[test]
    fn diagnostics_warn_when_non_present_node_writes_surface() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
        let world = graph.add_resource(
            "world_color",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );

        graph.add_node(
            "game_ui",
            FrameGraphNodeKind::GameUi,
            vec![world],
            vec![surface],
        );

        let diagnostics = RenderCompositionDiagnostics::from_plan_and_graph(
            &FrameCompositionPlan::single_main_view(Vec::new()),
            &graph,
        );

        assert!(
            diagnostics
                .warnings
                .iter()
                .any(|warning| warning.contains("non-present node 'game_ui' writes surface resource")),
            "expected non-present surface-write warning, got {:?}",
            diagnostics.warnings
        );
    }

    #[test]
    fn diagnostics_warn_when_postfx_has_no_input() {
        let mut graph = FrameGraph::new();
        let post_fx = graph.add_resource(
            "post_fx_color",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );

        graph.add_node(
            "post_fx:lens_droplets#0",
            sample_post_fx_node(),
            vec![],
            vec![post_fx],
        );

        let diagnostics = RenderCompositionDiagnostics::from_plan_and_graph(
            &FrameCompositionPlan::single_main_view(Vec::new()),
            &graph,
        );

        assert!(
            diagnostics
                .warnings
                .iter()
                .any(|warning| warning.contains("post-fx node 'post_fx:lens_droplets#0' has no reads")),
            "expected post-fx no-reads warning, got {:?}",
            diagnostics.warnings
        );
    }

    #[test]
    fn diagnostics_has_no_warning_when_debug_overlay_follows_postfx() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
        let world = graph.add_resource(
            "world_color",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );
        let post_fx = graph.add_resource(
            "post_fx_color",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );

        graph.add_node(
            "world",
            FrameGraphNodeKind::World,
            vec![],
            vec![world],
        );
        graph.add_node(
            "post_fx:lens_droplets#0",
            sample_post_fx_node(),
            vec![world],
            vec![post_fx],
        );
        graph.add_node(
            "debug_overlay",
            FrameGraphNodeKind::DebugOverlay,
            vec![post_fx],
            vec![post_fx],
        );
        graph.add_node(
            "present",
            FrameGraphNodeKind::Present,
            vec![post_fx],
            vec![surface],
        );

        let diagnostics = RenderCompositionDiagnostics::from_plan_and_graph(
            &FrameCompositionPlan::single_main_view(Vec::new()),
            &graph,
        );

        assert!(
            !diagnostics
                .warnings
                .iter()
                .any(|warning| warning.contains("appears before post-fx")),
            "unexpected order warning, got {:?}",
            diagnostics.warnings
        );
        assert!(
            !diagnostics
                .warnings
                .iter()
                .any(|warning| warning.contains("appears after debug-overlay")),
            "unexpected post-fx order warning, got {:?}",
            diagnostics.warnings
        );
    }

    #[test]
    fn camera_optics_render_descriptor_declares_visual_source_inputs() {
        let descriptor = super::post_fx_camera_optics(super::CameraOptics2d::default())
            .render_descriptor();

        assert_eq!(descriptor.feature_id, "camera_optics");
        assert_eq!(descriptor.executor_id, "screen_space.camera_optics");
        assert_eq!(
            descriptor.required_inputs,
            &[
                super::PostFxRenderInput::SourceColor,
                super::PostFxRenderInput::SceneNormal,
                super::PostFxRenderInput::SceneWetness,
                super::PostFxRenderInput::SceneHighlight,
                super::PostFxRenderInput::SceneEmissive,
            ]
        );
        assert_eq!(
            descriptor.output,
            super::PostFxRenderOutput::CompositeVisualSourceResponse
        );
        assert_eq!(descriptor.debug_policy.camera_debug_rank, Some(30));
    }

    #[test]
    fn focus_blur_render_descriptor_marks_depth_debug_and_layer_replay() {
        let descriptor = super::post_fx_focus_blur(super::FocusBlur2d::default()).render_descriptor();

        assert_eq!(descriptor.feature_id, "focus_blur");
        assert_eq!(
            descriptor.required_inputs,
            &[
                super::PostFxRenderInput::SourceColor,
                super::PostFxRenderInput::FrameRequest,
            ]
        );
        assert_eq!(descriptor.output, super::PostFxRenderOutput::ReplayScopedLayers);
        assert!(descriptor.debug_policy.supports_depth_debug_view);
        assert_eq!(descriptor.debug_policy.camera_debug_rank, Some(40));
    }

    #[test]
    fn cached_image_policy_distinguishes_blur_emboss_and_passthrough_effects() {
        let blur = super::post_fx_blur(super::PostFxBlur2d::default()).render_descriptor();
        let emboss =
            super::post_fx_emboss_edges(super::PostFxEmbossEdges2d::default()).render_descriptor();
        let rain_glass =
            super::post_fx_rain_glass(super::RainGlass2d::default()).render_descriptor();

        assert_eq!(
            blur.cached_image_policy,
            super::PostFxCachedImagePolicy::RasterEffectWithBoundsExpansion
        );
        assert_eq!(
            emboss.cached_image_policy,
            super::PostFxCachedImagePolicy::RasterEffect
        );
        assert_eq!(
            rain_glass.cached_image_policy,
            super::PostFxCachedImagePolicy::PassthroughCopy
        );
    }

    #[test]
    fn render_descriptor_lookup_by_kind_matches_effect_descriptor() {
        let from_effect =
            super::post_fx_shutter_blur(super::ShutterBlur2d::default()).render_descriptor();
        let from_kind = super::PostFxRenderDescriptor::for_kind("shutter_blur")
            .expect("known effect kind should resolve");

        assert_eq!(from_effect, from_kind);
        assert_eq!(
            from_kind.required_inputs,
            &[
                super::PostFxRenderInput::SourceColor,
                super::PostFxRenderInput::HostEffectIdentity,
            ]
        );
        assert_eq!(from_kind.debug_policy.camera_debug_rank, Some(20));
    }

    #[test]
    fn render_descriptor_lookup_returns_none_for_unknown_kind() {
        assert_eq!(super::PostFxRenderDescriptor::for_kind("unknown_post_fx"), None);
    }

    #[test]
    fn render_descriptor_registry_is_unique_and_self_consistent() {
        let mut seen = std::collections::BTreeSet::new();

        for entry in super::PostFxRenderDescriptor::registry() {
            assert!(
                seen.insert(entry.kind),
                "duplicate render descriptor kind {}",
                entry.kind
            );
            assert_eq!(entry.kind, entry.descriptor.feature_id);
            assert_eq!(
                super::PostFxRenderDescriptor::for_kind(entry.kind),
                Some(entry.descriptor)
            );
        }
    }

    #[test]
    fn every_effect_variant_has_an_explicit_render_descriptor() {
        let effects = [
            super::post_fx_blur(super::PostFxBlur2d::default()),
            super::post_fx_camera_exposure(super::CameraExposure2d::default()),
            super::post_fx_camera_optics(super::CameraOptics2d::default()),
            super::post_fx_color_quantize(super::ColorQuantize2d::default()),
            super::post_fx_color_ramp(super::ColorRamp2d::default()),
            super::post_fx_crt(super::Crt2d::default()),
            super::post_fx_downscale(super::Downscale2d::default()),
            super::post_fx_dirty_bloom(super::DirtyBloom2d::default()),
            super::post_fx_emboss_edges(super::PostFxEmbossEdges2d::default()),
            super::post_fx_film_emulsion(super::FilmEmulsion2d::default()),
            super::post_fx_film_noise(super::FilmNoise2d::default()),
            super::post_fx_focus_blur(super::FocusBlur2d::default()),
            super::post_fx_lens_droplets(super::PostFxLensDroplets2d::default()),
            super::post_fx_rain_glass(super::RainGlass2d::default()),
            super::post_fx_scan_output(super::ScanOutput2d::default()),
            super::post_fx_shutter_blur(super::ShutterBlur2d::default()),
            super::post_fx_wet_reflections(super::PostFxWetReflections2d::default()),
        ];

        let effect_kinds = effects
            .iter()
            .map(|effect| effect.kind())
            .collect::<std::collections::BTreeSet<_>>();
        let descriptor_kinds = super::PostFxRenderDescriptor::registry()
            .iter()
            .map(|entry| entry.kind)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(descriptor_kinds, effect_kinds);

        for effect in &effects {
            let descriptor = effect.render_descriptor();

            assert_eq!(descriptor.feature_id, effect.kind());
            assert!(
                super::PostFxRenderDescriptor::for_kind(effect.kind()).is_some(),
                "missing render descriptor for {}",
                effect.kind()
            );
            assert!(
                descriptor.requires_executor(),
                "missing executor id for {}",
                effect.kind()
            );
            assert!(
                descriptor
                    .required_inputs
                    .contains(&super::PostFxRenderInput::SourceColor),
                "missing source color input for {}",
                effect.kind()
            );
        }
    }

    #[test]
    fn pipeline_helpers_follow_render_descriptor_policy() {
        let blur = super::post_fx_blur(super::PostFxBlur2d::default());
        let emboss = super::post_fx_emboss_edges(super::PostFxEmbossEdges2d::default());
        let camera_optics = super::post_fx_camera_optics(super::CameraOptics2d::default());

        assert!(blur.uses_cached_image_pipeline());
        assert!(emboss.uses_cached_image_pipeline());
        assert!(!camera_optics.uses_cached_image_pipeline());

        assert!(!blur.uses_frame_graph_pipeline());
        assert!(!emboss.uses_frame_graph_pipeline());
        assert!(camera_optics.uses_frame_graph_pipeline());
    }
}

