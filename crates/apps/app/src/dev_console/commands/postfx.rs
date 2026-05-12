use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

pub(crate) struct PostFxConsoleCommandHandler;

impl ConsoleCommandHandler for PostFxConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "postfx-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "postfx.cert",
                aliases: &[],
                category: "render",
                help: "Show LensDroplets2D certification reports.",
                usage: "postfx.cert",
                examples: &["postfx.cert"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "postfx.stats",
                aliases: &[],
                category: "render",
                help: "Show active 2D post-fx stack stats.",
                usage: "postfx.stats",
                examples: &["postfx.stats"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "postfx.cert" || command.name == "postfx.stats"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let post_fx = match ctx.required::<amigo_2d_post_fx::PostFx2dService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match amigo_2d_post_fx::handle_post_fx_dev_console_command(
            amigo_2d_post_fx::PostFxDevConsoleCommandContext {
                post_fx_service: post_fx.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            amigo_2d_post_fx::PostFxDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            amigo_2d_post_fx::PostFxDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            amigo_2d_post_fx::PostFxDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}
