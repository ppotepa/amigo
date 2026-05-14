use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use super::shared::apply_panel_toggle;

pub(crate) struct DebugParticlesCommandHandler;

impl ConsoleCommandHandler for DebugParticlesCommandHandler {
    fn name(&self) -> &'static str {
        "debug-particles"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.particles",
            aliases: &[],
            category: "debug",
            help: "Show or hide particle overlay.",
            usage: "debug.particles on|off|toggle",
            examples: &[
                "debug.particles on",
                "debug.particles off",
                "debug.particles",
            ],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.particles"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(
            ctx,
            &command,
            DebugOverlayPanel::Particles,
            "debug.particles",
        )
    }
}
