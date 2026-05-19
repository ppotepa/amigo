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
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            super::script_command::LayeredImage2dScriptCommandHandler,
        );
        Ok(())
    }
}
