use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;
use crate::render_extractor_registry::WgpuRenderExtractorBridgeRegistry;

use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuSprite2dRenderExtractorBridge);
}

pub(super) fn register_installer(bridges: &WgpuRenderExtractorBridgeRegistry) {
    bridges.register(
        amigo_sprite_2d_plugin::render::SPRITE_2D_EXTRACTOR_ID,
        register,
    );
}

pub struct WgpuSprite2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuSprite2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_sprite_2d_plugin::render::Sprite2dRenderExtractor.name()
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
        for command in amigo_sprite_2d_plugin::render::extract_sprite2d_render_commands(
            amigo_sprite_2d_plugin::render::Sprite2dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                sprite_scene_service: sprite_scene_service.as_ref(),
            },
        ) {
            packet.push_renderable_2d(
                amigo_sprite_2d_plugin::render::sprite_draw_command_to_renderable_2d(&command),
            );
        }
    }
}
