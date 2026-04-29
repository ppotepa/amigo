use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::ui_support;
use super::super::super::*;

pub(crate) struct SceneUiCommandHandler;

impl SceneCommandHandler for SceneUiCommandHandler {
    fn name(&self) -> &'static str {
        "scene-ui"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueUi { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueUi { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ui_support::register_ui_font_asset_references(
                    ctx.asset_catalog,
                    &command.source_mod,
                    &command.document,
                );
                ctx.ui_scene_service.queue(UiDrawCommand {
                    entity_id: entity,
                    entity_name: command.entity_name.clone(),
                    document: ui_support::convert_scene_ui_document(&command.document),
                });
                ctx.scene_event_queue.publish(SceneEvent::UiQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued ui document entity `{}` from mod `{}`",
                    command.entity_name, command.source_mod
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
