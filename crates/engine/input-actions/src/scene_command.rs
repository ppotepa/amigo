use std::collections::BTreeMap;

use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    InputActionBindingSceneCommand, SceneCommand, SceneEvent, SceneEventQueue, SceneService,
    format_scene_command,
};

use crate::{
    InputActionBinding, InputActionId, InputActionMap, InputActionService, parse_key_code,
};

pub struct InputActionsSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub input_action_service: &'a InputActionService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct InputActionsSceneCommandOutcome {
    pub id: String,
    pub source_mod: String,
    pub action_count: usize,
}

pub fn can_handle_input_actions_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueInputActionMap { .. })
}

pub fn handle_input_actions_scene_command(
    ctx: InputActionsSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<InputActionsSceneCommandOutcome> {
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
                    entity_name: command.entity_name,
                    map_id: command.id.clone(),
                });
            Ok(InputActionsSceneCommandOutcome {
                id: command.id,
                source_mod: command.source_mod,
                action_count: command.actions.len(),
            })
        }
        _ => Err(AmigoError::Message(format!(
            "input-actions cannot handle command {}",
            format_scene_command(&command)
        ))),
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
