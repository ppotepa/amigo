use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_runtime_control::RuntimeControlService;
use std::sync::Arc;

use crate::LayeredImageSceneService;

pub struct LayeredImagePlugin;

impl RuntimePlugin for LayeredImagePlugin {
    fn name(&self) -> &'static str {
        "amigo-layered-image-2d-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(LayeredImageSceneService::default())?;
        if !registry.has::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>() {
            registry.register(
                amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry::default(),
            )?;
        }
        if let Some(editor_apply) =
            registry.resolve::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>()
        {
            editor_apply.register(crate::LayeredImageEditorRuntimeApplyProvider);
        }
        amigo_scene::register_scene_reset_handler(
            registry,
            super::reset::LayeredImage2dSceneResetHandler,
        )?;
        if let Some(metadata) = registry.resolve::<amigo_scene::ComponentMetadataProviderRegistry>()
        {
            metadata.register(crate::scene::LayeredImage2dComponentMetadataProvider);
        }
        if let Some(schemas) = registry.resolve::<amigo_scene::ComponentSchemaRegistry>() {
            schemas.register_provider(&crate::scene::LayeredImage2dSceneDescriptorProvider);
            schemas.register_schema_provider(crate::scene::LayeredImage2dSceneSchemaProvider);
        }
        if let Some(hydrators) = registry.resolve::<amigo_scene::ComponentHydratorRegistry>() {
            hydrators.register(crate::scene::LayeredImage2dComponentHydrator);
            hydrators.register_plugin(crate::scene::LayeredImage2dPluginComponentHydrator);
        }
        if let Some(render_extractors) =
            registry.resolve::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()
        {
            crate::render::register_layered_image_2d_render_extractor_id(
                render_extractors.as_ref(),
            );
        }
        if let (Some(control), Some(layered), Some(assets)) = (
            registry.resolve::<RuntimeControlService>(),
            registry.resolve::<LayeredImageSceneService>(),
            registry.resolve::<amigo_assets::AssetCatalog>(),
        ) {
            control.register_provider(Arc::new(crate::LayeredImage2dControlProvider::new(
                layered, assets,
            )));
        }
        register_domain_plugin(
            registry,
            "amigo-layered-image-2d-plugin",
            &["rendering_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            super::scene_command::LayeredImage2dSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
            plugin_scene_handlers.register(
                amigo_scene::LAYERED_IMAGE_2D_PLUGIN_SCENE_COMMAND_TYPE,
                Arc::new(super::scene_command::LayeredImage2dSceneCommandHandler),
            );
        }
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            super::script_command::LayeredImage2dScriptCommandHandler,
        );
        if !registry.has::<amigo_devtools::RuntimeConsoleCommandRegistry>() {
            registry.register(amigo_devtools::RuntimeConsoleCommandRegistry::default())?;
        }
        amigo_devtools::register_runtime_console_command_handler(
            registry
                .required::<amigo_devtools::RuntimeConsoleCommandRegistry>()?
                .as_ref(),
            super::dev_console::LayeredImageConsoleCommandHandler,
        );
        Ok(())
    }
}
