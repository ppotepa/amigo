use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::super::*;

pub(crate) struct SceneTileMap2dCommandHandler;

impl SceneCommandHandler for SceneTileMap2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-tilemap-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueTileMap2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueTileMap2d { command } => {
                crate::app_helpers::register_mod_asset_reference(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.tileset,
                    "2d",
                    "tilemap",
                );
                if let Some(ruleset) = command.ruleset.as_ref() {
                    crate::app_helpers::register_mod_asset_reference(
                        ctx.asset_catalog,
                        &command.source_mod,
                        ruleset,
                        "2d",
                        "tile-ruleset",
                    );
                }

                let entity = amigo_2d_tilemap::queue_tilemap_scene_command(
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
                ctx.dev_console_state.write_line(format!(
                    "queued 2d tilemap entity `{}` from mod `{}` with tileset `{}`",
                    command.entity_name,
                    command.source_mod,
                    command.tileset.as_str()
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}
