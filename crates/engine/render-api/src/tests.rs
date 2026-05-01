mod tests {
    use super::{
        RenderExtractor, RenderExtractorRegistry, RenderFrameExtractor,
        RenderFrameExtractorRegistry, RenderFramePacket,
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
}
