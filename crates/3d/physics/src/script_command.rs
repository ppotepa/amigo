use amigo_core::AmigoResult;
use amigo_math::Vec3;
use amigo_runtime::Runtime;
use amigo_scene::{
    BoxCollider3dSceneCommand, RigidBody3dSceneCommand, SceneCommand,
    StaticBoxCollider3dSceneCommand,
};
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

pub struct Physics3dScriptCommandContext<'a> {
    pub selected_mod: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Physics3dScriptCommandOutcome {
    Submit(SceneCommand),
    Unhandled,
}

pub fn handle_physics3d_script_command(
    ctx: Physics3dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Physics3dScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("dynamic_box", [source_mod, entity_name, sx, sy, sz, vx, vy, vz]) => {
            submit_dynamic_box(source_mod, entity_name, sx, sy, sz, vx, vy, vz)
        }
        ("dynamic_box", [entity_name, sx, sy, sz, vx, vy, vz]) => {
            submit_dynamic_box(ctx.selected_mod, entity_name, sx, sy, sz, vx, vy, vz)
        }
        ("static_box", [source_mod, entity_name, sx, sy, sz]) => {
            submit_static_box(source_mod, entity_name, sx, sy, sz)
        }
        ("static_box", [entity_name, sx, sy, sz]) => {
            submit_static_box(ctx.selected_mod, entity_name, sx, sy, sz)
        }
        _ => Physics3dScriptCommandOutcome::Unhandled,
    }
}

fn submit_dynamic_box(
    source_mod: &str,
    entity_name: &str,
    sx: &str,
    sy: &str,
    sz: &str,
    vx: &str,
    vy: &str,
    vz: &str,
) -> Physics3dScriptCommandOutcome {
    let Some(_size) = parse_vec3(sx, sy, sz) else {
        return Physics3dScriptCommandOutcome::Unhandled;
    };
    let Some(velocity) = parse_vec3(vx, vy, vz) else {
        return Physics3dScriptCommandOutcome::Unhandled;
    };
    Physics3dScriptCommandOutcome::Submit(SceneCommand::Plugin {
        command: amigo_scene::rigid_body_3d_plugin_scene_command(RigidBody3dSceneCommand::new(
            source_mod.to_owned(),
            entity_name.to_owned(),
            velocity,
            1.0,
            0.05,
        )),
    })
}

fn submit_static_box(
    source_mod: &str,
    entity_name: &str,
    sx: &str,
    sy: &str,
    sz: &str,
) -> Physics3dScriptCommandOutcome {
    let Some(size) = parse_vec3(sx, sy, sz) else {
        return Physics3dScriptCommandOutcome::Unhandled;
    };
    Physics3dScriptCommandOutcome::Submit(SceneCommand::Plugin {
        command: amigo_scene::static_box_collider_3d_plugin_scene_command(
            StaticBoxCollider3dSceneCommand::new(
                source_mod.to_owned(),
                entity_name.to_owned(),
                size,
                Vec3::ZERO,
            ),
        ),
    })
}

fn parse_vec3(x: &str, y: &str, z: &str) -> Option<Vec3> {
    Some(Vec3::new(
        x.parse::<f32>().ok()?,
        y.parse::<f32>().ok()?,
        z.parse::<f32>().ok()?,
    ))
}

pub struct Physics3dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Physics3dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "3d.physics"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "3d.physics"
            && matches!(command.name.as_str(), "dynamic_box" | "static_box")
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<amigo_scene::SceneCommandQueue>()?;
        match handle_physics3d_script_command(
            Physics3dScriptCommandContext { selected_mod: "" },
            command.clone(),
        ) {
            Physics3dScriptCommandOutcome::Submit(scene_command) => {
                scene_command_queue.submit(scene_command);
                if command.name == "dynamic_box" {
                    let args = command.arguments;
                    let offset = if args.len() == 8 { 1 } else { 0 };
                    if args.len() >= offset + 4 {
                        scene_command_queue.submit(SceneCommand::Plugin {
                            command: amigo_scene::box_collider_3d_plugin_scene_command(
                                BoxCollider3dSceneCommand::new(
                                    if offset == 1 {
                                        args[0].clone()
                                    } else {
                                        String::new()
                                    },
                                    args[offset].clone(),
                                    Vec3::new(
                                        args[offset + 1].parse().unwrap_or(1.0),
                                        args[offset + 2].parse().unwrap_or(1.0),
                                        args[offset + 3].parse().unwrap_or(1.0),
                                    ),
                                    Vec3::ZERO,
                                ),
                            ),
                        });
                    }
                }
            }
            Physics3dScriptCommandOutcome::Unhandled => {}
        }
        Ok(())
    }
}
