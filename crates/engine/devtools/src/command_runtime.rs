use std::sync::Arc;

use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{DevConsoleCommand, DevConsoleOutputLevel, DevConsoleState};

use crate::{
    ConsoleCommandDescriptor, ConsoleCommandRegistry, ConsoleCommandResult, ConsoleCommandSpec,
    ParsedConsoleCommand, parse_console_command,
};

pub trait RuntimeConsoleCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor>;

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool;

    fn handle(
        &self,
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult;
}

impl<T> ConsoleCommandSpec for T
where
    T: RuntimeConsoleCommandHandler + ?Sized,
{
    fn name(&self) -> &'static str {
        RuntimeConsoleCommandHandler::name(self)
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        RuntimeConsoleCommandHandler::descriptors(self)
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        RuntimeConsoleCommandHandler::can_handle(self, command)
    }
}

pub type RuntimeConsoleCommandRegistry = ConsoleCommandRegistry<dyn RuntimeConsoleCommandHandler>;

pub fn register_runtime_console_command_handler<H>(
    registry: &RuntimeConsoleCommandRegistry,
    handler: H,
) where
    H: RuntimeConsoleCommandHandler + 'static,
{
    registry.register_arc(Arc::new(handler));
}

pub struct DevConsoleCommandContext<'a> {
    pub runtime: &'a Runtime,
    pub console: &'a DevConsoleState,
    pub registry: &'a RuntimeConsoleCommandRegistry,
}

impl<'a> DevConsoleCommandContext<'a> {
    pub fn required<T: Send + Sync + 'static>(&self) -> AmigoResult<Arc<T>> {
        self.runtime.required::<T>()
    }
}

pub fn dispatch_console_command(runtime: &Runtime, command: DevConsoleCommand) {
    let Ok(console) = runtime.required::<DevConsoleState>() else {
        return;
    };
    let Ok(registry) = runtime.required::<RuntimeConsoleCommandRegistry>() else {
        console.write_line("error: console command registry is not registered");
        return;
    };

    console.record_command(command.line.clone());

    let Some(parsed) = parse_console_command(&command.line) else {
        return;
    };

    let ctx = DevConsoleCommandContext {
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

