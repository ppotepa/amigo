use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::super::*;

pub(crate) struct SceneCamera2dCommandHandler;

impl SceneCommandHandler for SceneCamera2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-camera-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::QueueCameraFollow2d { .. }
                | SceneCommand::QueueParallax2d { .. }
                | SceneCommand::QueueTileMapMarker2d { .. }
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueCameraFollow2d { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.camera_follow_scene_service
                    .queue(CameraFollow2dSceneCommand {
                        source_mod: command.source_mod.clone(),
                        entity_name: command.entity_name.clone(),
                        target: command.target.clone(),
                        offset: command.offset,
                        lerp: command.lerp,
                    });
                ctx.scene_event_queue.publish(SceneEvent::CameraFollowQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    target: command.target.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d camera follow `{}` -> `{}` from mod `{}`",
                    command.entity_name, command.target, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueParallax2d { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.parallax_scene_service.queue(Parallax2dSceneCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    camera: command.camera.clone(),
                    factor: command.factor,
                    anchor: command.anchor,
                    camera_origin: None,
                });
                ctx.scene_event_queue.publish(SceneEvent::ParallaxQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    camera: command.camera.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d parallax `{}` -> `{}` from mod `{}`",
                    command.entity_name, command.camera, command.source_mod
                ));
                Ok(())
            }
            SceneCommand::QueueTileMapMarker2d { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                let symbol = command.symbol.chars().next().unwrap_or_default();
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
                    ctx.dev_console_state.write_line(format!(
                        "cannot resolve tilemap marker `{}` for `{}` because no tilemap has been queued yet",
                        command.symbol, command.entity_name
                    ));
                    return Ok(());
                };

                let markers = marker_cells(&tilemap.tilemap, symbol);
                let Some(marker) = markers.get(command.index) else {
                    ctx.dev_console_state.write_line(format!(
                        "cannot resolve tilemap marker `{}`[{}] for `{}` in tilemap `{}`",
                        command.symbol, command.index, command.entity_name, tilemap.entity_name
                    ));
                    return Ok(());
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

                ctx.scene_event_queue.publish(SceneEvent::TileMapMarkerQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    symbol: command.symbol.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "anchored entity `{}` to tilemap marker `{}`[{}] in `{}`",
                    command.entity_name, command.symbol, command.index, tilemap.entity_name
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
