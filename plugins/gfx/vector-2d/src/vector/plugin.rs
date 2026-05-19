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
        Ok(())
    }
}
