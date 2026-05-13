use amigo_assets::AssetCatalog;
use amigo_core::RuntimeDiagnostics;

use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

pub(crate) struct CoreConsoleCommandHandler;

impl ConsoleCommandHandler for CoreConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "core-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "help",
                aliases: &["?", "commands"],
                category: "core",
                help: "Show available console commands.",
                usage: "help [command]",
                examples: &["help", "help render.stats"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "diagnostics",
                aliases: &["diag"],
                category: "core",
                help: "Show runtime diagnostics.",
                usage: "diagnostics",
                examples: &["diagnostics"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "clear",
                aliases: &["cls"],
                category: "core",
                help: "Clear console output.",
                usage: "clear",
                examples: &["clear"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(
            command.name.as_str(),
            "help" | "?" | "commands" | "clear" | "cls" | "echo" | "diagnostics" | "diag"
        )
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        match command.name.as_str() {
            "clear" | "cls" => {
                ctx.console.clear_output();
                ConsoleCommandResult::Silent
            }
            "echo" => ConsoleCommandResult::ok(command.args.join(" ")),
            "diagnostics" | "diag" => diagnostics(ctx),
            "help" | "?" | "commands" => help(ctx, command.args.first().map(String::as_str)),
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn help(ctx: &ConsoleCommandContext<'_>, command_name: Option<&str>) -> ConsoleCommandResult {
    let mut descriptors = ctx.registry.descriptors();
    descriptors.sort_by(|a, b| a.name.cmp(b.name));

    if let Some(command_name) = command_name {
        let Some(descriptor) = descriptors.iter().find(|descriptor| {
            descriptor.name == command_name || descriptor.aliases.contains(&command_name)
        }) else {
            return ConsoleCommandResult::error(format!("unknown command `{command_name}`"));
        };
        return ConsoleCommandResult::ok(format!(
            "{}\nusage: {}\ncategory: {}\nexamples: {}",
            descriptor.help,
            descriptor.usage,
            descriptor.category,
            descriptor.examples.join(", ")
        ));
    }

    let lines = descriptors
        .iter()
        .map(|descriptor| {
            let scope = if descriptor.dev_only { " [dev]" } else { "" };
            if descriptor.aliases.is_empty() {
                format!("{}{} - {}", descriptor.name, scope, descriptor.help)
            } else {
                format!(
                    "{}{} ({}) - {}",
                    descriptor.name,
                    scope,
                    descriptor.aliases.join(", "),
                    descriptor.help
                )
            }
        })
        .collect::<Vec<_>>();
    ConsoleCommandResult::ok(format!("commands:\n{}", lines.join("\n")))
}

fn diagnostics(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let diagnostics = match ctx.required::<RuntimeDiagnostics>() {
        Ok(diagnostics) => diagnostics,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };
    let assets = match ctx.required::<AssetCatalog>() {
        Ok(assets) => assets,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };

    ConsoleCommandResult::ok(format!(
        "window={} input={} render={} script={} loaded_mods={} assets_loaded={} assets_prepared={} assets_failed={} assets_pending={}",
        diagnostics.window_backend,
        diagnostics.input_backend,
        diagnostics.render_backend,
        diagnostics.script_backend,
        diagnostics.loaded_mods.len(),
        assets.loaded_assets().len(),
        assets.prepared_assets().len(),
        assets.failed_assets().len(),
        assets.pending_loads().len()
    ))
}



