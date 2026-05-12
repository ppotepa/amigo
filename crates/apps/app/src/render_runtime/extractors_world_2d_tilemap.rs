use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_tilemap_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedTileMap2dExtractor);
}

pub(crate) struct ResolvedTileMap2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedTileMap2dExtractor {
    fn name(&self) -> &'static str { "resolved_tilemap_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_2d_tilemap::extract_tilemap2d_render_commands(
            amigo_2d_tilemap::TileMap2dRenderExtractionContext {
                scene_service: context.scene_service,
                tilemap_scene_service: context.tilemap_scene_service,
            },
        ) { packet.push_world_2d_tilemap(command); }
    }
}
