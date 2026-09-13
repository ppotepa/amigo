use crate::MeshSceneService;
use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::{Mesh3dSceneCommand, SceneCommand};
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

pub struct Mesh3dScriptCommandContext<'a> {
    pub selected_mod: &'a str,
    pub mesh_scene_service: Option<&'a MeshSceneService>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mesh3dScriptCommandOutcome {
    Submit(SceneCommand),
    AppliedNprPreset {
        entity_name: String,
        preset_id: String,
    },
    SetNprTemporalPathSmoothing {
        entity_name: String,
        enabled: bool,
    },
    SetNprRenderStrategy {
        entity_name: String,
        strategy: amigo_render_api::NprRenderStrategy3d,
    },
    SetNprGpuDebugMode {
        entity_name: String,
        debug_mode: amigo_render_api::NprGpuDebugMode3d,
    },
    SetMeshAsset {
        entity_name: String,
        mesh_key: String,
    },
    SetMeshAnimation {
        entity_name: String,
        clip_index: u32,
        time_seconds: f32,
    },
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
        ("apply_npr_preset", [entity_name, preset_id]) => {
            let Some(mesh_scene_service) = ctx.mesh_scene_service else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            if mesh_scene_service.apply_npr_preset(entity_name, preset_id) {
                Mesh3dScriptCommandOutcome::AppliedNprPreset {
                    entity_name: entity_name.clone(),
                    preset_id: preset_id.clone(),
                }
            } else {
                Mesh3dScriptCommandOutcome::Unhandled
            }
        }
        ("set_npr_temporal_path_smoothing", [entity_name, enabled]) => {
            let Some(mesh_scene_service) = ctx.mesh_scene_service else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let enabled = enabled == "true" || enabled == "1" || enabled == "on";
            if mesh_scene_service.set_npr_temporal_path_smoothing(entity_name, enabled) {
                Mesh3dScriptCommandOutcome::SetNprTemporalPathSmoothing {
                    entity_name: entity_name.clone(),
                    enabled,
                }
            } else {
                Mesh3dScriptCommandOutcome::Unhandled
            }
        }
        ("set_npr_render_strategy", [entity_name, strategy]) => {
            let Some(mesh_scene_service) = ctx.mesh_scene_service else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let Some(strategy) = npr_render_strategy_from_script(strategy) else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            if mesh_scene_service.set_npr_render_strategy(entity_name, strategy) {
                Mesh3dScriptCommandOutcome::SetNprRenderStrategy {
                    entity_name: entity_name.clone(),
                    strategy,
                }
            } else {
                Mesh3dScriptCommandOutcome::Unhandled
            }
        }
        ("set_npr_gpu_debug_mode", [entity_name, debug_mode]) => {
            let Some(mesh_scene_service) = ctx.mesh_scene_service else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let Some(debug_mode) = amigo_render_api::NprGpuDebugMode3d::parse(debug_mode) else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            if mesh_scene_service.set_npr_gpu_debug_mode(entity_name, debug_mode) {
                Mesh3dScriptCommandOutcome::SetNprGpuDebugMode {
                    entity_name: entity_name.clone(),
                    debug_mode,
                }
            } else {
                Mesh3dScriptCommandOutcome::Unhandled
            }
        }
        ("set_mesh_asset", [entity_name, mesh_key]) => {
            let Some(mesh_scene_service) = ctx.mesh_scene_service else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            if mesh_scene_service.set_mesh_asset(entity_name, AssetKey::new(mesh_key.clone())) {
                Mesh3dScriptCommandOutcome::SetMeshAsset {
                    entity_name: entity_name.clone(),
                    mesh_key: mesh_key.clone(),
                }
            } else {
                Mesh3dScriptCommandOutcome::Unhandled
            }
        }
        ("set_mesh_animation", [entity_name, clip_index, time_seconds, speed, playing]) => {
            let Some(mesh_scene_service) = ctx.mesh_scene_service else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let Ok(clip_index) = clip_index.parse::<u32>() else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let Ok(time_seconds) = time_seconds.parse::<f32>() else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let Ok(speed) = speed.parse::<f32>() else {
                return Mesh3dScriptCommandOutcome::Unhandled;
            };
            let playing = playing == "true" || playing == "1" || playing == "on";
            if mesh_scene_service.set_mesh_animation(
                entity_name,
                amigo_render_api::MeshAnimation3d {
                    clip_index,
                    time_seconds,
                    speed,
                    playing,
                },
            ) {
                Mesh3dScriptCommandOutcome::SetMeshAnimation {
                    entity_name: entity_name.clone(),
                    clip_index,
                    time_seconds,
                }
            } else {
                Mesh3dScriptCommandOutcome::Unhandled
            }
        }
        _ => Mesh3dScriptCommandOutcome::Unhandled,
    }
}

fn npr_render_strategy_from_script(value: &str) -> Option<amigo_render_api::NprRenderStrategy3d> {
    match value.trim() {
        "gpu_realtime" => Some(amigo_render_api::NprRenderStrategy3d::GpuRealtime),
        "cpu_reference" => Some(amigo_render_api::NprRenderStrategy3d::CpuReference),
        _ => None,
    }
}

pub struct Mesh3dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Mesh3dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "3d.mesh"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "3d.mesh"
            && ((command.name == "spawn" && command.arguments.len() == 3)
                || (command.name == "apply_npr_preset" && command.arguments.len() == 2)
                || (command.name == "set_npr_temporal_path_smoothing"
                    && command.arguments.len() == 2)
                || (command.name == "set_npr_render_strategy" && command.arguments.len() == 2)
                || (command.name == "set_npr_gpu_debug_mode" && command.arguments.len() == 2)
                || (command.name == "set_mesh_asset" && command.arguments.len() == 2)
                || (command.name == "set_mesh_animation" && command.arguments.len() == 5))
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<amigo_scene::SceneCommandQueue>()?;
        let mesh_scene_service = runtime.resolve::<MeshSceneService>();
        match handle_mesh3d_script_command(
            Mesh3dScriptCommandContext {
                selected_mod: "",
                mesh_scene_service: mesh_scene_service.as_deref(),
            },
            command,
        ) {
            Mesh3dScriptCommandOutcome::Submit(scene_command) => {
                scene_command_queue.submit(scene_command);
            }
            Mesh3dScriptCommandOutcome::AppliedNprPreset { .. } => {}
            Mesh3dScriptCommandOutcome::SetNprTemporalPathSmoothing { .. } => {}
            Mesh3dScriptCommandOutcome::SetNprRenderStrategy { .. } => {}
            Mesh3dScriptCommandOutcome::SetNprGpuDebugMode { .. } => {}
            Mesh3dScriptCommandOutcome::SetMeshAsset { .. } => {}
            Mesh3dScriptCommandOutcome::SetMeshAnimation { .. } => {}
            Mesh3dScriptCommandOutcome::Unhandled => {}
        }
        Ok(())
    }
}
