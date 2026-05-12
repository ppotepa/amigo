use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_composition_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedComposition2dExtractor);
}

pub(crate) struct ResolvedComposition2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedComposition2dExtractor {
    fn name(&self) -> &'static str { "resolved_composition_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        let commands = amigo_2d_composition::extract_composition2d_render_commands(
            amigo_2d_composition::Composition2dRenderExtractionContext {
                render_layer2d_scene_service: context.render_layer2d_scene_service,
                light_route2d_scene_service: context.light_route2d_scene_service,
            },
        );
        for command in commands.render_layers { packet.push_world_2d_render_layer(command); }
        for command in commands.light_routes { packet.push_world_2d_light_route(command); }
    }
}
