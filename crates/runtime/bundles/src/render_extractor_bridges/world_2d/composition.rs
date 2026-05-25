use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use super::common::optional;
use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuComposition2dRenderExtractorBridge);
}

pub(super) fn installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
    crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
        extractor_id: amigo_2d_composition::COMPOSITION_2D_EXTRACTOR_ID,
        install: register,
    }
}

pub struct WgpuComposition2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuComposition2dRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_2d_composition::Composition2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(render_layer2d_scene_service) =
            optional::<amigo_2d_composition::RenderLayer2dSceneService>(runtime)
        else {
            return;
        };
        let Some(light_route2d_scene_service) =
            optional::<amigo_2d_composition::LightRoute2dSceneService>(runtime)
        else {
            return;
        };
        amigo_2d_composition::Composition2dRenderExtractor.extract(
            amigo_2d_composition::Composition2dRenderExtractionContext {
                render_layer2d_scene_service: render_layer2d_scene_service.as_ref(),
                light_route2d_scene_service: light_route2d_scene_service.as_ref(),
            },
            packet,
        );
    }
}
