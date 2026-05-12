use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

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
        )?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::Physics2dSceneCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "collision_events_2d",
            move |runtime| crate::tick_collision_events_2d(runtime),
        );
        Ok(())
    }
}
