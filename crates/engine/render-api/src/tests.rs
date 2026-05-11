mod tests {
    use super::{
        FrameCompositionPlan, FrameGraph, FrameGraphNodeKind, FrameResourceKind, RenderFeatureId,
        PostFxPassPlan, RenderExtractor, RenderExtractorRegistry, RenderFrameExtractor,
        RenderFrameExtractorRegistry, RenderFramePacket, RenderPassInput, RenderPassOutput,
        RenderPassPlan, World2DPassPlan,
    };

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
            RenderPassPlan::World2D(World2DPassPlan {
                output: RenderPassOutput::WorldColor,
            }),
            RenderPassPlan::PostFx(PostFxPassPlan {
                feature_id: RenderFeatureId::new("lens_droplets"),
                effect_index: 0,
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

        graph.add_node("world_2d", FrameGraphNodeKind::World2D, vec![], vec![world]);
        graph.add_node(
            "present",
            FrameGraphNodeKind::Present,
            vec![world],
            vec![surface],
        );

        assert_eq!(graph.node_labels(), vec!["world_2d", "present"]);
    }

    #[test]
    fn frame_graph_node_kind_has_no_legacy_composite() {
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
}
