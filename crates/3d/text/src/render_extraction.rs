use amigo_scene::SceneService;

use crate::{Text3dDrawCommand, Text3dSceneService};

pub struct Text3dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub text3d_scene_service: &'a Text3dSceneService,
}

pub trait Text3dRenderOutput {
    fn push_text3d_render_command(&mut self, command: Text3dDrawCommand);
}

pub struct Text3dRenderExtractor;

impl Text3dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "text_3d"
    }

    pub fn extract(
        &self,
        ctx: Text3dRenderExtractionContext<'_>,
        output: &mut impl Text3dRenderOutput,
    ) {
        for command in extract_text3d_render_commands(ctx) {
            output.push_text3d_render_command(command);
        }
    }
}

pub fn extract_text3d_render_commands(
    ctx: Text3dRenderExtractionContext<'_>,
) -> Vec<Text3dDrawCommand> {
    ctx.text3d_scene_service
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
