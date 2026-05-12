use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_3d_material_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedMaterial3dExtractor);
}

pub(crate) struct ResolvedMaterial3dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedMaterial3dExtractor {
    fn name(&self) -> &'static str { "resolved_material_3d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_3d_material::extract_material3d_render_commands(
            amigo_3d_material::Material3dRenderExtractionContext {
                scene_service: context.scene_service,
                material_scene_service: context.material_scene_service,
            },
        ) { packet.push_world_3d_material(command); }
    }
}
