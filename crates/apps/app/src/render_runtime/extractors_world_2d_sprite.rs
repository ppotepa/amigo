use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_sprite_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedSprite2dExtractor);
}

pub(crate) struct ResolvedSprite2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedSprite2dExtractor {
    fn name(&self) -> &'static str { "resolved_sprite_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_2d_sprite::extract_sprite2d_render_commands(
            amigo_2d_sprite::Sprite2dRenderExtractionContext {
                scene_service: context.scene_service,
                sprite_scene_service: context.sprite_scene_service,
            },
        ) { packet.push_world_2d_sprite(command); }
    }
}
