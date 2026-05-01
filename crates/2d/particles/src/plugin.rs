use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};

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
        )
    }
}
