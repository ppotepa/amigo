use amigo_math::ColorRgba;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

use crate::{GlobalLight2dSceneService, LightGroup2dSceneService};

pub struct Lighting2dScriptCommandContext<'a> {
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lighting2dScriptCommandOutcome {
    Updated(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_lighting2d_script_command(command: &ScriptCommand) -> bool {
    matches!(command.namespace.as_str(), "2d.light" | "2d.light_group")
}

pub fn handle_lighting2d_script_command(
    ctx: Lighting2dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Lighting2dScriptCommandOutcome {
    match (
        command.namespace.as_str(),
        command.name.as_str(),
        command.arguments.as_slice(),
    ) {
        ("2d.light", "set_intensity", [id, intensity]) => match intensity.parse::<f32>() {
            Ok(intensity) => {
                if !ctx.global_light2d_scene_service.set_intensity(id, intensity) {
                    return Lighting2dScriptCommandOutcome::Updated(format!(
                        "global 2d light `{id}` not found"
                    ));
                }
                Lighting2dScriptCommandOutcome::Updated(format!(
                    "updated global 2d light `{id}` intensity"
                ))
            }
            Err(error) => Lighting2dScriptCommandOutcome::ParseError(format!(
                "invalid global 2d light intensity `{intensity}`: {error}"
            )),
        },
        ("2d.light", "set_color", [id, color]) => match parse_color_rgba_hex(color) {
            Some(color) => {
                if !ctx.global_light2d_scene_service.set_color(id, color) {
                    return Lighting2dScriptCommandOutcome::Updated(format!(
                        "global 2d light `{id}` not found"
                    ));
                }
                Lighting2dScriptCommandOutcome::Updated(format!("updated global 2d light `{id}`"))
            }
            None => {
                Lighting2dScriptCommandOutcome::ParseError(format!("invalid global 2d light color `{color}`"))
            }
        },
        ("2d.light_group", "set_intensity", [id, intensity]) => match intensity.parse::<f32>() {
            Ok(intensity) => {
                if !ctx.light_group2d_scene_service.set_intensity(id, intensity) {
                    return Lighting2dScriptCommandOutcome::Updated(format!(
                        "2d light group `{id}` not found"
                    ));
                }
                Lighting2dScriptCommandOutcome::Updated(format!(
                    "updated 2d light group `{id}` intensity"
                ))
            }
            Err(error) => Lighting2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d light group intensity `{intensity}`: {error}"
            )),
        },
        ("2d.light_group", "set_color", [id, color]) => match parse_color_rgba_hex(color) {
            Some(color) => {
                if !ctx.light_group2d_scene_service.set_color(id, color) {
                    return Lighting2dScriptCommandOutcome::Updated(format!(
                        "2d light group `{id}` not found"
                    ));
                }
                Lighting2dScriptCommandOutcome::Updated(format!("updated 2d light group `{id}`"))
            }
            None => Lighting2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d light group color `{color}`"
            )),
        },
        _ => Lighting2dScriptCommandOutcome::Unhandled,
    }
}

fn parse_color_rgba_hex(value: &str) -> Option<ColorRgba> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            parse_hex_channel(&hex[6..8])?,
        ),
        _ => return None,
    };
    Some(ColorRgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

fn parse_hex_channel(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}

pub struct Lighting2dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Lighting2dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "2d.light"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_lighting2d_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let global_light2d_scene_service = runtime.required::<GlobalLight2dSceneService>()?;
        let light_group2d_scene_service = runtime.required::<LightGroup2dSceneService>()?;
        let _ = handle_lighting2d_script_command(
            Lighting2dScriptCommandContext {
                global_light2d_scene_service: global_light2d_scene_service.as_ref(),
                light_group2d_scene_service: light_group2d_scene_service.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
