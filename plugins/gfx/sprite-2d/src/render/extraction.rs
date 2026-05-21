use amigo_scene::SceneService;
use amigo_render_api::RenderExtractionOutput2d;

use crate::sprite::{SpriteDrawCommand, SpriteSceneService};

pub struct Sprite2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub sprite_scene_service: &'a SpriteSceneService,
}

pub struct Sprite2dRenderExtractor;

impl Sprite2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        super::SPRITE_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Sprite2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_sprite2d_render_commands(ctx) {
            output.push_renderable_2d(super::sprite_draw_command_to_renderable_2d(&command));
        }
    }
}

pub fn extract_sprite2d_render_commands(
    ctx: Sprite2dRenderExtractionContext<'_>,
) -> Vec<SpriteDrawCommand> {
    ctx.sprite_scene_service
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
