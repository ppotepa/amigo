use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use super::service::SpriteSceneService;

#[derive(Debug, Clone)]
pub struct SpriteDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct SpritePlugin;

impl RuntimePlugin for SpritePlugin {
    fn name(&self) -> &'static str {
        "amigo-sprite-2d-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SpriteSceneService::default())?;
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::Sprite2dSceneResetHandler,
        )?;
        if let Some(metadata) = registry.resolve::<amigo_scene::ComponentMetadataProviderRegistry>()
        {
            metadata.register(crate::scene::Sprite2dComponentMetadataProvider);
        }
        if let Some(schemas) = registry.resolve::<amigo_scene::ComponentSchemaRegistry>() {
            schemas.register_provider(&crate::scene::descriptors::Sprite2dSceneDescriptorProvider);
            schemas.register_schema_provider(crate::scene::Sprite2dSceneSchemaProvider);
        }
        if let Some(hydrators) = registry.resolve::<amigo_scene::ComponentHydratorRegistry>() {
            hydrators.register(crate::scene::hydration::Sprite2dComponentHydrator);
            hydrators.register_plugin(crate::scene::hydration::Sprite2dPluginComponentHydrator);
        }
        if let Some(graph_providers) =
            registry.resolve::<amigo_scene::ComponentGraphProviderRegistry>()
        {
            graph_providers.register(crate::scene::Sprite2dPluginGraphProvider);
        }
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_sprite_2d_render_extractor_id(render_extractors.as_ref());
        }
        registry.register(SpriteDomainInfo {
            crate_name: "amigo-sprite-2d-plugin",
            capability: "rendering_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-sprite-2d-plugin",
            &["rendering_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::Sprite2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::PluginSceneCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                "amigo.gfx.sprite-2d.scene-command.Sprite2D",
                std::sync::Arc::new(super::scene_command::Sprite2dSceneCommandHandler),
            );
        }
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            super::script_command::Sprite2dScriptCommandHandler,
        );
        Ok(())
    }
}
