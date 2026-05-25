use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use super::common::optional;
use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuLayeredImage2dRenderExtractorBridge);
}

pub(super) fn installer() -> crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
    crate::render_extractor_registry::WgpuRenderExtractorBridgeInstaller {
        extractor_id: amigo_layered_image_2d_plugin::render::LAYERED_IMAGE_2D_EXTRACTOR_ID,
        install: register,
    }
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
        for command in amigo_layered_image_2d_plugin::extract_layered_image2d_render_commands(
            amigo_layered_image_2d_plugin::LayeredImage2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                layered_image_scene_service: layered_image_scene_service.as_ref(),
            },
        ) {
            packet.push_renderable_2d(
                amigo_layered_image_2d_plugin::render::layered_image_draw_command_to_renderable_2d(
                    &command,
                ),
            );
        }
    }
}
