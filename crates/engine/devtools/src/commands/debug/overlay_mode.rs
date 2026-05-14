use crate::DebugOverlayLayoutMode;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use super::shared::overlay_service;

pub(crate) struct DebugOverlayModeCommandHandler;

impl ConsoleCommandHandler for DebugOverlayModeCommandHandler {
    fn name(&self) -> &'static str {
        "debug-overlay-mode"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.overlay.mode",
            aliases: &[],
            category: "debug",
            help: "Set debug overlay layout mode.",
            usage: "debug.overlay.mode compact|full",
            examples: &["debug.overlay.mode compact", "debug.overlay.mode full"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.overlay.mode"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let overlay = match overlay_service(ctx) {
            Ok(service) => service,
            Err(result) => return result,
        };
        let Some(value) = command.args.first().map(String::as_str) else {
            return ConsoleCommandResult::error("usage: debug.overlay.mode compact|full");
        };

        let mode = match value {
            "compact" => DebugOverlayLayoutMode::Compact,
            "full" => DebugOverlayLayoutMode::Full,
            _ => {
                return ConsoleCommandResult::error(format!(
                    "invalid value `{value}`; expected compact or full"
                ));
            }
        };

        overlay.set_layout_mode(mode);
        ConsoleCommandResult::ok(format!("debug.overlay.mode {value}"))
    }
}
