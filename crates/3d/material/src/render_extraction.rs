use amigo_scene::SceneService;

use crate::{MaterialDrawCommand, MaterialSceneService};

pub struct Material3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub material_scene_service: &'a MaterialSceneService,
}

pub fn extract_material3d_render_commands(
    ctx: Material3dRenderExtractionContext<'_>,
) -> Vec<MaterialDrawCommand> {
    ctx.material_scene_service
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
