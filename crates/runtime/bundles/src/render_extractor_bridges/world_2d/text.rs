use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;
use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuText2dRenderExtractorBridge);
}

pub(super) fn register_installer(bridges: &WgpuRenderExtractorBridgeRegistry) {
    bridges.register(amigo_text_2d_plugin::render::TEXT_2D_EXTRACTOR_ID, register);
}

pub struct WgpuText2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuText2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_text_2d_plugin::render::Text2dRenderExtractor.name()
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
        for command in amigo_text_2d_plugin::render::extract_text2d_render_commands(
            amigo_text_2d_plugin::render::Text2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                text_scene_service: text_scene_service.as_ref(),
            },
        ) {
            packet.push_renderable_2d(
                amigo_text_2d_plugin::render::text_draw_command_to_renderable_2d(&command),
            );
        }
    }
}
