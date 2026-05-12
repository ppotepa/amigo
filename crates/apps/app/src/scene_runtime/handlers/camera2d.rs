use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneCamera2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneCamera2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-camera-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_scene::can_handle_scene_camera2d_scene_command(command)
            || amigo_2d_tilemap::can_handle_tilemap_marker_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        if amigo_scene::can_handle_scene_camera2d_scene_command(&command) {
            let outcome = amigo_scene::handle_scene_camera2d_scene_command(
                amigo_scene::SceneCamera2dCommandContext {
                    scene_service: ctx.scene_service,
                    camera_follow_scene_service: ctx.camera_follow_scene_service,
                    parallax_scene_service: ctx.parallax_scene_service,
                    scene_event_queue: ctx.scene_event_queue,
                },
                command,
            )?;
            match outcome {
                amigo_scene::SceneCamera2dCommandOutcome::CameraFollow {
                    entity_name,
                    target,
                    source_mod,
                } => ctx.dev_console_state.write_line(format!(
                    "queued 2d camera follow `{}` -> `{}` from mod `{}`",
                    entity_name, target, source_mod
                )),
                amigo_scene::SceneCamera2dCommandOutcome::Parallax {
                    entity_name,
                    camera,
                    source_mod,
                } => ctx.dev_console_state.write_line(format!(
                    "queued 2d parallax `{}` -> `{}` from mod `{}`",
                    entity_name, camera, source_mod
                )),
            }
            return Ok(());
        }

        if amigo_2d_tilemap::can_handle_tilemap_marker_scene_command(&command) {
            let outcome = amigo_2d_tilemap::handle_tilemap_marker_scene_command(
                amigo_2d_tilemap::TileMapMarkerSceneCommandContext {
                    scene_service: ctx.scene_service,
                    tilemap_scene_service: ctx.tilemap_scene_service,
                    scene_event_queue: ctx.scene_event_queue,
                },
                command,
            )?;

            match outcome {
                amigo_2d_tilemap::TileMapMarkerSceneCommandOutcome::Anchored {
                    entity_name,
                    symbol,
                    index,
                    tilemap_entity,
                    ..
                } => {
                    ctx.dev_console_state.write_line(format!(
                        "anchored entity `{}` to tilemap marker `{}`[{}] in `{}`",
                        entity_name, symbol, index, tilemap_entity
                    ));
                }
                amigo_2d_tilemap::TileMapMarkerSceneCommandOutcome::MissingTileMap {
                    entity_name,
                    symbol,
                    ..
                } => {
                    ctx.dev_console_state.write_line(format!(
                        "cannot resolve tilemap marker `{}` for `{}` because no tilemap has been queued yet",
                        symbol, entity_name
                    ));
                }
                amigo_2d_tilemap::TileMapMarkerSceneCommandOutcome::MissingMarker {
                    entity_name,
                    symbol,
                    index,
                    tilemap_entity,
                    ..
                } => {
                    ctx.dev_console_state.write_line(format!(
                        "cannot resolve tilemap marker `{}`[{}] for `{}` in tilemap `{}`",
                        symbol, index, entity_name, tilemap_entity
                    ));
                }
            }

            return Ok(());
        }

        Err(AmigoError::Message(format!(
            "{} cannot handle command {}",
            self.name(),
            amigo_scene::format_scene_command(&command)
        )))
    }
}


