use std::collections::BTreeMap;
use std::f32::consts::PI;

use amigo_assets::{PreparedAsset, PreparedAssetKind};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

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

pub struct PcSpeakerGenerator;

impl PcSpeakerGenerator {
    pub fn generate_clip(clip: &PregeneratedGeneratedAudioClip) -> GeneratedAudioPcm {
        let mut samples = Vec::new();
        let sample_rate = clip.sample_rate.max(1);

        for tone in &clip.sequence.tones {
            let tone_sample_count =
                milliseconds_to_sample_count(tone.duration_ms as f32, sample_rate);
            if tone_sample_count == 0 {
                continue;
            }

            let attack_samples =
                milliseconds_to_sample_count(clip.envelope.attack_ms as f32, sample_rate);
            let release_samples =
                milliseconds_to_sample_count(clip.envelope.release_ms as f32, sample_rate);
            let mut phase = 0.0f32;
            let frequency = tone.frequency.max(0.0);

            for sample_index in 0..tone_sample_count {
                let amplitude = envelope_amplitude(
                    sample_index,
                    tone_sample_count,
                    attack_samples,
                    release_samples,
                ) * tone.volume.clamp(0.0, 1.0);
                let sample = waveform_sample(tone.wave, phase, None) * amplitude;
                samples.push(sample);

                if frequency > 0.0 {
                    phase = (phase + frequency / sample_rate as f32).fract();
                }
            }
        }

        GeneratedAudioPcm {
            sample_rate,
            samples,
        }
    }

    pub fn generate_realtime_frame(
        clip: &RealtimeGeneratedAudioClip,
        params: Option<&BTreeMap<String, f32>>,
        state: &mut PcSpeakerRealtimeState,
        frame_sample_count: usize,
    ) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_sample_count);
        let sample_rate = clip.sample_rate.max(1);
        let sample_dt_ms = 1000.0 / sample_rate as f32;
        let interval_ms = mapped_value(&clip.interval_ms, &clip.params, params).max(16.0);
        let frequency = mapped_value(&clip.frequency, &clip.params, params).max(1.0);
        let beep_duration_ms = interval_ms.clamp(24.0, 60.0) * 0.45;

        for _ in 0..frame_sample_count {
            if state.beep_remaining_ms <= 0.0 {
                state.interval_remaining_ms -= sample_dt_ms;
                if state.interval_remaining_ms <= 0.0 {
                    state.interval_remaining_ms += interval_ms;
                    state.beep_remaining_ms = beep_duration_ms;
                }
            }

            if state.beep_remaining_ms > 0.0 {
                let sample = waveform_sample(clip.wave, state.phase, Some(&mut state.noise_state))
                    * clip.volume.clamp(0.0, 1.0);
                samples.push(sample);
                state.phase = (state.phase + frequency / sample_rate as f32).fract();
                state.beep_remaining_ms = (state.beep_remaining_ms - sample_dt_ms).max(0.0);
            } else {
                samples.push(0.0);
            }
        }

        samples
    }
}

pub fn parse_generated_audio_asset(prepared: &PreparedAsset) -> Result<GeneratedAudioClip, String> {
    if prepared.kind != PreparedAssetKind::GeneratedAudio {
        return Err(format!(
            "asset `{}` is not generated-audio",
            prepared.key.as_str()
        ));
    }

    let generator = metadata_string(prepared, "generator")?;
    let mode = match metadata_string(prepared, "mode")?.as_str() {
        "pregenerated" => GeneratedAudioMode::Pregenerated,
        "realtime" => GeneratedAudioMode::Realtime,
        other => return Err(format!("unsupported generated-audio mode `{other}`")),
    };

    match mode {
        GeneratedAudioMode::Pregenerated => Ok(GeneratedAudioClip::Pregenerated(
            PregeneratedGeneratedAudioClip {
                generator,
                sample_rate: metadata_u32(prepared, "sample_rate")?
                    .unwrap_or(DEFAULT_AUDIO_SAMPLE_RATE),
                sequence: ToneSequence {
                    tones: parse_tone_sequence(prepared)?,
                },
                envelope: Envelope {
                    attack_ms: metadata_u32(prepared, "envelope.attack_ms")?.unwrap_or(0),
                    release_ms: metadata_u32(prepared, "envelope.release_ms")?.unwrap_or(0),
                },
            },
        )),
        GeneratedAudioMode::Realtime => {
            Ok(GeneratedAudioClip::Realtime(RealtimeGeneratedAudioClip {
                generator,
                sample_rate: metadata_u32(prepared, "sample_rate")?
                    .unwrap_or(DEFAULT_AUDIO_SAMPLE_RATE),
                wave: Waveform::parse(&metadata_string(prepared, "wave")?)?,
                volume: metadata_f32(prepared, "volume")?
                    .unwrap_or(0.25)
                    .clamp(0.0, 1.0),
                params: parse_param_specs(prepared),
                interval_ms: parse_mapping(prepared, "mapping.interval_ms")?,
                frequency: parse_mapping(prepared, "mapping.frequency")?,
            }))
        }
    }
}

