use crate::debug_overlay::DebugOverlayPanel;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugStatsCommandHandler;

impl ConsoleCommandHandler for DebugStatsCommandHandler {
    fn name(&self) -> &'static str {
        "debug-stats"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.stats",
            aliases: &[],
            category: "debug",
            help: "Show or hide summary stats overlay.",
            usage: "debug.stats on|off|toggle",
            examples: &["debug.stats on", "debug.stats off", "debug.stats"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.stats"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Stats, "debug.stats")
    }
}
