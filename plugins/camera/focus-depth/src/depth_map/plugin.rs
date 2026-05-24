use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::DepthMap2dSceneService;

pub struct DepthMap2dPlugin;

impl RuntimePlugin for DepthMap2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-focus-depth-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(DepthMap2dSceneService::default())?;
        if let Some(schemas) = registry.resolve::<amigo_scene::ComponentSchemaRegistry>() {
            schemas.register_provider(&crate::scene::FocusDepthSceneDescriptorProvider);
            schemas.register_schema_provider(crate::scene::DepthMap2dSceneSchemaProvider);
            schemas.register_schema_provider(crate::scene::DepthAuxMap2dSceneSchemaProvider);
        }
        if let Some(hydrators) = registry.resolve::<amigo_scene::ComponentHydratorRegistry>() {
            hydrators.register(crate::scene::DepthMap2dComponentHydrator);
            hydrators.register_plugin(crate::scene::DepthMap2dPluginComponentHydrator);
            hydrators.register_plugin(crate::scene::DepthAuxMap2dPluginComponentHydrator);
        }
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_depth_map_2d_render_extractor_id(render_extractors.as_ref());
        }
        register_domain_plugin(
            registry,
            "amigo-focus-depth-plugin",
            &["rendering_2d", "camera_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::DepthMap2dSceneCommandHandler,
        );
        Ok(())
    }
}
