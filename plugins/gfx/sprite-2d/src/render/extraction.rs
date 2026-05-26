use amigo_math::{Transform2, Transform3, Vec2};
use amigo_render_api::RenderExtractionOutput2d;
use amigo_scene::SceneService;

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
        .map(|mut command| {
            if let Some(transform) = ctx.scene_service.transform_of(&command.entity_name) {
                command.transform = transform2_from_transform3(transform);
            }
            command
        })
        .collect()
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}

fn transform2_from_transform3(transform: Transform3) -> Transform2 {
    Transform2 {
        translation: Vec2::new(transform.translation.x, transform.translation.y),
        rotation_radians: transform.rotation_euler.z,
        scale: Vec2::new(transform.scale.x, transform.scale.y),
    }
}
