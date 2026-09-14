use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::{Mesh3dSceneCommand, MeshAnimation3dSceneCommand, SceneCommand};
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
            Mesh3dScriptCommandOutcome::Submit(SceneCommand::Plugin {
                command: amigo_scene::mesh_3d_plugin_scene_command(Mesh3dSceneCommand::new(
                    source_mod.clone(),
                    entity_name.clone(),
                    AssetKey::new(mesh_key.clone()),
                )),
            })
        }
        ("spawn", [entity_name, mesh_key]) => {
            Mesh3dScriptCommandOutcome::Submit(SceneCommand::Plugin {
                command: amigo_scene::mesh_3d_plugin_scene_command(Mesh3dSceneCommand::new(
                    ctx.selected_mod.to_owned(),
                    entity_name.clone(),
                    AssetKey::new(mesh_key.clone()),
                )),
            })
        }
        ("play_animation", [entity_name, clip, speed, looped]) => {
            let Ok(speed) = speed.parse::<f32>() else { return Mesh3dScriptCommandOutcome::Unhandled; };
            let Ok(looped) = looped.parse::<bool>() else { return Mesh3dScriptCommandOutcome::Unhandled; };
            Mesh3dScriptCommandOutcome::Submit(SceneCommand::Plugin { command:
                amigo_scene::mesh_animation_3d_plugin_scene_command(MeshAnimation3dSceneCommand {
                    source_mod: ctx.selected_mod.to_owned(), entity_name: entity_name.clone(), clip: clip.clone(), speed, looped, sample_fps: None,
                })
            })
        }
        ("play_animation", [entity_name, clip, speed, looped, sample_fps]) => {
            let (Ok(speed), Ok(looped), Ok(sample_fps)) = (
                speed.parse::<f32>(), looped.parse::<bool>(), sample_fps.parse::<f32>()
            ) else { return Mesh3dScriptCommandOutcome::Unhandled; };
            Mesh3dScriptCommandOutcome::Submit(SceneCommand::Plugin { command:
                amigo_scene::mesh_animation_3d_plugin_scene_command(MeshAnimation3dSceneCommand {
                    source_mod: ctx.selected_mod.to_owned(), entity_name: entity_name.clone(), clip: clip.clone(), speed, looped, sample_fps: Some(sample_fps),
                })
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
        command.namespace == "3d.mesh" && ((command.name == "spawn" && command.arguments.len() == 3)
            || (command.name == "play_animation" && matches!(command.arguments.len(), 4 | 5)))
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<amigo_scene::SceneCommandQueue>()?;
        match handle_mesh3d_script_command(Mesh3dScriptCommandContext { selected_mod: "" }, command)
        {
            Mesh3dScriptCommandOutcome::Submit(scene_command) => {
                scene_command_queue.submit(scene_command);
            }
            Mesh3dScriptCommandOutcome::Unhandled => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_animation_command_preserves_authored_pose_rate() {
        let outcome = handle_mesh3d_script_command(
            Mesh3dScriptCommandContext {
                selected_mod: "npr-city",
            },
            ScriptCommand::new(
                "3d.mesh",
                "play_animation",
                vec!["hero", "Walk", "1", "true", "12"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
            ),
        );
        let Mesh3dScriptCommandOutcome::Submit(SceneCommand::Plugin { command }) = outcome else {
            panic!("sampled animation should become a scene command");
        };
        let animation = command
            .payload_as::<MeshAnimation3dSceneCommand>()
            .expect("animation payload should retain its type");
        assert_eq!(animation.sample_fps, Some(12.0));
        assert_eq!(animation.clip, "Walk");
    }
}
