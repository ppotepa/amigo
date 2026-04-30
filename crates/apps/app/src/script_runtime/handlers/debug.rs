use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

pub(super) struct DebugScriptCommandHandler;

impl ScriptCommandHandler for DebugScriptCommandHandler {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "debug")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("log", [line]) => {
                ctx.dev_console_state.write_line(format!("script: {line}"));
            }
            ("warn", [line]) => {
                ctx.dev_console_state
                    .write_line(format!("script warning: {line}"));
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
