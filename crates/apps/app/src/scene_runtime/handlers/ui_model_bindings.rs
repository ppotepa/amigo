use amigo_scene::{SceneCommand, UiModelBindingKindSceneCommand};

use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneUiModelBindingsCommandHandler;

impl SceneCommandHandler for SceneUiModelBindingsCommandHandler {
    fn name(&self) -> &'static str {
        "scene-ui-model-bindings"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueUiModelBindings { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueUiModelBindings { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                for binding in command.bindings {
                    ctx.ui_model_binding_service.queue(UiModelBinding {
                        path: binding.path,
                        state_key: binding.state_key,
                        kind: match binding.kind {
                            UiModelBindingKindSceneCommand::Text => UiModelBindingKind::Text,
                            UiModelBindingKindSceneCommand::Value => UiModelBindingKind::Value,
                            UiModelBindingKindSceneCommand::Visible => UiModelBindingKind::Visible,
                            UiModelBindingKindSceneCommand::Enabled => UiModelBindingKind::Enabled,
                            UiModelBindingKindSceneCommand::Selected => {
                                UiModelBindingKind::Selected
                            }
                            UiModelBindingKindSceneCommand::Color => UiModelBindingKind::Color,
                            UiModelBindingKindSceneCommand::Background => {
                                UiModelBindingKind::Background
                            }
                        },
                        format: binding.format,
                    });
                }
                ctx.scene_event_queue
                    .publish(SceneEvent::UiModelBindingsQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued ui model bindings `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            other => Err(AmigoError::Message(format!(
                "{} cannot handle {}",
                self.name(),
                amigo_scene::format_scene_command(&other)
            ))),
        }
    }
}
