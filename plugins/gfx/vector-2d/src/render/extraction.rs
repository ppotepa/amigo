use amigo_math::{Transform2, Transform3, Vec2};
use amigo_render_api::RenderExtractionOutput2d;
use amigo_scene::SceneService;

use crate::vector::{VectorSceneService, VectorShape2dDrawCommand};

pub struct Vector2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub vector_scene_service: &'a VectorSceneService,
}

pub struct Vector2dRenderExtractor;

impl Vector2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        super::VECTOR_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: Vector2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_vector2d_render_commands(ctx) {
            output.push_renderable_2d(super::vector_draw_command_to_renderable_2d(&command));
        }
    }
}

pub fn extract_vector2d_render_commands(
    ctx: Vector2dRenderExtractionContext<'_>,
) -> Vec<VectorShape2dDrawCommand> {
    ctx.vector_scene_service
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
