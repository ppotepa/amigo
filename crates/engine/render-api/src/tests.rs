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
}

