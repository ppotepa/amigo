use std::collections::BTreeMap;

use amigo_input_actions::{InputActionBinding, InputActionId, InputActionMap, parse_key_code};
use amigo_scene::InputActionBindingSceneCommand;

use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneInputActionsCommandHandler;

impl SceneCommandHandler for SceneInputActionsCommandHandler {
    fn name(&self) -> &'static str {
        "scene-input-actions"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueInputActionMap { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueInputActionMap { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                let map = InputActionMap {
                    id: command.id.clone(),
                    actions: command
                        .actions
                        .iter()
                        .map(|(id, binding)| {
                            (
                                InputActionId::new(id.clone()),
                                input_action_binding_from_scene_command(binding),
                            )
                        })
                        .collect::<BTreeMap<_, _>>(),
                };

                ctx.input_action_service.register_map(map, command.active);
                ctx.scene_event_queue
                    .publish(SceneEvent::InputActionMapQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                        map_id: command.id.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued input action map `{}` from mod `{}` with {} actions",
                    command.id,
                    command.source_mod,
                    command.actions.len()
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

fn input_action_binding_from_scene_command(
    binding: &InputActionBindingSceneCommand,
) -> InputActionBinding {
    match binding {
        InputActionBindingSceneCommand::Axis { positive, negative } => InputActionBinding::Axis {
            positive: positive.iter().map(|key| parse_key_code(key)).collect(),
            negative: negative.iter().map(|key| parse_key_code(key)).collect(),
        },
        InputActionBindingSceneCommand::Button { pressed } => InputActionBinding::Button {
            pressed: pressed.iter().map(|key| parse_key_code(key)).collect(),
        },
    }
}
