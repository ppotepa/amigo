use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::service::TileMap2dSceneService;

#[derive(Debug, Clone)]
pub struct TileMap2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct TileMap2dPlugin;

impl RuntimePlugin for TileMap2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-tilemap"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(TileMap2dSceneService::default())?;
        registry.register(TileMap2dDomainInfo {
            crate_name: "amigo-2d-tilemap",
            capability: "tilemap_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-2d-tilemap",
            &["tilemap_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::TileMap2dSceneCommandHandler,
        );
        Ok(())
    }
}
