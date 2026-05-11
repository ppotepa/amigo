use crate::debug_overlay::DebugOverlayCorner;
use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

use super::shared::overlay_service;

pub(crate) struct DebugOverlayCornerCommandHandler;

impl ConsoleCommandHandler for DebugOverlayCornerCommandHandler {
    fn name(&self) -> &'static str {
        "debug-overlay-corner"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.overlay.corner",
            aliases: &[],
            category: "debug",
            help: "Set debug overlay corner.",
            usage: "debug.overlay.corner tl|tr|bl|br",
            examples: &["debug.overlay.corner tr", "debug.overlay.corner bl"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.overlay.corner"
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
            return ConsoleCommandResult::error("usage: debug.overlay.corner tl|tr|bl|br");
        };

        let corner = match value {
            "tl" => DebugOverlayCorner::TopLeft,
            "tr" => DebugOverlayCorner::TopRight,
            "bl" => DebugOverlayCorner::BottomLeft,
            "br" => DebugOverlayCorner::BottomRight,
            _ => {
                return ConsoleCommandResult::error(format!(
                    "invalid value `{value}`; expected tl, tr, bl, or br"
                ));
            }
        };

        overlay.set_corner(corner);
        ConsoleCommandResult::ok(format!("debug.overlay.corner {value}"))
    }
}
