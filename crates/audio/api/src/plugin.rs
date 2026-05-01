use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::{AudioCommandQueue, AudioSceneService, AudioStateService};

#[derive(Debug, Clone)]
pub struct AudioDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct AudioApiPlugin;

impl RuntimePlugin for AudioApiPlugin {
    fn name(&self) -> &'static str {
        "amigo-audio-api"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(AudioCommandQueue::default())?;
        registry.register(AudioSceneService::default())?;
        registry.register(AudioStateService::default())?;
        registry.register(AudioDomainInfo {
            crate_name: "amigo-audio-api",
            capability: "audio_api",
        })?;
        register_domain_plugin(
            registry,
            "amigo-audio-api",
            &["audio_api"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
