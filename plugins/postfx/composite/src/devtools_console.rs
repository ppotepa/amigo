use amigo_devtools::{
    ConsoleArgKind, ConsoleArgSpec, ConsoleCommandDescriptor, ConsoleCommandForm,
    ConsoleCommandResult, ConsoleCommandSchema, DevConsoleCommandContext, ParsedConsoleCommand,
    RuntimeConsoleCommandHandler,
};

const POSTFX_ITEMS_NO_TARGET_ACTIONS: &[&str] = &["list", "count", "clear"];
const POSTFX_ITEMS_ADD_ACTIONS: &[&str] = &["add"];
const POSTFX_ITEMS_INDEX_ACTIONS: &[&str] = &["inspect", "remove"];
const POSTFX_ITEMS_NO_TARGET_ARGS: &[ConsoleArgSpec] = &[ConsoleArgSpec::required(
    "action",
    ConsoleArgKind::Literal(POSTFX_ITEMS_NO_TARGET_ACTIONS),
)];
const POSTFX_ITEMS_ADD_ARGS: &[ConsoleArgSpec] = &[
    ConsoleArgSpec::required("action", ConsoleArgKind::Literal(POSTFX_ITEMS_ADD_ACTIONS)),
    ConsoleArgSpec::required("kind", ConsoleArgKind::PostFxKind),
];
const POSTFX_ITEMS_INDEX_ARGS: &[ConsoleArgSpec] = &[
    ConsoleArgSpec::required(
        "action",
        ConsoleArgKind::Literal(POSTFX_ITEMS_INDEX_ACTIONS),
    ),
    ConsoleArgSpec::required("index", ConsoleArgKind::PostFxIndex),
];
const POSTFX_ITEMS_FORMS: &[ConsoleCommandForm] = &[
    ConsoleCommandForm {
        usage: "postfx.items <list|count|clear>",
        args: POSTFX_ITEMS_NO_TARGET_ARGS,
    },
    ConsoleCommandForm {
        usage: "postfx.items add <kind>",
        args: POSTFX_ITEMS_ADD_ARGS,
    },
    ConsoleCommandForm {
        usage: "postfx.items <inspect|remove> <index>",
        args: POSTFX_ITEMS_INDEX_ARGS,
    },
];

pub struct PostFxConsoleCommandHandler;

impl RuntimeConsoleCommandHandler for PostFxConsoleCommandHandler {
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

    fn schemas(&self) -> Vec<ConsoleCommandSchema> {
        vec![ConsoleCommandSchema {
            command_name: "postfx.items",
            aliases: &[],
            forms: POSTFX_ITEMS_FORMS,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "postfx" || command.name.starts_with("postfx.")
    }

    fn handle(
        &self,
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let post_fx = match ctx.required::<crate::PostFx2dService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match crate::handle_post_fx_dev_console_command(
            crate::PostFxDevConsoleCommandContext {
                post_fx_service: post_fx.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            crate::PostFxDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            crate::PostFxDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            crate::PostFxDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}
