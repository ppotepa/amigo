//! Generated and procedural audio authoring utilities.
//! It parses authored graphs and produces sample data used by the mixer and output backends.

mod parser;
mod plugin;
mod render;
mod types;
#[cfg(test)]
mod tests;

pub use parser::parse_generated_audio_asset;
pub use plugin::GeneratedAudioPlugin;
pub use render::PcSpeakerGenerator;
pub use types::{
    DEFAULT_AUDIO_SAMPLE_RATE, Envelope, GeneratedAudioClip, GeneratedAudioDomainInfo,
    GeneratedAudioMode, GeneratedAudioParamMapping, GeneratedAudioParamSpec,
    PcSpeakerRealtimeState, PregeneratedGeneratedAudioClip, RealtimeGeneratedAudioClip,
    Tone, ToneSequence, Waveform, GeneratedAudioPcm,
};
