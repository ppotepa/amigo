use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

pub struct DevtoolsPlugin;

impl RuntimePlugin for DevtoolsPlugin {
    fn name(&self) -> &'static str {
        "amigo-devtools"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(crate::DebugOverlayService::default())?;
        let emergency_notices = crate::EmergencyNoticeService::default();
        if let Some(run_log) = registry.resolve::<amigo_scripting_api::RunLogService>() {
            emergency_notices.attach_run_log(run_log);
        }
        registry.register(emergency_notices)?;

        if let Some(console_registry) = registry.resolve::<crate::RuntimeConsoleCommandRegistry>() {
            crate::commands::register_builtin_console_commands(console_registry.as_ref());
        } else {
            let console_registry = crate::RuntimeConsoleCommandRegistry::default();
            crate::commands::register_builtin_console_commands(&console_registry);
            registry.register(console_registry)?;
        }
        if !registry.has::<crate::ConsoleCompletionState>() {
            registry.register(crate::ConsoleCompletionState::default())?;
        }
        if !registry.has::<crate::ConsoleCompletionProviderRegistry>() {
            registry.register(crate::ConsoleCompletionProviderRegistry::default())?;
        }

        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::DebugScriptCommandHandler,
        );
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::DevShellScriptCommandHandler,
        );

        Ok(())
    }
}
