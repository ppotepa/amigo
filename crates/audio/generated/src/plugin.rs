use amigo_capabilities::{register_domain_plugin, DEFAULT_CAPABILITY_VERSION};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::types::GeneratedAudioDomainInfo;

pub struct GeneratedAudioPlugin;

impl RuntimePlugin for GeneratedAudioPlugin {
    fn name(&self) -> &'static str {
        "amigo-audio-generated"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(GeneratedAudioDomainInfo {
            crate_name: "amigo-audio-generated",
            capability: "generated_audio",
        })?;
        register_domain_plugin(
            registry,
            "amigo-audio-generated",
            &["generated_audio"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )
    }
}
