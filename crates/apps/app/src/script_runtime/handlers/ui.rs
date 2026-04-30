use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};
use amigo_math::ColorRgba;

pub(super) struct UiScriptCommandHandler;

impl ScriptCommandHandler for UiScriptCommandHandler {
    fn name(&self) -> &'static str {
        "ui"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "ui")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("set-text", [path, value]) => {
                if ctx.ui_state_service.set_text(path.clone(), value.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("updated ui text override `{path}`"));
                }
            }
            ("set-value", [path, value]) => match value.parse::<f32>() {
                Ok(value) => {
                    if ctx.ui_state_service.set_value(path.clone(), value) {
                        ctx.dev_console_state.write_line(format!(
                            "updated ui value override `{path}` to {}",
                            value.clamp(0.0, 1.0)
                        ));
                    }
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "failed to parse ui value `{value}` as f32: {error}"
                )),
            },
            ("set_selected", [path, value]) | ("set-selected", [path, value]) => {
                if ctx
                    .ui_state_service
                    .set_selected(path.clone(), value.clone())
                {
                    ctx.dev_console_state.write_line(format!(
                        "updated ui selected override `{path}` to `{value}`"
                    ));
                }
            }
            ("set-color", [path, value]) => match parse_color_rgba_hex(value) {
                Some(color) => {
                    if ctx.ui_state_service.set_color(path.clone(), color) {
                        ctx.dev_console_state
                            .write_line(format!("updated ui color override `{path}`"));
                    }
                }
                None => ctx
                    .dev_console_state
                    .write_line(format!("failed to parse ui color `{value}`")),
            },
            ("set-background", [path, value]) | ("set_background", [path, value]) => {
                match parse_color_rgba_hex(value) {
                    Some(color) => {
                        if ctx.ui_state_service.set_background(path.clone(), color) {
                            ctx.dev_console_state
                                .write_line(format!("updated ui background override `{path}`"));
                        }
                    }
                    None => ctx
                        .dev_console_state
                        .write_line(format!("failed to parse ui background `{value}`")),
                }
            }
            ("show", [path]) => {
                if ctx.ui_state_service.show(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("showed ui path `{path}`"));
                }
            }
            ("hide", [path]) => {
                if ctx.ui_state_service.hide(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("hid ui path `{path}`"));
                }
            }
            ("enable", [path]) => {
                if ctx.ui_state_service.enable(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("enabled ui path `{path}`"));
                }
            }
            ("disable", [path]) => {
                if ctx.ui_state_service.disable(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("disabled ui path `{path}`"));
                }
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
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
