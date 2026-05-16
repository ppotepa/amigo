use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::DepthMap2dSceneService;

pub struct DepthMap2dPlugin;

impl RuntimePlugin for DepthMap2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-depth-map"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(DepthMap2dSceneService::default())?;
        register_domain_plugin(
            registry,
            "amigo-2d-depth-map",
            &["rendering_2d", "camera_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::DepthMap2dSceneCommandHandler,
        );
        Ok(())
    }
}
