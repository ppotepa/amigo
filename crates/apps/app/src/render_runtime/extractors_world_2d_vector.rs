use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_vector_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedVector2dExtractor);
}

pub(crate) struct ResolvedVector2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedVector2dExtractor {
    fn name(&self) -> &'static str { "resolved_vector_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_2d_vector::extract_vector2d_render_commands(
            amigo_2d_vector::Vector2dRenderExtractionContext {
                scene_service: context.scene_service,
                vector_scene_service: context.vector_scene_service,
            },
        ) { packet.push_world_2d_vector(command); }
    }
}
