use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::{
    COMPOSITION_2D_CAPABILITY, COMPOSITION_2D_PLUGIN_LABEL, LightRoute2dSceneService,
    RenderLayer2dSceneService,
};

pub struct Composition2dPlugin;

impl RuntimePlugin for Composition2dPlugin {
    fn name(&self) -> &'static str {
        COMPOSITION_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(RenderLayer2dSceneService::default())?;
        registry.register(LightRoute2dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, crate::Composition2dSceneResetHandler)?;
        register_domain_plugin(
            registry,
            COMPOSITION_2D_PLUGIN_LABEL,
            &["rendering_2d"],
            &[COMPOSITION_2D_CAPABILITY],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::Composition2dSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::RenderLayer2dScriptCommandHandler,
        );
        Ok(())
    }
}
