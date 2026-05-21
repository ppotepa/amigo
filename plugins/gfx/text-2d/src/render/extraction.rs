use amigo_scene::SceneService;
use amigo_render_api::RenderExtractionOutput2d;

use crate::{Text2dDrawCommand, Text2dSceneService};

pub struct Text2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub text_scene_service: &'a Text2dSceneService,
}

pub struct Text2dRenderExtractor;

impl Text2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        super::TEXT_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Text2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_text2d_render_commands(ctx) {
            output.push_renderable_2d(super::text_draw_command_to_renderable_2d(&command));
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
