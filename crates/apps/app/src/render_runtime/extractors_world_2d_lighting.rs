use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_2d_lighting_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedLighting2dExtractor);
}

pub(crate) struct ResolvedLighting2dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedLighting2dExtractor {
    fn name(&self) -> &'static str { "resolved_lighting_2d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        let commands = amigo_2d_lighting::extract_lighting2d_render_commands(
            amigo_2d_lighting::Lighting2dRenderExtractionContext {
                global_light2d_scene_service: context.global_light2d_scene_service,
                lightmap2d_scene_service: context.lightmap2d_scene_service,
                light_group2d_scene_service: context.light_group2d_scene_service,
            },
        );
        for command in commands.global_lights { packet.push_world_2d_global_light(command); }
        for command in commands.lightmaps { packet.push_world_2d_lightmap(command); }
        for command in commands.light_groups { packet.push_world_2d_light_group(command); }
    }
}
