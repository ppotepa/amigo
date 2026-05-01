use std::collections::BTreeMap;

pub const DEFAULT_AUDIO_SAMPLE_RATE: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Square,
    Sine,
    Triangle,
    Noise,
}

impl Waveform {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "square" => Ok(Self::Square),
            "sine" => Ok(Self::Sine),
            "triangle" => Ok(Self::Triangle),
            "noise" => Ok(Self::Noise),
            other => Err(format!("unsupported waveform `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedAudioMode {
    Pregenerated,
    Realtime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub attack_ms: u32,
    pub release_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tone {
    pub wave: Waveform,
    pub frequency: f32,
    pub duration_ms: u32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToneSequence {
    pub tones: Vec<Tone>,
}

impl ToneSequence {
    pub fn total_duration_ms(&self) -> u32 {
        self.tones.iter().map(|tone| tone.duration_ms).sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedAudioParamSpec {
    pub default: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedAudioParamMapping {
    pub from_param: String,
    pub near_value: f32,
    pub far_value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PregeneratedGeneratedAudioClip {
    pub generator: String,
    pub sample_rate: u32,
    pub sequence: ToneSequence,
    pub envelope: Envelope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RealtimeGeneratedAudioClip {
    pub generator: String,
    pub sample_rate: u32,
    pub wave: Waveform,
    pub volume: f32,
    pub params: BTreeMap<String, GeneratedAudioParamSpec>,
    pub interval_ms: GeneratedAudioParamMapping,
    pub frequency: GeneratedAudioParamMapping,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedAudioClip {
    Pregenerated(PregeneratedGeneratedAudioClip),
    Realtime(RealtimeGeneratedAudioClip),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedAudioPcm {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PcSpeakerRealtimeState {
    pub phase: f32,
    pub interval_remaining_ms: f32,
    pub beep_remaining_ms: f32,
    pub noise_state: u32,
}

impl Default for PcSpeakerRealtimeState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            interval_remaining_ms: 0.0,
            beep_remaining_ms: 0.0,
            noise_state: 0x1234_5678,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedAudioDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}