fn parse_tone_sequence(prepared: &PreparedAsset) -> Result<Vec<Tone>, String> {
    let mut tones = Vec::new();

    for index in 0.. {
        let wave_key = format!("sequence.{index}.wave");
        let Some(wave_value) = prepared.metadata.get(&wave_key) else {
            break;
        };
        tones.push(Tone {
            wave: Waveform::parse(wave_value)?,
            frequency: metadata_f32(prepared, &format!("sequence.{index}.frequency"))?
                .ok_or_else(|| format!("missing `sequence.{index}.frequency`"))?,
            duration_ms: metadata_u32(prepared, &format!("sequence.{index}.duration_ms"))?
                .ok_or_else(|| format!("missing `sequence.{index}.duration_ms`"))?,
            volume: metadata_f32(prepared, &format!("sequence.{index}.volume"))?
                .unwrap_or(0.25)
                .clamp(0.0, 1.0),
        });
    }

    if tones.is_empty() {
        return Err(format!(
            "generated-audio asset `{}` is missing `sequence` entries",
            prepared.key.as_str()
        ));
    }

    Ok(tones)
}

fn parse_param_specs(prepared: &PreparedAsset) -> BTreeMap<String, GeneratedAudioParamSpec> {
    let mut params = BTreeMap::new();

    for key in prepared.metadata.keys() {
        let Some(rest) = key.strip_prefix("params.") else {
            continue;
        };
        let Some((param_name, field_name)) = rest.split_once('.') else {
            continue;
        };
        let entry = params
            .entry(param_name.to_owned())
            .or_insert(GeneratedAudioParamSpec {
                default: 0.0,
                min: 0.0,
                max: 1.0,
            });

        let value = metadata_f32(prepared, key)
            .ok()
            .flatten()
            .unwrap_or_default();
        match field_name {
            "default" => entry.default = value,
            "min" => entry.min = value,
            "max" => entry.max = value,
            _ => {}
        }
    }

    params
}

fn parse_mapping(
    prepared: &PreparedAsset,
    prefix: &str,
) -> Result<GeneratedAudioParamMapping, String> {
    Ok(GeneratedAudioParamMapping {
        from_param: metadata_string(prepared, &format!("{prefix}.from_param"))?,
        near_value: metadata_f32(prepared, &format!("{prefix}.near_value"))?
            .ok_or_else(|| format!("missing `{prefix}.near_value`"))?,
        far_value: metadata_f32(prepared, &format!("{prefix}.far_value"))?
            .ok_or_else(|| format!("missing `{prefix}.far_value`"))?,
    })
}

fn mapped_value(
    mapping: &GeneratedAudioParamMapping,
    specs: &BTreeMap<String, GeneratedAudioParamSpec>,
    params: Option<&BTreeMap<String, f32>>,
) -> f32 {
    let spec = specs.get(&mapping.from_param);
    let (default, min, max) = spec
        .map(|spec| (spec.default, spec.min, spec.max))
        .unwrap_or((0.0, 0.0, 1.0));
    let raw_value = params
        .and_then(|params| params.get(&mapping.from_param).copied())
        .unwrap_or(default);
    let clamped = raw_value.clamp(min, max);
    let normalized = if (max - min).abs() <= f32::EPSILON {
        0.0
    } else {
        (clamped - min) / (max - min)
    };
    mapping.near_value + (mapping.far_value - mapping.near_value) * normalized
}

