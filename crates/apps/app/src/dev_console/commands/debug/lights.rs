use crate::debug_overlay::DebugOverlayPanel;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugLightsCommandHandler;

impl ConsoleCommandHandler for DebugLightsCommandHandler {
    fn name(&self) -> &'static str { "debug-lights" }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.lights",
            aliases: &[],
            category: "debug",
            help: "Show or hide lights overlay.",
            usage: "debug.lights on|off|toggle",
            examples: &["debug.lights on", "debug.lights off", "debug.lights"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool { command.name == "debug.lights" }

    fn handle(&self, ctx: &ConsoleCommandContext<'_>, command: ParsedConsoleCommand) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Lights, "debug.lights")
    }
}
