use amigo_math::ColorRgba;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

use crate::UiStateService;

pub struct UiScriptCommandContext<'a> {
    pub ui_state_service: &'a UiStateService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiScriptCommandOutcome {
    Updated(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_ui_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "ui"
}

pub fn handle_ui_script_command(
    ctx: UiScriptCommandContext<'_>,
    command: ScriptCommand,
) -> UiScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("set-text", [path, value]) => {
            if ctx.ui_state_service.set_text(path.clone(), value.clone()) {
                UiScriptCommandOutcome::Updated(format!("updated ui text override `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui text override `{path}` unchanged"))
            }
        }
        ("set-value", [path, value]) => match value.parse::<f32>() {
            Ok(value) => {
                if ctx.ui_state_service.set_value(path.clone(), value) {
                    UiScriptCommandOutcome::Updated(format!(
                        "updated ui value override `{path}` to {}",
                        value.clamp(0.0, 1.0)
                    ))
                } else {
                    UiScriptCommandOutcome::Updated(format!("ui value override `{path}` unchanged"))
                }
            }
            Err(error) => UiScriptCommandOutcome::ParseError(format!(
                "failed to parse ui value `{value}` as f32: {error}"
            )),
        },
        ("set_selected", [path, value]) | ("set-selected", [path, value]) => {
            if ctx.ui_state_service.set_selected(path.clone(), value.clone()) {
                UiScriptCommandOutcome::Updated(format!(
                    "updated ui selected override `{path}` to `{value}`"
                ))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui selected override `{path}` unchanged"))
            }
        }
        ("set-options", [path, options @ ..]) | ("set_options", [path, options @ ..]) => {
            let options = options
                .iter()
                .filter(|option| !option.is_empty())
                .cloned()
                .collect::<Vec<_>>();
            if ctx.ui_state_service.set_options(path.clone(), options.clone()) {
                UiScriptCommandOutcome::Updated(format!(
                    "updated ui options override `{path}` with {} options",
                    options.len()
                ))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui options override `{path}` unchanged"))
            }
        }
        ("set-color", [path, value]) => match parse_color_rgba_hex(value) {
            Some(color) => {
                if ctx.ui_state_service.set_color(path.clone(), color) {
                    UiScriptCommandOutcome::Updated(format!("updated ui color override `{path}`"))
                } else {
                    UiScriptCommandOutcome::Updated(format!("ui color override `{path}` unchanged"))
                }
            }
            None => UiScriptCommandOutcome::ParseError(format!("failed to parse ui color `{value}`")),
        },
        ("set-background", [path, value]) | ("set_background", [path, value]) => {
            match parse_color_rgba_hex(value) {
                Some(color) => {
                    if ctx.ui_state_service.set_background(path.clone(), color) {
                        UiScriptCommandOutcome::Updated(format!(
                            "updated ui background override `{path}`"
                        ))
                    } else {
                        UiScriptCommandOutcome::Updated(format!(
                            "ui background override `{path}` unchanged"
                        ))
                    }
                }
                None => UiScriptCommandOutcome::ParseError(format!(
                    "failed to parse ui background `{value}`"
                )),
            }
        }
        ("show", [path]) => {
            if ctx.ui_state_service.show(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("showed ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already visible"))
            }
        }
        ("hide", [path]) => {
            if ctx.ui_state_service.hide(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("hid ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already hidden"))
            }
        }
        ("enable", [path]) => {
            if ctx.ui_state_service.enable(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("enabled ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already enabled"))
            }
        }
        ("disable", [path]) => {
            if ctx.ui_state_service.disable(path.clone()) {
                UiScriptCommandOutcome::Updated(format!("disabled ui path `{path}`"))
            } else {
                UiScriptCommandOutcome::Updated(format!("ui path `{path}` already disabled"))
            }
        }
        _ => UiScriptCommandOutcome::Unhandled,
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

pub struct UiScriptCommandHandler;

impl RuntimeScriptCommandHandler for UiScriptCommandHandler {
    fn name(&self) -> &'static str {
        "ui"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_ui_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let ui_state_service = runtime.required::<UiStateService>()?;
        let _ = handle_ui_script_command(
            UiScriptCommandContext {
                ui_state_service: ui_state_service.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
