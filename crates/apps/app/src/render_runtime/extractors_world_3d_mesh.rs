use amigo_render_api::RenderFrameExtractor;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn register_world_3d_mesh_render_extractors<'a>(
    registry: &mut AppRenderExtractorRegistry<'a>,
) {
    registry.register(ResolvedMesh3dExtractor);
}

pub(crate) struct ResolvedMesh3dExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedMesh3dExtractor {
    fn name(&self) -> &'static str { "resolved_mesh_3d" }
    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in amigo_3d_mesh::extract_mesh3d_render_commands(
            amigo_3d_mesh::Mesh3dRenderExtractionContext {
                scene_service: context.scene_service,
                mesh_scene_service: context.mesh_scene_service,
            },
        ) { packet.push_world_3d_mesh(command); }
    }
}
