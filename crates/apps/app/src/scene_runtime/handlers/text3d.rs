use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::super::*;

pub(crate) struct SceneText3dCommandHandler;

impl SceneCommandHandler for SceneText3dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-text-3d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueText3d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueText3d { command } => {
                let entity = amigo_3d_text::queue_text3d_scene_command(
                    ctx.scene_service,
                    ctx.text3d_scene_service,
                    &command,
                );
                crate::app_helpers::register_mod_asset_reference(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.font,
                    "3d",
                    "text",
                );
                ctx.scene_event_queue.publish(SceneEvent::Text3dQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    font: command.font.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 3d text entity `{}` from mod `{}` with font `{}`",
                    command.entity_name,
                    command.source_mod,
                    command.font.as_str()
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
