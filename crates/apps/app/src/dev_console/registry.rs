use std::sync::Arc;

pub(crate) use amigo_devtools::{
    DevConsoleCommandContext as ConsoleCommandContext,
    RuntimeConsoleCommandHandler as ConsoleCommandHandler,
    RuntimeConsoleCommandRegistry as ConsoleCommandRegistry,
};

pub(crate) fn register_console_command_handler<H>(registry: &ConsoleCommandRegistry, handler: H)
where
    H: ConsoleCommandHandler + 'static,
{
    let _ = handler.name();
    registry.register_arc(Arc::new(handler));
}

