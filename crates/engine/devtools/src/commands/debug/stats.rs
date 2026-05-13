use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

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



