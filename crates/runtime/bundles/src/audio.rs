use amigo_audio_api::AudioApiPlugin;
use amigo_audio_generated::GeneratedAudioPlugin;
use amigo_audio_mixer::AudioMixerPlugin;
use amigo_audio_output::AudioOutputPlugin;
use amigo_core::AmigoResult;
use amigo_runtime::{PluginBundle, RuntimeBuilder};
use amigo_session::RuntimeSession;

pub use amigo_audio_api::{
    AudioClip, AudioClipKey, AudioCommand, AudioCommandQueue, AudioPlaybackMode, AudioSceneService,
    AudioScriptCommandContext, AudioScriptCommandOutcome, AudioStateService,
    handle_audio_script_command,
};
pub use amigo_audio_output::{AudioOutputBackendService, AudioOutputStartStatus};

pub struct AudioRuntimeBundle;

impl PluginBundle for AudioRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-audio-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(AudioApiPlugin)?
            .with_plugin(GeneratedAudioPlugin)?
            .with_plugin(AudioMixerPlugin)?
            .with_plugin(AudioOutputPlugin)
    }
}

pub fn register_audio_runtime_capabilities(session: &mut RuntimeSession) {
    amigo_audio_api::register_audio_runtime_capabilities(session);
    amigo_audio_mixer::register_audio_mixer_runtime_capabilities(session);
}
