use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;
use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuBeacon2dRenderExtractorBridge);
}

pub(super) fn register_installer(bridges: &WgpuRenderExtractorBridgeRegistry) {
    bridges.register(
        amigo_beacon_light_2d_plugin::render::BEACON_2D_EXTRACTOR_ID,
        register,
    );
}

pub struct WgpuBeacon2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuBeacon2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_beacon_light_2d_plugin::render::Beacon2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(beacon_scene_service) =
            runtime.resolve::<amigo_beacon_light_2d_plugin::BeaconLight2dSceneService>()
        else {
            return;
        };
        amigo_beacon_light_2d_plugin::render::Beacon2dRenderExtractor.extract(
            amigo_beacon_light_2d_plugin::render::Beacon2dRenderExtractionContext {
                beacon_scene_service: beacon_scene_service.as_ref(),
            },
            packet,
        );
    }
}
