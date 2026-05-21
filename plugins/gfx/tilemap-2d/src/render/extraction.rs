use amigo_scene::SceneService;
use amigo_render_api::RenderExtractionOutput2d;

use crate::{TileMap2dDrawCommand, TileMap2dSceneService};

use super::TILEMAP_2D_EXTRACTOR_ID;

pub struct TileMap2dRenderExtractionContext<'a> {
    pub scene_service: &'a SceneService,
    pub tilemap_scene_service: &'a TileMap2dSceneService,
}

pub struct TileMap2dRenderExtractor;

impl TileMap2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        TILEMAP_2D_EXTRACTOR_ID
    }

    pub fn extract(
        &self,
        ctx: TileMap2dRenderExtractionContext<'_>,
        output: &mut impl RenderExtractionOutput2d,
    ) {
        for command in extract_tilemap2d_render_commands(ctx) {
            output.push_renderable_2d(super::tilemap_draw_command_to_renderable_2d(&command));
        }
    }
}

pub fn extract_tilemap2d_render_commands(
    ctx: TileMap2dRenderExtractionContext<'_>,
) -> Vec<TileMap2dDrawCommand> {
    ctx.tilemap_scene_service
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
