use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};

use crate::service::Physics2dSceneService;

#[derive(Debug, Clone)]
pub struct Physics2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Physics2dPlugin;

impl RuntimePlugin for Physics2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-physics"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Physics2dSceneService::default())?;
        registry.register(Physics2dDomainInfo {
            crate_name: "amigo-2d-physics",
            capability: "physics_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-2d-physics",
            &["physics_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
