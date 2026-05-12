use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_postfx_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedPostFx2dExtractor);
}

pub(crate) struct ResolvedPostFx2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedPostFx2dExtractor {
    fn name(&self) -> &'static str { "resolved_post_fx_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        if let Some(stack) = amigo_2d_post_fx::extract_post_fx2d_render_stack(
            amigo_2d_post_fx::PostFx2dRenderExtractionContext { post_fx_service: context.post_fx_service },
        ) { packet.set_post_fx_stack(stack); }
    }
}
