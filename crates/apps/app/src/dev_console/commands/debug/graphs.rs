use crate::debug_overlay::DebugOverlayPanel;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::apply_panel_group_toggle;

pub(crate) struct DebugGraphsCommandHandler;

impl ConsoleCommandHandler for DebugGraphsCommandHandler {
    fn name(&self) -> &'static str {
        "debug-graphs"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.graphs",
            aliases: &[],
            category: "debug",
            help: "Show or hide all debug graphs.",
            usage: "debug.graphs on|off|toggle",
            examples: &["debug.graphs on", "debug.graphs off", "debug.graphs"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.graphs"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_group_toggle(
            ctx,
            &command,
            &[DebugOverlayPanel::FpsGraph],
            "debug.graphs",
        )
    }
}
