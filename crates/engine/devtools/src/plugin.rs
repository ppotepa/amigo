use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

pub struct DevtoolsPlugin;

impl RuntimePlugin for DevtoolsPlugin {
    fn name(&self) -> &'static str {
        "amigo-devtools"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(crate::DebugOverlayService::default())?;

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
