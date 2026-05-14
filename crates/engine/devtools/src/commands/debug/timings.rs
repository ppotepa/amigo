use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use super::shared::apply_panel_toggle;

pub(crate) struct DebugTimingsCommandHandler;

impl ConsoleCommandHandler for DebugTimingsCommandHandler {
    fn name(&self) -> &'static str {
        "debug-timings"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.timings",
            aliases: &[],
            category: "debug",
            help: "Show or hide timings overlay.",
            usage: "debug.timings on|off|toggle",
            examples: &["debug.timings on", "debug.timings off", "debug.timings"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.timings"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Timings, "debug.timings")
    }
}
