use amigo_scene::SceneService;

use crate::{MaterialDrawCommand, MaterialSceneService};

pub struct Material3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub material_scene_service: &'a MaterialSceneService,
}

pub trait Material3dRenderOutput {
    fn push_material3d_render_command(&mut self, command: MaterialDrawCommand);
}

pub struct Material3dRenderExtractor;

impl Material3dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "material_3d"
    }

    pub fn extract(
        &self,
        ctx: Material3dRenderExtractionContext<'_>,
        output: &mut impl Material3dRenderOutput,
    ) {
        for command in extract_material3d_render_commands(ctx) {
            output.push_material3d_render_command(command);
        }
    }
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

