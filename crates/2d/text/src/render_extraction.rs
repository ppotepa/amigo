use amigo_scene::SceneService;

use crate::{Text2dDrawCommand, Text2dSceneService};

pub struct Text2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub text_scene_service: &'a Text2dSceneService,
}

pub trait Text2dRenderOutput {
    fn push_text2d_render_command(&mut self, command: Text2dDrawCommand);
}

pub struct Text2dRenderExtractor;

impl Text2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "text_2d"
    }

    pub fn extract(
        &self,
        ctx: Text2dRenderExtractionContext<'_>,
        output: &mut impl Text2dRenderOutput,
    ) {
        for command in extract_text2d_render_commands(ctx) {
            output.push_text2d_render_command(command);
        }
    }
}

pub fn extract_text2d_render_commands(
    ctx: Text2dRenderExtractionContext<'_>,
) -> Vec<Text2dDrawCommand> {
    ctx.text_scene_service
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
