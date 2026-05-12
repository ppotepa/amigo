use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_layered_image_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedLayeredImage2dExtractor);
}

pub(crate) struct ResolvedLayeredImage2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedLayeredImage2dExtractor {
    fn name(&self) -> &'static str { "resolved_layered_image_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_2d_layered_image::extract_layered_image2d_render_commands(
            amigo_2d_layered_image::LayeredImage2dRenderExtractionContext {
                scene_service: context.scene_service,
                layered_image_scene_service: context.layered_image_scene_service,
            },
        ) { packet.push_world_2d_layered_image(command); }
    }
}
