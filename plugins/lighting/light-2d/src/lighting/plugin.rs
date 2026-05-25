use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use std::sync::Arc;

use super::{
    GlobalLight2dSceneService, LIGHTING_2D_CAPABILITY, LIGHTING_2D_PLUGIN_LABEL,
    LightGroup2dSceneService, LightMap2dSceneService,
};

pub struct Lighting2dPlugin;

impl RuntimePlugin for Lighting2dPlugin {
    fn name(&self) -> &'static str {
        LIGHTING_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(GlobalLight2dSceneService::default())?;
        registry.register(LightMap2dSceneService::default())?;
        registry.register(LightGroup2dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::Lighting2dSceneResetHandler,
        )?;
        if let Some(schemas) = registry.resolve::<amigo_scene::ComponentSchemaRegistry>() {
            schemas.register_provider(&crate::scene::Lighting2dSceneDescriptorProvider);
            schemas.register_schema_provider(crate::scene::GlobalLight2dSceneSchemaProvider);
        }
        if let Some(hydrators) = registry.resolve::<amigo_scene::ComponentHydratorRegistry>() {
            hydrators.register(crate::scene::GlobalLight2dComponentHydrator);
            hydrators.register_plugin(crate::scene::GlobalLight2dPluginComponentHydrator);
        }
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            render_extractors.register(super::LIGHTING_2D_EXTRACTOR_ID);
        }
        register_domain_plugin(
            registry,
            LIGHTING_2D_PLUGIN_LABEL,
            &["rendering_2d"],
            &[LIGHTING_2D_CAPABILITY],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::Lighting2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                amigo_scene::GLOBAL_LIGHT_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(super::scene_command::Lighting2dSceneCommandHandler),
            );
            plugin_scene_handlers.register(
                amigo_scene::LIGHTMAP_2D_SOURCE_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(super::scene_command::Lighting2dSceneCommandHandler),
            );
            plugin_scene_handlers.register(
                amigo_scene::LIGHT_GROUP_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(super::scene_command::Lighting2dSceneCommandHandler),
            );
        }
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            super::script_command::Lighting2dScriptCommandHandler,
        );
        if !registry.has::<amigo_devtools::RuntimeConsoleCommandRegistry>() {
            registry.register(amigo_devtools::RuntimeConsoleCommandRegistry::default())?;
        }
        amigo_devtools::register_runtime_console_command_handler(
            registry
                .required::<amigo_devtools::RuntimeConsoleCommandRegistry>()?
                .as_ref(),
            super::dev_console::Lighting2dConsoleCommandHandler,
        );
        Ok(())
    }
}
