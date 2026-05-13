use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::LayeredImageSceneService;

pub struct LayeredImagePlugin;

impl RuntimePlugin for LayeredImagePlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-layered-image"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(LayeredImageSceneService::default())?;
        register_domain_plugin(
            registry,
            "amigo-2d-layered-image",
            &["rendering_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::LayeredImage2dSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::LayeredImage2dScriptCommandHandler,
        );
        Ok(())
    }
}

