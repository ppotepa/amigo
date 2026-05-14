use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use super::shared::apply_panel_toggle;

pub(crate) struct DebugSchedulerCommandHandler;

impl ConsoleCommandHandler for DebugSchedulerCommandHandler {
    fn name(&self) -> &'static str {
        "debug-scheduler"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.scheduler",
            aliases: &[],
            category: "debug",
            help: "Show or hide scheduler overlay.",
            usage: "debug.scheduler on|off|toggle",
            examples: &[
                "debug.scheduler on",
                "debug.scheduler off",
                "debug.scheduler",
            ],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.scheduler"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(
            ctx,
            &command,
            DebugOverlayPanel::Scheduler,
            "debug.scheduler",
        )
    }
}
