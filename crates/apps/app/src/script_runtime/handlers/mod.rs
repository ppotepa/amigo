//! Script command handlers grouped by target subsystem.
//! They translate scripting requests into concrete runtime operations and diagnostics updates.

mod audio;
mod debug;
mod dev_shell;
mod render;

use super::{ScriptCommandHandlerRegistry, register_script_command_handler};

pub(super) fn register_builtin_script_command_handlers(
    registry: &mut ScriptCommandHandlerRegistry,
) {
    register_script_command_handler(registry, render::RenderScriptCommandHandler);
    register_script_command_handler(registry, audio::AudioScriptCommandHandler);
    register_script_command_handler(registry, debug::DebugScriptCommandHandler);
    register_script_command_handler(registry, dev_shell::DevShellScriptCommandHandler);
}



