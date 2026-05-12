use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_text_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedText2dExtractor);
}

pub(crate) struct ResolvedText2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedText2dExtractor {
    fn name(&self) -> &'static str { "resolved_text_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_2d_text::extract_text2d_render_commands(
            amigo_2d_text::Text2dRenderExtractionContext {
                scene_service: context.scene_service,
                text_scene_service: context.text2d_scene_service,
            },
        ) { packet.push_world_2d_text(command); }
    }
}
