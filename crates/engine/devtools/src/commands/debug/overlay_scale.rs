use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use super::shared::overlay_service;

pub(crate) struct DebugOverlayScaleCommandHandler;

impl ConsoleCommandHandler for DebugOverlayScaleCommandHandler {
    fn name(&self) -> &'static str {
        "debug-overlay-scale"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "debug.overlay.scale",
            aliases: &[],
            category: "debug",
            help: "Set debug overlay scale.",
            usage: "debug.overlay.scale <0.5..3.0>",
            examples: &["debug.overlay.scale 1.25", "debug.overlay.scale 0.8"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "debug.overlay.scale"
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
        let Some(value) = command.args.first() else {
            return ConsoleCommandResult::error("usage: debug.overlay.scale <0.5..3.0>");
        };
        let Ok(scale) = value.parse::<f32>() else {
            return ConsoleCommandResult::error(format!("invalid scale `{value}`"));
        };

        overlay.set_scale(scale);
        let actual = overlay.snapshot().settings.scale;
        ConsoleCommandResult::ok(format!("debug.overlay.scale {:.2}", actual))
    }
}
