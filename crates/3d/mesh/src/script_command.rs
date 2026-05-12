use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::{Mesh3dSceneCommand, SceneCommand};
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

pub struct Mesh3dScriptCommandContext<'a> {
    pub selected_mod: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mesh3dScriptCommandOutcome {
    Submit(SceneCommand),
    Unhandled,
}

pub fn handle_mesh3d_script_command(
    ctx: Mesh3dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Mesh3dScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("spawn", [source_mod, entity_name, mesh_key]) => {
            Mesh3dScriptCommandOutcome::Submit(SceneCommand::QueueMesh3d {
                command: Mesh3dSceneCommand::new(
                    source_mod.clone(),
                    entity_name.clone(),
                    AssetKey::new(mesh_key.clone()),
                ),
            })
        }
        ("spawn", [entity_name, mesh_key]) => {
            Mesh3dScriptCommandOutcome::Submit(SceneCommand::QueueMesh3d {
                command: Mesh3dSceneCommand::new(
                    ctx.selected_mod.to_owned(),
                    entity_name.clone(),
                    AssetKey::new(mesh_key.clone()),
                ),
            })
        }
        _ => Mesh3dScriptCommandOutcome::Unhandled,
    }
}

pub struct Mesh3dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Mesh3dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "3d.mesh"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "3d.mesh"
            && command.name == "spawn"
            && command.arguments.len() == 3
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<amigo_scene::SceneCommandQueue>()?;
        match handle_mesh3d_script_command(
            Mesh3dScriptCommandContext { selected_mod: "" },
            command,
        ) {
            Mesh3dScriptCommandOutcome::Submit(scene_command) => {
                scene_command_queue.submit(scene_command);
            }
            Mesh3dScriptCommandOutcome::Unhandled => {}
        }
        Ok(())
    }
}
