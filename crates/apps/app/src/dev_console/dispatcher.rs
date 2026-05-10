use std::sync::Arc;

use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{DevConsoleCommand, DevConsoleOutputLevel, DevConsoleState};

use crate::runtime_context::required;

use super::model::ConsoleCommandResult;
use super::parser::parse_console_command;
use super::registry::ConsoleCommandRegistry;

pub(crate) struct ConsoleCommandContext<'a> {
    pub(crate) runtime: &'a Runtime,
    pub(crate) console: &'a DevConsoleState,
    pub(crate) registry: &'a ConsoleCommandRegistry,
}

impl<'a> ConsoleCommandContext<'a> {
    pub(crate) fn required<T: Send + Sync + 'static>(&self) -> AmigoResult<Arc<T>> {
        required::<T>(self.runtime)
    }
}

pub(crate) fn dispatch_console_command(runtime: &Runtime, command: DevConsoleCommand) {
    let Ok(console) = required::<DevConsoleState>(runtime) else {
        return;
    };
    let Ok(registry) = required::<ConsoleCommandRegistry>(runtime) else {
        console.write_line("error: console command registry is not registered");
        return;
    };

    console.record_command(command.line.clone());

    let Some(parsed) = parse_console_command(&command.line) else {
        return;
    };

    let ctx = ConsoleCommandContext {
        runtime,
        console: console.as_ref(),
        registry: registry.as_ref(),
    };
    let result = registry
        .handler_for(&parsed)
        .map(|handler| handler.handle(&ctx, parsed.clone()))
        .unwrap_or_else(|| ConsoleCommandResult::unknown(parsed.raw.clone()));

    match result {
        ConsoleCommandResult::Ok(message) => {
            console.write_line_with_level(message, DevConsoleOutputLevel::Success)
        }
        ConsoleCommandResult::Error(message) => {
            console.write_line_with_level(format!("error: {message}"), DevConsoleOutputLevel::Error)
        }
        ConsoleCommandResult::Unknown(raw) => console.write_line_with_level(
            format!("unknown command: {raw}"),
            DevConsoleOutputLevel::Warning,
        ),
        ConsoleCommandResult::Silent => {}
    }
}
