use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneLayeredImage2dCommandHandler;

impl SceneCommandHandler for SceneLayeredImage2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-layered-image-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueLayeredImage2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueLayeredImage2d { command } => {
                let entity = amigo_2d_layered_image::queue_layered_image_scene_command(
                    ctx.scene_service,
                    ctx.layered_image_scene_service,
                    &command,
                );

                crate::app_helpers::register_mod_asset_reference(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.asset,
                    "2d",
                    "layered-image",
                );

                ctx.dev_console_state.write_line(format!(
                    "queued layered image entity `{}` from mod `{}` with asset `{}`",
                    command.entity_name,
                    command.source_mod,
                    command.asset.as_str()
                ));

                ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                    entity_id: entity.raw(),
                    name: command.entity_name,
                });

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
