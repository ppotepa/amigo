use crate::debug_overlay::DebugOverlayPanel;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugMemoryCommandHandler;

impl ConsoleCommandHandler for DebugMemoryCommandHandler {
    fn name(&self) -> &'static str {
        "debug-memory"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.memory",
            aliases: &[],
            category: "debug",
            help: "Show or hide memory overlay placeholder.",
            usage: "debug.memory on|off|toggle",
            examples: &["debug.memory on", "debug.memory off", "debug.memory"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.memory"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Memory, "debug.memory")
    }
}
