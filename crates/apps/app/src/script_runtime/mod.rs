//! App-side scripting runtime integration.
//! This module now delegates script command dispatch to engine-registered
//! `RuntimeScriptCommandHandler` implementations.

use super::*;
use amigo_runtime::{HandlerDispatcher, Runtime, RuntimePlugin};
use amigo_session::RuntimeSession;

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
pub(crate) use test_helpers::*;

pub(crate) struct ScriptCommandRuntimePlugin;

impl RuntimePlugin for ScriptCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-script-command-registry"
    }

    fn register(&self, _services: &mut ServiceRegistry) -> AmigoResult<()> {
        Ok(())
    }
}

pub(crate) fn dispatch_script_command_with_runtime(
    runtime: &Runtime,
    command: ScriptCommand,
) -> AmigoResult<()> {
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let handlers =
        runtime.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;

    let result = HandlerDispatcher::new(handlers).dispatch_first(|handler| {
        handler
            .can_handle(&command)
            .then(|| handler.handle(runtime, command.clone()))
    });

    if let Some(result) = result {
        result?;
    } else {
        dev_console_state.write_line(format!(
            "unhandled script command: {}",
            crate::app_helpers::format_script_command(&command)
        ));
    }

    Ok(())
}

pub(crate) fn dispatch_script_command_for_session(
    session: &RuntimeSession,
    command: ScriptCommand,
) -> AmigoResult<()> {
    let command_label = crate::app_helpers::format_script_command(&command);
    session.begin_script_command_dispatch(command_label.clone());
    if let Err(error) = dispatch_script_command_with_runtime(session.runtime(), command) {
        session.mark_script_dispatch_error(command_label, error.to_string());
        return Err(error);
    }

    session.complete_script_command_dispatch();
    Ok(())
}
