use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

use crate::model::{PARTICLES_2D_CAPABILITY, PARTICLES_2D_PLUGIN_LABEL};
use crate::service::{Particle2dSceneService, ParticlePreset2dService};

pub struct Particle2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Particle2dPlugin;

impl RuntimePlugin for Particle2dPlugin {
    fn name(&self) -> &'static str {
        PARTICLES_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Particle2dSceneService::default())?;
        registry.register(ParticlePreset2dService::default())?;
        registry.register(Particle2dDomainInfo {
            crate_name: "amigo-2d-particles",
            capability: PARTICLES_2D_CAPABILITY,
        })?;
        register_domain_plugin(
            registry,
            PARTICLES_2D_PLUGIN_LABEL,
            &[PARTICLES_2D_CAPABILITY],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::Particles2dSceneCommandHandler,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "particles_2d",
            move |runtime| crate::tick_particles_2d_world(runtime, 1.0 / 60.0),
        );
        Ok(())
    }
}

