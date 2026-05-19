use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

use crate::BeaconLight2dSceneService;

pub struct Beacon2dScriptCommandContext<'a> {
    pub beacon_scene_service: &'a BeaconLight2dSceneService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Beacon2dScriptCommandOutcome {
    Updated(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_beacon2d_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "2d.beacon"
}

pub fn handle_beacon2d_script_command(
    ctx: Beacon2dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Beacon2dScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("set_base_intensity", [target, value]) => {
            update_f32(target, value, "base intensity", |target, value| {
                ctx.beacon_scene_service.set_base_intensity(target, value)
            })
        }
        ("set_frequency_hz", [target, value]) => {
            update_f32(target, value, "frequency", |target, value| {
                ctx.beacon_scene_service.set_frequency_hz(target, value)
            })
        }
        ("set_duty_cycle", [target, value]) => {
            update_f32(target, value, "duty cycle", |target, value| {
                ctx.beacon_scene_service.set_duty_cycle(target, value)
            })
        }
        ("set_halo_radius_px", [target, value]) => {
            update_f32(target, value, "halo radius", |target, value| {
                ctx.beacon_scene_service.set_halo_radius_px(target, value)
            })
        }
        ("set_core_radius_px", [target, value]) => {
            update_f32(target, value, "core radius", |target, value| {
                ctx.beacon_scene_service.set_core_radius_px(target, value)
            })
        }
        ("set_glow_strength", [target, value]) => {
            update_f32(target, value, "glow strength", |target, value| {
                ctx.beacon_scene_service.set_glow_strength(target, value)
            })
        }
        ("set_beam_enabled", [target, value]) => {
            update_bool(target, value, "beam enabled", |target, value| {
                ctx.beacon_scene_service.set_beam_enabled(target, value)
            })
        }
        ("set_beam_length_px", [target, value]) => {
            update_f32(target, value, "beam length", |target, value| {
                ctx.beacon_scene_service.set_beam_length_px(target, value)
            })
        }
        ("set_beam_width_degrees", [target, value]) => {
            update_f32(target, value, "beam width", |target, value| {
                ctx.beacon_scene_service
                    .set_beam_width_degrees(target, value)
            })
        }
        ("set_beam_strength", [target, value]) => {
            update_f32(target, value, "beam strength", |target, value| {
                ctx.beacon_scene_service.set_beam_strength(target, value)
            })
        }
        ("set_aberration_px", [target, value]) => {
            update_f32(target, value, "aberration", |target, value| {
                ctx.beacon_scene_service.set_aberration_px(target, value)
            })
        }

("set_bloom", [target, value]) => update_f32(target, value, "bloom", |target, value| {
            ctx.beacon_scene_service.set_bloom(target, value)
        }),
("set_position_2d", [target, x, y]) => {
            update_vec2(target, x, y, "position", |target, x, y| {
                ctx.beacon_scene_service.set_position_2d(target, x, y)
            })
        }
        ("set_distance_m", [target, value]) => {
            update_f32(target, value, "distance", |target, value| {
                ctx.beacon_scene_service.set_distance_m(target, value)
            })
        }
        ("set_z_depth", [target, value]) => update_f32(target, value, "z depth", |target, value| {
            ctx.beacon_scene_service.set_z_depth(target, value)
        }),
        _ => Beacon2dScriptCommandOutcome::Unhandled,
    }
}

fn update_f32(
    target: &str,
    value: &str,
    label: &str,
    update: impl FnOnce(&str, f32) -> bool,
) -> Beacon2dScriptCommandOutcome {
    match value.parse::<f32>() {
        Ok(value) => {
            if !update(target, value) {
                return Beacon2dScriptCommandOutcome::Updated(format!(
                    "beacon `{target}` not found"
                ));
            }
            Beacon2dScriptCommandOutcome::Updated(format!("updated beacon `{target}` {label}"))
        }
        Err(error) => Beacon2dScriptCommandOutcome::ParseError(format!(
            "invalid beacon {label} `{value}`: {error}"
        )),
    }
}

fn update_bool(
    target: &str,
    value: &str,
    label: &str,
    update: impl FnOnce(&str, bool) -> bool,
) -> Beacon2dScriptCommandOutcome {
    match value.parse::<bool>() {
        Ok(value) => {
            if !update(target, value) {
                return Beacon2dScriptCommandOutcome::Updated(format!(
                    "beacon `{target}` not found"
                ));
            }
            Beacon2dScriptCommandOutcome::Updated(format!("updated beacon `{target}` {label}"))
        }
        Err(error) => Beacon2dScriptCommandOutcome::ParseError(format!(
            "invalid beacon {label} `{value}`: {error}"
        )),
    }
}

fn update_vec2(
    target: &str,
    x: &str,
    y: &str,
    label: &str,
    update: impl FnOnce(&str, f32, f32) -> bool,
) -> Beacon2dScriptCommandOutcome {
    let Ok(x) = x.parse::<f32>() else {
        return Beacon2dScriptCommandOutcome::ParseError(format!(
            "invalid 2d beacon {label} x `{x}` for `{target}`"
        ));
    };
    let Ok(y) = y.parse::<f32>() else {
        return Beacon2dScriptCommandOutcome::ParseError(format!(
            "invalid 2d beacon {label} y `{y}` for `{target}`"
        ));
    };
    if update(target, x, y) {
        Beacon2dScriptCommandOutcome::Updated(format!(
            "updated 2d beacon `{target}` {label} to ({x}, {y})"
        ))
    } else {
        Beacon2dScriptCommandOutcome::Updated(format!(
            "2d beacon `{target}` not found for {label}"
        ))
    }
}

pub struct Beacon2dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Beacon2dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "2d.beacon"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_beacon2d_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let beacon_scene_service = runtime.required::<BeaconLight2dSceneService>()?;
        let _ = handle_beacon2d_script_command(
            Beacon2dScriptCommandContext {
                beacon_scene_service: beacon_scene_service.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
