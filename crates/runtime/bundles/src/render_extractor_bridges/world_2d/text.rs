use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuText2dRenderExtractorBridge);
}

pub struct WgpuText2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuText2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_text_2d_plugin::Text2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(scene_service) = optional::<SceneService>(runtime) else {
            return;
        };
        let Some(text_scene_service) =
            optional::<amigo_text_2d_plugin::Text2dSceneService>(runtime)
        else {
            return;
        };
        amigo_text_2d_plugin::Text2dRenderExtractor.extract(
            amigo_text_2d_plugin::Text2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                text_scene_service: text_scene_service.as_ref(),
            },
            packet,
        );
    }
}
