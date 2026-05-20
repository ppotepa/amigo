use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
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
        amigo_scene::register_scene_reset_handler(registry, crate::AudioSceneResetHandler)?;
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
        )?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene_command::AudioSceneCommandHandler,
        );
        let script_handlers =
            registry.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            script_handlers.as_ref(),
            crate::script_command::AudioScriptCommandHandler,
        );
        Ok(())
    }
}
