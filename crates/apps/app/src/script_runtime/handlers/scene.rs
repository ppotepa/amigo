use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

pub(super) struct SceneScriptCommandHandler;

impl ScriptCommandHandler for SceneScriptCommandHandler {
    fn name(&self) -> &'static str {
        "scene"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "scene")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("select", [scene_id]) => {
                ctx.scene_command_queue.submit(SceneCommand::SelectScene {
                    scene: SceneKey::new(scene_id.clone()),
                });
            }
            ("reload", []) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::ReloadActiveScene);
            }
            ("activate-set", [set_id]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::ActivateSet { id: set_id.clone() });
            }
            ("spawn", [entity_name]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::SpawnNamedEntity {
                        name: entity_name.clone(),
                        transform: None,
                    });
            }
            ("clear", []) => {
                ctx.scene_command_queue.submit(SceneCommand::ClearEntities);
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
