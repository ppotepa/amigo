use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};

#[derive(Debug, Clone)]
pub struct VectorDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Vector2dPlugin;

impl RuntimePlugin for Vector2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-vector"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(crate::service::VectorSceneService::default())?;
        registry.register(VectorDomainInfo {
            crate_name: "amigo-2d-vector",
            capability: "vector_2d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-2d-vector",
            &["vector_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
