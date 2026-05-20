use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuSprite2dRenderExtractorBridge);
}

pub struct WgpuSprite2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuSprite2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_sprite_2d_plugin::Sprite2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(scene_service) = optional::<SceneService>(runtime) else {
            return;
        };
        let Some(sprite_scene_service) =
            optional::<amigo_sprite_2d_plugin::SpriteSceneService>(runtime)
        else {
            return;
        };
        amigo_sprite_2d_plugin::Sprite2dRenderExtractor.extract(
            amigo_sprite_2d_plugin::Sprite2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                sprite_scene_service: sprite_scene_service.as_ref(),
            },
            packet,
        );
    }
}
