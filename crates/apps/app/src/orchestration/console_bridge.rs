use super::*;

pub(crate) fn handle_console_command(
    runtime: &Runtime,
    command: amigo_scripting_api::DevConsoleCommand,
) {
    crate::dev_console::dispatcher::dispatch_console_command(runtime, command);
}
