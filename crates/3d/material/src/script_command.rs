use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::{Material3dSceneCommand, SceneCommand};
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

pub struct Material3dScriptCommandContext<'a> {
    pub selected_mod: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Material3dScriptCommandOutcome {
    Submit(SceneCommand),
    Unhandled,
}

pub fn handle_material3d_script_command(
    ctx: Material3dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Material3dScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("bind", [source_mod, entity_name, label, material_key]) => {
            Material3dScriptCommandOutcome::Submit(SceneCommand::QueueMaterial3d {
                command: Material3dSceneCommand::new(
                    source_mod.clone(),
                    entity_name.clone(),
                    label.clone(),
                    Some(AssetKey::new(material_key.clone())),
                ),
            })
        }
        ("bind", [entity_name, label, material_key]) => {
            Material3dScriptCommandOutcome::Submit(SceneCommand::QueueMaterial3d {
                command: Material3dSceneCommand::new(
                    ctx.selected_mod.to_owned(),
                    entity_name.clone(),
                    label.clone(),
                    Some(AssetKey::new(material_key.clone())),
                ),
            })
        }
        _ => Material3dScriptCommandOutcome::Unhandled,
    }
}

pub struct Material3dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Material3dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "3d.material"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "3d.material"
            && command.name == "bind"
            && command.arguments.len() == 4
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<amigo_scene::SceneCommandQueue>()?;
        match handle_material3d_script_command(
            Material3dScriptCommandContext { selected_mod: "" },
            command,
        ) {
            Material3dScriptCommandOutcome::Submit(scene_command) => {
                scene_command_queue.submit(scene_command);
            }
            Material3dScriptCommandOutcome::Unhandled => {}
        }
        Ok(())
    }
}

