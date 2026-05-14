use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

use crate::{SceneCommand, SceneCommandQueue, SceneKey};

pub struct SceneScriptCommandContext<'a> {
    pub scene_command_queue: &'a SceneCommandQueue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneScriptCommandOutcome {
    Submitted,
    Unhandled,
}

pub fn can_handle_scene_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "scene"
}

pub fn handle_scene_script_command(
    ctx: SceneScriptCommandContext<'_>,
    command: ScriptCommand,
) -> SceneScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("select", [scene_id]) => {
            ctx.scene_command_queue.submit(SceneCommand::SelectScene {
                scene: SceneKey::new(scene_id.clone()),
            });
            SceneScriptCommandOutcome::Submitted
        }
        ("reload", []) => {
            ctx.scene_command_queue
                .submit(SceneCommand::ReloadActiveScene);
            SceneScriptCommandOutcome::Submitted
        }
        ("activate-set", [set_id]) => {
            ctx.scene_command_queue
                .submit(SceneCommand::ActivateSet { id: set_id.clone() });
            SceneScriptCommandOutcome::Submitted
        }
        ("spawn", [entity_name]) => {
            ctx.scene_command_queue
                .submit(SceneCommand::SpawnNamedEntity {
                    name: entity_name.clone(),
                    transform: None,
                });
            SceneScriptCommandOutcome::Submitted
        }
        ("clear", []) => {
            ctx.scene_command_queue.submit(SceneCommand::ClearEntities);
            SceneScriptCommandOutcome::Submitted
        }
        _ => SceneScriptCommandOutcome::Unhandled,
    }
}

pub struct SceneScriptCommandHandler;

impl RuntimeScriptCommandHandler for SceneScriptCommandHandler {
    fn name(&self) -> &'static str {
        "scene"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_scene_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<SceneCommandQueue>()?;
        let _ = handle_scene_script_command(
            SceneScriptCommandContext {
                scene_command_queue: scene_command_queue.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
