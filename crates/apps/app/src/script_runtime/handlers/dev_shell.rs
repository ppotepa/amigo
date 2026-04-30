use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

pub(super) struct DevShellScriptCommandHandler;

impl ScriptCommandHandler for DevShellScriptCommandHandler {
    fn name(&self) -> &'static str {
        "dev-shell"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "dev-shell")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("refresh-diagnostics", [target_mod]) => {
                ctx.dev_console_state.write_line(format!(
                    "diagnostics refreshed for mod={} scene={} window={} input={} render={} script={}",
                    target_mod,
                    ctx.launch_selection.selected_scene(),
                    ctx.diagnostics.window_backend,
                    ctx.diagnostics.input_backend,
                    ctx.diagnostics.render_backend,
                    ctx.diagnostics.script_backend
                ));
                ctx.script_event_queue.publish(ScriptEvent::new(
                    "dev-shell.diagnostics-refreshed",
                    vec![target_mod.clone()],
                ));
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
