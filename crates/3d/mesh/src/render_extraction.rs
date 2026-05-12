use amigo_scene::SceneService;

use crate::{MeshDrawCommand, MeshSceneService};

pub struct Mesh3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub mesh_scene_service: &'a MeshSceneService,
}

pub fn extract_mesh3d_render_commands(
    ctx: Mesh3dRenderExtractionContext<'_>,
) -> Vec<MeshDrawCommand> {
    ctx.mesh_scene_service
        .commands()
        .into_iter()
        .filter(|command| is_entity_render_visible(ctx.scene_service, &command.entity_name))
        .collect()
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}
