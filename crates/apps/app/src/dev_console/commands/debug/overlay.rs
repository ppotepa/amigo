use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::{overlay_service, parse_toggle_action, state_label, ToggleAction};

pub(crate) struct DebugOverlayCommandHandler;

impl ConsoleCommandHandler for DebugOverlayCommandHandler {
    fn name(&self) -> &'static str { "debug-overlay" }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.overlay",
            aliases: &["overlay.debug"],
            category: "debug",
            help: "Enable or disable the engine debug overlay.",
            usage: "debug.overlay on|off|toggle",
            examples: &["debug.overlay on", "debug.overlay off", "debug.overlay"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(command.name.as_str(), "debug.overlay" | "overlay.debug")
    }

    fn handle(&self, ctx: &ConsoleCommandContext<'_>, command: ParsedConsoleCommand) -> ConsoleCommandResult {
        let overlay = match overlay_service(ctx) {
            Ok(service) => service,
            Err(result) => return result,
        };

        let enabled = match parse_toggle_action(&command) {
            Ok(ToggleAction::On) => {
                overlay.set_enabled(true);
                true
            }
            Ok(ToggleAction::Off) => {
                overlay.set_enabled(false);
                false
            }
            Ok(ToggleAction::Toggle) => overlay.toggle_enabled(),
            Err(result) => return result,
        };

        ConsoleCommandResult::ok(format!("debug.overlay {}", state_label(enabled)))
    }
}
