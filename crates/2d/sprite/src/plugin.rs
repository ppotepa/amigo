use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::service::SpriteSceneService;

#[derive(Debug, Clone)]
pub struct SpriteDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct SpritePlugin;

impl RuntimePlugin for SpritePlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-sprite"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SpriteSceneService::default())?;
        registry.register(SpriteDomainInfo {
            crate_name: "amigo-2d-sprite",
            capability: "rendering_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-2d-sprite",
            &["rendering_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
