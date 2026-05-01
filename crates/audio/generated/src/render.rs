use std::collections::BTreeMap;
use std::f32::consts::PI;

use crate::parser::mapped_value;
use crate::types::{
    PcSpeakerRealtimeState, PregeneratedGeneratedAudioClip, RealtimeGeneratedAudioClip, Waveform,
    GeneratedAudioPcm,
};

pub struct PcSpeakerGenerator;

impl PcSpeakerGenerator {
    pub fn generate_clip(clip: &PregeneratedGeneratedAudioClip) -> GeneratedAudioPcm {
        let mut samples = Vec::new();
        let sample_rate = clip.sample_rate.max(1);

        for tone in &clip.sequence.tones {
            let tone_sample_count = milliseconds_to_sample_count(tone.duration_ms as f32, sample_rate);
            if tone_sample_count == 0 {
                continue;
            }

            let attack_samples = milliseconds_to_sample_count(clip.envelope.attack_ms as f32, sample_rate);
            let release_samples = milliseconds_to_sample_count(clip.envelope.release_ms as f32, sample_rate);
            let mut phase = 0.0f32;
            let frequency = tone.frequency.max(0.0);

            for sample_index in 0..tone_sample_count {
                let amplitude =
                    envelope_amplitude(sample_index, tone_sample_count, attack_samples, release_samples)
                        * tone.volume.clamp(0.0, 1.0);
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
                let sample =
                    waveform_sample(clip.wave, state.phase, Some(&mut state.noise_state)) * clip.volume.clamp(0.0, 1.0);
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

fn milliseconds_to_sample_count(duration_ms: f32, sample_rate: u32) -> usize {
    ((duration_ms.max(0.0) / 1000.0) * sample_rate as f32).round() as usize
}
