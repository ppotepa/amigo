use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_runtime_control::RuntimeControlService;
use std::sync::Arc;

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
        if let (Some(control), Some(render_layers)) = (
            registry.resolve::<RuntimeControlService>(),
            registry.resolve::<RenderLayer2dSceneService>(),
        ) {
            control.register_provider(Arc::new(crate::RenderLayer2dControlProvider::new(
                render_layers,
            )));
        }
        amigo_scene::register_scene_reset_handler(registry, crate::Composition2dSceneResetHandler)?;
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            render_extractors.register(crate::COMPOSITION_2D_EXTRACTOR_ID);
        }
        register_domain_plugin(
            registry,
            COMPOSITION_2D_PLUGIN_LABEL,
            &["rendering_2d"],
            &[COMPOSITION_2D_CAPABILITY],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        plugin_scene_handlers.register(
            amigo_scene::RENDER_LAYER_2D_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Composition2dSceneCommandHandler),
        );
        plugin_scene_handlers.register(
            amigo_scene::VISUAL2D_SPATIAL_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Composition2dSceneCommandHandler),
        );
        plugin_scene_handlers.register(
            amigo_scene::LIGHT_ROUTE_2D_PLUGIN_SCENE_COMMAND_TYPE,
            std::sync::Arc::new(crate::scene_command::Composition2dSceneCommandHandler),
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
