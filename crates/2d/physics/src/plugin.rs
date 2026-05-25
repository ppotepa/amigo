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
        amigo_scene::register_scene_reset_handler(registry, crate::Physics2dSceneResetHandler)?;
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
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        for command_type in [
            amigo_scene::KINEMATIC_BODY_2D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::AABB_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::STATIC_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::CIRCLE_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::TRIGGER_2D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::COLLISION_EVENT_RULE_2D_PLUGIN_SCENE_COMMAND_TYPE,
        ] {
            plugin_scene_handlers.register(
                command_type,
                std::sync::Arc::new(crate::scene_command::Physics2dSceneCommandHandler),
            );
        }
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "collision_events_2d",
            move |runtime| crate::tick_collision_events_2d(runtime),
        );
        Ok(())
    }
}
