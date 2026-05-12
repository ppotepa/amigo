use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};
use super::extractors_world_3d_material;
use super::extractors_world_3d_mesh;

pub(crate) fn register_world_3d_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    extractors_world_3d_mesh::register_world_3d_mesh_render_extractors(registry);
    extractors_world_3d_material::register_world_3d_material_render_extractors(registry);
    registry.register(ResolvedText3dExtractor);
}

pub(crate) struct ResolvedText3dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedText3dExtractor {
    fn name(&self) -> &'static str { "resolved_text_3d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_3d_text::extract_text3d_render_commands(
            amigo_3d_text::Text3dRenderExtractionContext {
                scene_service: context.scene_service,
                text3d_scene_service: context.text3d_scene_service,
            },
        ) { packet.push_world_3d_text(command); }
    }
}
