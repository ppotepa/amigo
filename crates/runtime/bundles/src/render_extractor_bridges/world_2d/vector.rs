use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use super::common::optional;
use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuVector2dRenderExtractorBridge);
}

pub(super) fn installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
    crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
        extractor_id: amigo_vector_2d_plugin::render::VECTOR_2D_EXTRACTOR_ID,
        install: register,
    }
}

pub struct WgpuVector2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuVector2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_vector_2d_plugin::render::Vector2dRenderExtractor.name()
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
        for command in amigo_vector_2d_plugin::render::extract_vector2d_render_commands(
            amigo_vector_2d_plugin::render::Vector2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                vector_scene_service: vector_scene_service.as_ref(),
            },
        ) {
            packet.push_renderable_2d(
                amigo_vector_2d_plugin::render::vector_draw_command_to_renderable_2d(&command),
            );
        }
    }
}
