use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_runtime_control::RuntimeControlService;
use std::sync::Arc;

use super::{
    GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService,
    LIGHTING_2D_CAPABILITY, LIGHTING_2D_PLUGIN_LABEL,
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
        if let (Some(control), Some(light_groups)) = (
            registry.resolve::<RuntimeControlService>(),
            registry.resolve::<LightGroup2dSceneService>(),
        ) {
            control.register_provider(Arc::new(super::LightGroup2dControlProvider::new(
                light_groups,
            )));
        }
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::Lighting2dSceneResetHandler,
        )?;
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::GlobalLight2dSceneComponentSpec,
        >(registry)?;
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            render_extractors.register(super::LIGHTING_2D_EXTRACTOR_ID);
        }
        register_domain_plugin(
            registry,
            LIGHTING_2D_PLUGIN_LABEL,
            &[LIGHTING_2D_CAPABILITY],
            &["rendering_2d"],
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
