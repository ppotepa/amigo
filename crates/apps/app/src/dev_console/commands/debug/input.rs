use crate::debug_overlay::DebugOverlayPanel;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugInputCommandHandler;

impl ConsoleCommandHandler for DebugInputCommandHandler {
    fn name(&self) -> &'static str {
        "debug-input"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.input",
            aliases: &[],
            category: "debug",
            help: "Show or hide input overlay.",
            usage: "debug.input on|off|toggle",
            examples: &["debug.input on", "debug.input off", "debug.input"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.input"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Input, "debug.input")
    }
}
