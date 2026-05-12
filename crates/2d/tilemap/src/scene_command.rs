use amigo_assets::{AssetCatalog, AssetKey};
use amigo_2d_physics::Physics2dSceneService;
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::{TileMap2dSceneService, queue_tilemap_scene_command};

pub struct TileMapSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub tilemap_scene_service: &'a TileMap2dSceneService,
    pub physics_scene_service: &'a Physics2dSceneService,
    pub asset_catalog: &'a AssetCatalog,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct TileMapSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub tileset: AssetKey,
    pub ruleset: Option<AssetKey>,
}

pub fn can_handle_tilemap_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueTileMap2d { .. })
}

pub fn handle_tilemap_scene_command(
    ctx: TileMapSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<TileMapSceneCommandOutcome> {
    match command {
        SceneCommand::QueueTileMap2d { command } => {
            let entity = queue_tilemap_scene_command(
                ctx.scene_service,
                ctx.tilemap_scene_service,
                ctx.physics_scene_service,
                ctx.asset_catalog,
                &command,
            );

            ctx.scene_event_queue.publish(SceneEvent::TileMapQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                tileset: command.tileset.clone(),
            });

            Ok(TileMapSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                tileset: command.tileset,
                ruleset: command.ruleset,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "tilemap-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
