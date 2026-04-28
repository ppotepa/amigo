use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_assets::PreparedAsset;
use amigo_audio_api::AudioClipKey;
use amigo_audio_generated::{
    DEFAULT_AUDIO_SAMPLE_RATE, GeneratedAudioClip, PcSpeakerGenerator, PcSpeakerRealtimeState,
    parse_generated_audio_asset,
};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioMixFrame {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct QueuedOneShotPlayback {
    label: String,
    sample_rate: u32,
    samples: Vec<f32>,
    cursor: usize,
}

#[derive(Debug, Default)]
struct AudioMixerState {
    one_shots: Vec<QueuedOneShotPlayback>,
    realtime_sources: BTreeMap<String, PcSpeakerRealtimeState>,
    mixed_frames: Vec<AudioMixFrame>,
}

#[derive(Debug, Default)]
pub struct AudioMixerService {
    state: Mutex<AudioMixerState>,
}

impl AudioMixerService {
    pub fn queue_generated_one_shot(
        &self,
        label: impl Into<String>,
        prepared_asset: &PreparedAsset,
    ) -> Result<(), String> {
        let GeneratedAudioClip::Pregenerated(clip) = parse_generated_audio_asset(prepared_asset)?
        else {
            return Err(format!(
                "asset `{}` is not a pregenerated generated-audio clip",
                prepared_asset.key.as_str()
            ));
        };

        let pcm = PcSpeakerGenerator::generate_clip(&clip);
        self.state
            .lock()
            .expect("audio mixer service mutex should not be poisoned")
            .one_shots
            .push(QueuedOneShotPlayback {
                label: label.into(),
                sample_rate: pcm.sample_rate,
                samples: pcm.samples,
                cursor: 0,
            });
        Ok(())
    }

    pub fn tick_generated_audio(
        &self,
        prepared_assets: &BTreeMap<String, PreparedAsset>,
        playing_sources: &BTreeMap<String, AudioClipKey>,
        source_params: &BTreeMap<String, BTreeMap<String, f32>>,
        master_volume: f32,
        frame_sample_count: usize,
    ) -> Option<AudioMixFrame> {
        let mut state = self
            .state
            .lock()
            .expect("audio mixer service mutex should not be poisoned");

        state
            .realtime_sources
            .retain(|source_id, _| playing_sources.contains_key(source_id));
        for source_id in playing_sources.keys() {
            state.realtime_sources.entry(source_id.clone()).or_default();
        }

        let mut mix_samples = vec![0.0; frame_sample_count];
        let mut mixed_sources = Vec::new();
        let mut retained_one_shots = Vec::with_capacity(state.one_shots.len());

        for mut playback in state.one_shots.drain(..) {
            let remaining = playback.samples.len().saturating_sub(playback.cursor);
            if remaining == 0 {
                continue;
            }

            let sample_count = remaining.min(frame_sample_count);
            for (target, sample) in mix_samples
                .iter_mut()
                .zip(playback.samples[playback.cursor..playback.cursor + sample_count].iter())
            {
                *target += *sample;
            }

            playback.cursor += sample_count;
            mixed_sources.push(playback.label.clone());
            if playback.cursor < playback.samples.len() {
                retained_one_shots.push(playback);
            }
        }
        state.one_shots = retained_one_shots;

        for (source_id, clip_key) in playing_sources {
            let Some(prepared_asset) = prepared_assets.get(clip_key.as_str()) else {
                continue;
            };
            let Ok(GeneratedAudioClip::Realtime(clip)) =
                parse_generated_audio_asset(prepared_asset)
            else {
                continue;
            };
            let realtime_state = state.realtime_sources.entry(source_id.clone()).or_default();
            let samples = PcSpeakerGenerator::generate_realtime_frame(
                &clip,
                source_params.get(source_id),
                realtime_state,
                frame_sample_count,
            );
            for (target, sample) in mix_samples.iter_mut().zip(samples.iter()) {
                *target += *sample;
            }
            mixed_sources.push(source_id.clone());
        }

        if mixed_sources.is_empty() {
            return None;
        }

        let master_volume = master_volume.clamp(0.0, 1.0);
        for sample in &mut mix_samples {
            *sample = (*sample * master_volume).clamp(-1.0, 1.0);
        }

        let frame = AudioMixFrame {
            sample_rate: DEFAULT_AUDIO_SAMPLE_RATE,
            samples: mix_samples,
            sources: mixed_sources,
        };
        state.mixed_frames.push(frame.clone());
        Some(frame)
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("audio mixer service mutex should not be poisoned");
        state.one_shots.clear();
        state.realtime_sources.clear();
        state.mixed_frames.clear();
    }

    pub fn frames(&self) -> Vec<AudioMixFrame> {
        self.state
            .lock()
            .expect("audio mixer service mutex should not be poisoned")
            .mixed_frames
            .clone()
    }

    pub fn active_realtime_sources(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("audio mixer service mutex should not be poisoned")
            .realtime_sources
            .keys()
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct AudioMixerDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct AudioMixerPlugin;

impl RuntimePlugin for AudioMixerPlugin {
    fn name(&self) -> &'static str {
        "amigo-audio-mixer"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(AudioMixerService::default())?;
        registry.register(AudioMixerDomainInfo {
            crate_name: "amigo-audio-mixer",
            capability: "audio_mix",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use amigo_assets::{AssetKey, AssetSourceKind, LoadedAsset, prepare_asset_from_contents};
    use amigo_audio_api::AudioClipKey;

    use super::AudioMixerService;

    fn prepared_generated_audio(contents: &str) -> amigo_assets::PreparedAsset {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/audio/test"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/audio/test.yml"),
            byte_len: contents.len() as u64,
        };
        prepare_asset_from_contents(&loaded, contents)
            .expect("generated audio metadata should parse")
    }

    #[test]
    fn queues_and_ticks_generated_one_shot_audio() {
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
    volume: 0.5
envelope:
  attack_ms: 0
  release_ms: 0
"#,
        );
        let service = AudioMixerService::default();

        service
            .queue_generated_one_shot("jump", &prepared)
            .expect("one-shot should queue");
        let frame = service
            .tick_generated_audio(
                &BTreeMap::new(),
                &BTreeMap::new(),
                &BTreeMap::new(),
                1.0,
                16,
            )
            .expect("mix frame should be produced");

        assert_eq!(frame.sample_rate, 44_100);
        assert_eq!(frame.samples.len(), 16);
        assert!(frame.sources.iter().any(|source| source == "jump"));
    }

    #[test]
    fn ticks_realtime_generated_audio_source() {
        let prepared = prepared_generated_audio(
            r#"
kind: generated-audio
generator: pc-speaker
mode: realtime
wave: square
volume: 0.2
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
        let service = AudioMixerService::default();
        let prepared_assets =
            BTreeMap::from([(prepared.key.as_str().to_owned(), prepared.clone())]);
        let playing_sources = BTreeMap::from([(
            "proximity-beep".to_owned(),
            AudioClipKey::new("playground-sidescroller/audio/test"),
        )]);
        let source_params = BTreeMap::from([(
            "proximity-beep".to_owned(),
            BTreeMap::from([("distance".to_owned(), 32.0)]),
        )]);

        let frame = service
            .tick_generated_audio(&prepared_assets, &playing_sources, &source_params, 1.0, 128)
            .expect("realtime frame should be produced");

        assert_eq!(frame.samples.len(), 128);
        assert!(
            frame
                .sources
                .iter()
                .any(|source| source == "proximity-beep")
        );
        assert!(
            service
                .active_realtime_sources()
                .iter()
                .any(|source| source == "proximity-beep")
        );
    }
}
