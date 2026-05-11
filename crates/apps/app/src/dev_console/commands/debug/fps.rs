use crate::debug_overlay::DebugOverlayPanel;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugFpsCommandHandler;

impl ConsoleCommandHandler for DebugFpsCommandHandler {
    fn name(&self) -> &'static str {
        "debug-fps"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.fps",
            aliases: &[],
            category: "debug",
            help: "Show or hide FPS overlay.",
            usage: "debug.fps on|off|toggle",
            examples: &["debug.fps on", "debug.fps off", "debug.fps"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.fps"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Fps, "debug.fps")
    }
}
