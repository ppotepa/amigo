use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone)]
pub struct VectorDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Vector2dPlugin;

impl RuntimePlugin for Vector2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-vector-2d-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(super::service::VectorSceneService::default())?;
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::Vector2dSceneResetHandler,
        )?;
        if let Some(metadata) = registry.resolve::<amigo_scene::ComponentMetadataProviderRegistry>()
        {
            metadata.register(crate::scene::Vector2dComponentMetadataProvider);
        }
        if let Some(schemas) = registry.resolve::<amigo_scene::ComponentSchemaRegistry>() {
            schemas.register_descriptor(crate::scene::vector_2d_scene_descriptor());
            schemas.register_schema_provider(crate::scene::Vector2dSceneSchemaProvider);
        }
        if let Some(hydrators) = registry.resolve::<amigo_scene::ComponentHydratorRegistry>() {
            hydrators.register(crate::scene::hydration::VectorShape2dComponentHydrator);
            hydrators.register_plugin(crate::scene::hydration::Vector2dPluginComponentHydrator);
        }
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_vector_2d_render_extractor_id(render_extractors.as_ref());
        }
        registry.register(VectorDomainInfo {
            crate_name: "amigo-vector-2d-plugin",
            capability: "vector_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-vector-2d-plugin",
            &["vector_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::Vector2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::PluginSceneCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                "amigo.gfx.vector-2d.scene-command.VectorShape2D",
                std::sync::Arc::new(super::scene_command::Vector2dSceneCommandHandler),
            );
        }
        Ok(())
    }
}
