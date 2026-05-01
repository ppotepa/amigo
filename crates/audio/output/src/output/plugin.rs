#[derive(Debug, Clone)]
pub struct AudioOutputDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct AudioOutputPlugin;

impl RuntimePlugin for AudioOutputPlugin {
    fn name(&self) -> &'static str {
        "amigo-audio-output"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(AudioOutputBackendService::default())?;
        registry.register(AudioOutputDomainInfo {
            crate_name: "amigo-audio-output",
            capability: "audio_output",
        })?;
        amigo_capabilities::register_domain_plugin(
            registry,
            "amigo-audio-output",
            &["audio_output"],
            &[],
            amigo_capabilities::DEFAULT_CAPABILITY_VERSION,
        )
    }
}

