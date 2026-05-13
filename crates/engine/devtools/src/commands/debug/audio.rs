use crate::DebugOverlayPanel;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

use super::shared::apply_panel_toggle;

pub(crate) struct DebugAudioCommandHandler;

impl ConsoleCommandHandler for DebugAudioCommandHandler {
    fn name(&self) -> &'static str {
        "debug-audio"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.audio",
            aliases: &[],
            category: "debug",
            help: "Show or hide audio overlay.",
            usage: "debug.audio on|off|toggle",
            examples: &["debug.audio on", "debug.audio off", "debug.audio"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.audio"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        apply_panel_toggle(ctx, &command, DebugOverlayPanel::Audio, "debug.audio")
    }
}



