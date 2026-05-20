use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuVector2dRenderExtractorBridge);
}

pub struct WgpuVector2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuVector2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_vector_2d_plugin::Vector2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(scene_service) = optional::<SceneService>(runtime) else {
            return;
        };
        let Some(vector_scene_service) =
            optional::<amigo_vector_2d_plugin::VectorSceneService>(runtime)
        else {
            return;
        };
        amigo_vector_2d_plugin::Vector2dRenderExtractor.extract(
            amigo_vector_2d_plugin::Vector2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                vector_scene_service: vector_scene_service.as_ref(),
            },
            packet,
        );
    }
}
