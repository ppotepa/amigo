use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::overlay_service;

pub(crate) struct DebugResetCommandHandler;

impl ConsoleCommandHandler for DebugResetCommandHandler {
    fn name(&self) -> &'static str {
        "debug-reset"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.reset",
            aliases: &[],
            category: "debug",
            help: "Reset debug overlay settings.",
            usage: "debug.reset",
            examples: &["debug.reset"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.reset"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        _command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let overlay = match overlay_service(ctx) {
            Ok(service) => service,
            Err(result) => return result,
        };
        overlay.reset();
        ConsoleCommandResult::ok("debug overlay reset")
    }
}
