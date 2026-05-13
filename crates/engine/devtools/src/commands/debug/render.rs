use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugRenderCommandHandler;

impl ConsoleCommandHandler for DebugRenderCommandHandler {
    fn name(&self) -> &'static str {
        "debug-render"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.render",
            aliases: &[],
            category: "debug",
            help: "Show or hide render counters overlay.",
            usage: "debug.render on|off|toggle",
            examples: &["debug.render on", "debug.render off", "debug.render"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.render"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Render, "debug.render")
    }
}



