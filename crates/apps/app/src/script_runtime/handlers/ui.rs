use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

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