fn envelope_amplitude(
    sample_index: usize,
    total_samples: usize,
    attack_samples: usize,
    release_samples: usize,
) -> f32 {
    let attack = if attack_samples > 0 && sample_index < attack_samples {
        (sample_index as f32 / attack_samples as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let release = if release_samples > 0 && sample_index + release_samples >= total_samples {
        ((total_samples - sample_index) as f32 / release_samples as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };

    attack.min(release)
}

fn waveform_sample(waveform: Waveform, phase: f32, noise_state: Option<&mut u32>) -> f32 {
    match waveform {
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sine => (phase * 2.0 * PI).sin(),
        Waveform::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
        Waveform::Noise => match noise_state {
            Some(state) => {
                *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let normalized = ((*state >> 8) & 0xFFFF) as f32 / 65_535.0;
                normalized * 2.0 - 1.0
            }
            None => 0.0,
        },
    }
}

fn milliseconds_to_sample_count(duration_ms: f32, sample_rate: u32) -> usize {
    ((duration_ms.max(0.0) / 1000.0) * sample_rate as f32).round() as usize
}

fn metadata_string(prepared: &PreparedAsset, key: &str) -> Result<String, String> {
    prepared
        .metadata
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing `{key}` in `{}`", prepared.key.as_str()))
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Result<Option<u32>, String> {
    match prepared.metadata.get(key) {
        Some(value) => value.parse::<u32>().map(Some).map_err(|error| {
            format!(
                "invalid u32 `{key}` in `{}`: {error}",
                prepared.key.as_str()
            )
        }),
        None => Ok(None),
    }
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Result<Option<f32>, String> {
    match prepared.metadata.get(key) {
        Some(value) => value.parse::<f32>().map(Some).map_err(|error| {
            format!(
                "invalid f32 `{key}` in `{}`: {error}",
                prepared.key.as_str()
            )
        }),
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedAudioDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct GeneratedAudioPlugin;

impl RuntimePlugin for GeneratedAudioPlugin {
    fn name(&self) -> &'static str {
        "amigo-audio-generated"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(GeneratedAudioDomainInfo {
            crate_name: "amigo-audio-generated",
            capability: "generated_audio",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use amigo_assets::{
        AssetKey, AssetSourceKind, LoadedAsset, PreparedAsset, PreparedAssetKind,
        prepare_asset_from_contents,
    };

    use super::{
        DEFAULT_AUDIO_SAMPLE_RATE, GeneratedAudioClip, PcSpeakerGenerator, PcSpeakerRealtimeState,
        Waveform, parse_generated_audio_asset,
    };

    fn prepared_generated_audio(contents: &str) -> PreparedAsset {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/audio/test"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/audio/test.yml"),
            byte_len: contents.len() as u64,
        };
        let prepared = prepare_asset_from_contents(&loaded, contents)
            .expect("generated audio metadata should parse");
        assert_eq!(prepared.kind, PreparedAssetKind::GeneratedAudio);
        prepared
    }

    #[test]
    fn reports_total_sequence_duration() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 44100
sequence:
  - wave: square
    frequency: 330
    duration_ms: 40
    volume: 0.35
  - wave: square
    frequency: 520
    duration_ms: 90
    volume: 0.30
envelope:
  attack_ms: 2
  release_ms: 30
"#,
        );

        let GeneratedAudioClip::Pregenerated(clip) =
            parse_generated_audio_asset(&prepared).expect("clip should parse")
        else {
            panic!("expected pregenerated clip");
        };

        assert_eq!(clip.sequence.total_duration_ms(), 130);
    }

    #[test]
    fn generates_expected_sample_count_for_square_wave_clip() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 1000
sequence:
  - wave: square
    frequency: 100
    duration_ms: 10
    volume: 0.5
envelope:
  attack_ms: 0
  release_ms: 0
"#,
        );

        let GeneratedAudioClip::Pregenerated(clip) =
            parse_generated_audio_asset(&prepared).expect("clip should parse")
        else {
            panic!("expected pregenerated clip");
        };
        let pcm = PcSpeakerGenerator::generate_clip(&clip);

        assert_eq!(pcm.sample_rate, 1000);
        assert_eq!(pcm.samples.len(), 10);
        assert!(pcm.samples.iter().all(|sample| sample.abs() <= 0.5));
    }

    #[test]
    fn envelope_shapes_tone_amplitude() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: pregenerated
sample_rate: 1000
sequence:
  - wave: square
    frequency: 100
    duration_ms: 20
    volume: 1.0
envelope:
  attack_ms: 5
  release_ms: 5
"#,
        );

        let GeneratedAudioClip::Pregenerated(clip) =
            parse_generated_audio_asset(&prepared).expect("clip should parse")
        else {
            panic!("expected pregenerated clip");
        };
        let pcm = PcSpeakerGenerator::generate_clip(&clip);

        assert!(pcm.samples.first().copied().unwrap_or_default().abs() < 0.5);
        assert!(pcm.samples.last().copied().unwrap_or_default().abs() < 0.5);
        assert!(
            pcm.samples
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0_f32, f32::max)
                <= 1.0
        );
    }

    #[test]
    fn parses_realtime_generated_audio_and_maps_distance_deterministically() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: realtime
wave: square
volume: 0.20
params:
  distance:
    default: 256.0
    min: 0.0
    max: 512.0
mapping:
  interval_ms:
    from_param: distance
    near_value: 80
    far_value: 900
  frequency:
    from_param: distance
    near_value: 1200
    far_value: 320
"#,
        );

        let GeneratedAudioClip::Realtime(clip) =
            parse_generated_audio_asset(&prepared).expect("clip should parse")
        else {
            panic!("expected realtime clip");
        };
        assert_eq!(clip.sample_rate, DEFAULT_AUDIO_SAMPLE_RATE);
        assert_eq!(clip.wave, Waveform::Square);

        let mut state = PcSpeakerRealtimeState::default();
        let near = PcSpeakerGenerator::generate_realtime_frame(
            &clip,
            Some(&BTreeMap::from([("distance".to_owned(), 0.0)])),
            &mut state,
            256,
        );
        let far = PcSpeakerGenerator::generate_realtime_frame(
            &clip,
            Some(&BTreeMap::from([("distance".to_owned(), 512.0)])),
            &mut state,
            256,
        );

        assert_eq!(near.len(), 256);
        assert_eq!(far.len(), 256);
        assert!(near.iter().any(|sample| sample.abs() > 0.0));
    }
}
