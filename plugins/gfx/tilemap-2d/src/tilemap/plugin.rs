use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use std::sync::Arc;

use super::service::TileMap2dSceneService;

#[derive(Debug, Clone)]
pub struct TileMap2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct TileMap2dPlugin;

impl RuntimePlugin for TileMap2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-tilemap-2d-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(TileMap2dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::TileMap2dSceneResetHandler,
        )?;
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::TileMap2dSceneComponentSpec,
        >(registry)?;
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_tilemap_2d_render_extractor_id(render_extractors.as_ref());
        }
        registry.register(TileMap2dDomainInfo {
            crate_name: "amigo-tilemap-2d-plugin",
            capability: "tilemap_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-tilemap-2d-plugin",
            &["tilemap_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::TileMap2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                amigo_scene::TILEMAP_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(super::scene_command::TileMap2dSceneCommandHandler),
            );
            plugin_scene_handlers.register(
                amigo_scene::TILEMAP_MARKER_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(super::scene_command::TileMap2dSceneCommandHandler),
            );
        }
        Ok(())
    }
}
