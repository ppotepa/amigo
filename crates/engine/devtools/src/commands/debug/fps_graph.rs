use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugFpsGraphCommandHandler;

impl ConsoleCommandHandler for DebugFpsGraphCommandHandler {
    fn name(&self) -> &'static str {
        "debug-fps-graph"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.fps_graph",
            aliases: &[],
            category: "debug",
            help: "Show or hide FPS graph overlay.",
            usage: "debug.fps_graph on|off|toggle",
            examples: &[
                "debug.fps_graph on",
                "debug.fps_graph off",
                "debug.fps_graph",
            ],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.fps_graph"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(
            ctx,
            &command,
            DebugOverlayPanel::FpsGraph,
            "debug.fps_graph",
        )
    }
}



