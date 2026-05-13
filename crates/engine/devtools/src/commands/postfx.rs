use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

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
                examples: &["postfx.cert", "postfx cert"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "postfx.crt",
                aliases: &[],
                category: "render",
                help: "Inspect or tune CRT 2D post-fx parameters.",
                usage: "postfx.crt [field value|field=value ...]",
                examples: &["postfx.crt", "postfx.crt scanlines 0.18", "postfx.crt rgb_split=1.5"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "postfx.dirty_bloom",
                aliases: &[],
                category: "render",
                help: "Inspect or tune DirtyBloom2D post-fx parameters.",
                usage: "postfx.dirty_bloom [field value|field=value ...]",
                examples: &[
                    "postfx.dirty_bloom",
                    "postfx.dirty_bloom strength 0.9",
                    "postfx.dirty_bloom threshold=0.58 dirty_noise=0.25",
                ],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "postfx.stats",
                aliases: &[],
                category: "render",
                help: "Show active 2D post-fx stack stats.",
                usage: "postfx.stats",
                examples: &["postfx.stats", "postfx stats"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "postfx.items",
                aliases: &[],
                category: "render",
                help: "List, add, clear, or inspect active post-fx stack items.",
                usage: "postfx.items <list|count|add|clear|inspect> [args...]",
                examples: &[
                    "postfx.items list",
                    "postfx.items count",
                    "postfx.items add blur",
                    "postfx.items inspect 0",
                    "postfx.items clear",
                ],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "postfx" || command.name.starts_with("postfx.")
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



