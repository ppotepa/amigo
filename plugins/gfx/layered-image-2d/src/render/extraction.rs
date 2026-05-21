use amigo_scene::SceneService;
use amigo_render_api::RenderExtractionOutput2d;

use crate::{LayeredImageDrawCommand, LayeredImageSceneService};

use super::LAYERED_IMAGE_2D_EXTRACTOR_ID;

pub struct LayeredImage2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub layered_image_scene_service: &'a LayeredImageSceneService,
}

pub struct LayeredImage2dRenderExtractor;

impl LayeredImage2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        LAYERED_IMAGE_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: LayeredImage2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_layered_image2d_render_commands(ctx) {
            output.push_renderable_2d(super::layered_image_draw_command_to_renderable_2d(
                &command,
            ));
        }
    }
}

pub fn extract_layered_image2d_render_commands(
    ctx: LayeredImage2dRenderExtractionContext<'_>,
) -> Vec<LayeredImageDrawCommand> {
    ctx.layered_image_scene_service
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
