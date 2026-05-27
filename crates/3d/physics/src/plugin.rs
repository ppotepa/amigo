use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

use crate::service::Physics3dSceneService;

#[derive(Debug, Clone)]
pub struct Physics3dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Physics3dPlugin;

impl RuntimePlugin for Physics3dPlugin {
    fn name(&self) -> &'static str {
        "amigo-3d-physics"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Physics3dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(registry, crate::Physics3dSceneResetHandler)?;
        registry.register(Physics3dDomainInfo {
            crate_name: "amigo-3d-physics",
            capability: "physics_3d",
        })?;
        register_domain_plugin(
            registry,
            "amigo-3d-physics",
            &["physics_3d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let plugin_scene_handlers =
            registry.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        for command_type in [
            amigo_scene::PHYSICS_WORLD_3D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::RIGID_BODY_3D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::STATIC_BOX_COLLIDER_3D_PLUGIN_SCENE_COMMAND_TYPE,
            amigo_scene::PHYSICS_SPAWNER_3D_PLUGIN_SCENE_COMMAND_TYPE,
        ] {
            plugin_scene_handlers.register(
                command_type,
                std::sync::Arc::new(crate::scene_command::Physics3dSceneCommandHandler),
            );
        }
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::Physics3dScriptCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "physics_3d_step",
            move |runtime| crate::tick_physics_3d(runtime),
        );
        Ok(())
    }
}
