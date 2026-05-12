use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::service::register_ui_services;

pub struct UiPlugin;

impl RuntimePlugin for UiPlugin {
    fn name(&self) -> &'static str {
        "amigo-ui"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_ui_services(registry)?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::UiSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::UiScriptCommandHandler,
        );
        Ok(())
    }
}
