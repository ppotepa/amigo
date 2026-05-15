use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

pub struct Beacon2dPlugin;

impl RuntimePlugin for Beacon2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-lighting-beacon"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(crate::BeaconLight2dSceneService::default())?;
        register_domain_plugin(
            registry,
            "amigo-2d-lighting-beacon",
            &["beacon_2d"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::Beacon2dSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::Beacon2dScriptCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "beacon_2d",
            move |runtime| {
                let service = runtime.required::<crate::BeaconLight2dSceneService>()?;
                service.tick(amigo_session::simulation_delta_seconds(runtime));
                Ok(())
            },
        );
        Ok(())
    }
}
