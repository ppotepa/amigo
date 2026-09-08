use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use std::sync::Arc;

use crate::DepthMap2dSceneService;

pub struct DepthMap2dPlugin;

impl RuntimePlugin for DepthMap2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-focus-depth-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(DepthMap2dSceneService::default())?;
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::DepthMap2dSceneComponentSpec,
        >(registry)?;
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::DepthAuxMap2dSceneComponentSpec,
        >(registry)?;
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_depth_map_2d_render_extractor_id(render_extractors.as_ref());
        }
        register_domain_plugin(
            registry,
            "amigo-focus-depth-plugin",
            &["camera_2d"],
            &["rendering_2d"],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::DepthMap2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                amigo_scene::DEPTH_MAP_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(crate::DepthMap2dSceneCommandHandler),
            );
            plugin_scene_handlers.register(
                amigo_scene::DEPTH_AUX_MAP_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(crate::DepthMap2dSceneCommandHandler),
            );
        }
        Ok(())
    }
}
