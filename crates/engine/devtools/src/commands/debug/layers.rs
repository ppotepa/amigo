use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use super::shared::apply_panel_toggle;

pub(crate) struct DebugLayersCommandHandler;

impl ConsoleCommandHandler for DebugLayersCommandHandler {
    fn name(&self) -> &'static str {
        "debug-layers"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.layers",
            aliases: &[],
            category: "debug",
            help: "Show or hide layers overlay.",
            usage: "debug.layers on|off|toggle",
            examples: &["debug.layers on", "debug.layers off", "debug.layers"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.layers"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Layers, "debug.layers")
    }
}
