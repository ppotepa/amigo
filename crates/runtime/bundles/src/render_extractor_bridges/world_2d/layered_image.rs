use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuLayeredImage2dRenderExtractorBridge);
}

pub struct WgpuLayeredImage2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket>
    for WgpuLayeredImage2dRenderExtractorBridge
{
    fn name(&self) -> &'static str {
        amigo_layered_image_2d_plugin::LayeredImage2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(scene_service) = optional::<SceneService>(runtime) else {
            return;
        };
        let Some(layered_image_scene_service) =
            optional::<amigo_layered_image_2d_plugin::LayeredImageSceneService>(runtime)
        else {
            return;
        };
        amigo_layered_image_2d_plugin::LayeredImage2dRenderExtractor.extract(
            amigo_layered_image_2d_plugin::LayeredImage2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                layered_image_scene_service: layered_image_scene_service.as_ref(),
            },
            packet,
        );
    }
}
