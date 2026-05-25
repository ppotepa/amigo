use amigo_2d_physics::Physics2dSceneService;
use amigo_assets::{AssetCatalog, AssetKey};
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use super::{TileMap2dSceneService, marker_cells, queue_tilemap_scene_command};

pub struct TileMap2dSceneCommandHandler;

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
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::TILEMAP_2D_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_tilemap_scene_command(
    ctx: TileMapSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<TileMapSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::TILEMAP_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<amigo_scene::TileMap2dSceneCommand>()
                .ok_or_else(|| {
                    AmigoError::Message(
                        "tilemap plugin scene command payload type mismatch".to_owned(),
                    )
                })?
                .clone();
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

pub struct TileMapMarkerSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub tilemap_scene_service: &'a TileMap2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileMapMarkerSceneCommandOutcome {
    Anchored {
        entity_name: String,
        source_mod: String,
        symbol: String,
        index: usize,
        tilemap_entity: String,
    },
    MissingTileMap {
        entity_name: String,
        source_mod: String,
        symbol: String,
    },
    MissingMarker {
        entity_name: String,
        source_mod: String,
        symbol: String,
        index: usize,
        tilemap_entity: String,
    },
}

pub fn can_handle_tilemap_marker_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::TILEMAP_MARKER_2D_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_tilemap_marker_scene_command(
    ctx: TileMapMarkerSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<TileMapMarkerSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::TILEMAP_MARKER_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let command = command
                .payload_as::<amigo_scene::TileMapMarker2dSceneCommand>()
                .ok_or_else(|| {
                    AmigoError::Message(
                        "tilemap marker plugin scene command payload type mismatch".to_owned(),
                    )
                })?
                .clone();
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());

            let symbol_char = command.symbol.chars().next().unwrap_or_default();
            let tilemap = command
                .tilemap_entity
                .as_deref()
                .and_then(|tilemap_entity| {
                    ctx.tilemap_scene_service
                        .commands()
                        .into_iter()
                        .find(|queued| queued.entity_name == tilemap_entity)
                })
                .or_else(|| ctx.tilemap_scene_service.commands().into_iter().next());

            let Some(tilemap) = tilemap else {
                return Ok(TileMapMarkerSceneCommandOutcome::MissingTileMap {
                    entity_name: command.entity_name,
                    source_mod: command.source_mod,
                    symbol: command.symbol,
                });
            };

            let markers = marker_cells(&tilemap.tilemap, symbol_char);
            let Some(marker) = markers.get(command.index) else {
                return Ok(TileMapMarkerSceneCommandOutcome::MissingMarker {
                    entity_name: command.entity_name,
                    source_mod: command.source_mod,
                    symbol: command.symbol,
                    index: command.index,
                    tilemap_entity: tilemap.entity_name,
                });
            };

            let mut transform = ctx
                .scene_service
                .transform_of(&command.entity_name)
                .unwrap_or_default();

            transform.translation.x =
                marker.origin.x + tilemap.tilemap.tile_size.x * 0.5 + command.offset.x;
            transform.translation.y =
                marker.origin.y + tilemap.tilemap.tile_size.y * 0.5 + command.offset.y;

            let _ = ctx
                .scene_service
                .set_transform(&command.entity_name, transform);

            ctx.scene_event_queue
                .publish(SceneEvent::TileMapMarkerQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    symbol: command.symbol.clone(),
                });

            Ok(TileMapMarkerSceneCommandOutcome::Anchored {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                symbol: command.symbol,
                index: command.index,
                tilemap_entity: tilemap.entity_name,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "tilemap-2d marker handler cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for TileMap2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_tilemap_scene_command(command)
            || can_handle_tilemap_marker_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        if can_handle_tilemap_scene_command(&command) {
            let scene_service = runtime.required::<SceneService>()?;
            let tilemap_scene_service = runtime.required::<TileMap2dSceneService>()?;
            let physics_scene_service = runtime.required::<Physics2dSceneService>()?;
            let asset_catalog = runtime.required::<AssetCatalog>()?;
            let scene_event_queue = runtime.required::<SceneEventQueue>()?;

            handle_tilemap_scene_command(
                TileMapSceneCommandContext {
                    scene_service: scene_service.as_ref(),
                    tilemap_scene_service: tilemap_scene_service.as_ref(),
                    physics_scene_service: physics_scene_service.as_ref(),
                    asset_catalog: asset_catalog.as_ref(),
                    scene_event_queue: scene_event_queue.as_ref(),
                },
                command,
            )?;
            return Ok(());
        }

        let scene_service = runtime.required::<SceneService>()?;
        let tilemap_scene_service = runtime.required::<TileMap2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;
        handle_tilemap_marker_scene_command(
            TileMapMarkerSceneCommandContext {
                scene_service: scene_service.as_ref(),
                tilemap_scene_service: tilemap_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
