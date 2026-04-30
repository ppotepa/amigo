use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneText2dCommandHandler;

impl SceneCommandHandler for SceneText2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-text-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueText2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueText2d { command } => {
                let entity = amigo_2d_text::queue_text2d_scene_command(
                    ctx.scene_service,
                    ctx.text_scene_service,
                    &command,
                );
                crate::app_helpers::register_mod_asset_reference(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.font,
                    "2d",
                    "text",
                );
                ctx.scene_event_queue.publish(SceneEvent::TextQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    font: command.font.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d text entity `{}` from mod `{}` with font `{}`",
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
