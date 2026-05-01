use std::collections::VecDeque;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use amigo_audio_mixer::AudioMixFrame;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};

const MAX_BUFFERED_SAMPLES: usize = 44_100 * 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioOutputStartStatus {
    Started,
    AlreadyStarted,
    Unavailable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioOutputBackendSnapshot {
    pub backend_name: String,
    pub device_name: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub started: bool,
    pub buffered_samples: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Default)]
struct AudioOutputBackendState {
    queued_samples: VecDeque<f32>,
    device_name: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    started: bool,
    worker_active: bool,
    last_error: Option<String>,
}

#[derive(Clone, Default)]
pub struct AudioOutputBackendService {
    state: Arc<Mutex<AudioOutputBackendState>>,
}

impl std::fmt::Debug for AudioOutputBackendService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioOutputBackendService")
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

impl AudioOutputBackendService {
    pub fn backend_name(&self) -> &'static str {
        "system-audio"
    }

    pub fn start_if_available(&self) -> Result<AudioOutputStartStatus, String> {
        {
            let mut state = self
                .state
                .lock()
                .expect("audio output mutex should not be poisoned");
            if state.started || state.worker_active {
                return Ok(AudioOutputStartStatus::AlreadyStarted);
            }
            state.worker_active = true;
            state.last_error = None;
        }

        let (tx, rx) = mpsc::channel();
        let shared_state = Arc::clone(&self.state);
        thread::Builder::new()
            .name("amigo-audio-output".to_owned())
            .spawn(move || run_audio_output_worker(shared_state, tx))
            .map_err(|error| {
                let message = format!("failed to spawn audio output worker: {error}");
                let mut state = self
                    .state
                    .lock()
                    .expect("audio output mutex should not be poisoned");
                state.worker_active = false;
                state.last_error = Some(message.clone());
                message
            })?;

        match rx.recv() {
            Ok(AudioOutputWorkerInitResult::Started) => Ok(AudioOutputStartStatus::Started),
            Ok(AudioOutputWorkerInitResult::Unavailable(reason)) => {
                let mut state = self
                    .state
                    .lock()
                    .expect("audio output mutex should not be poisoned");
                state.last_error = Some(reason);
                Ok(AudioOutputStartStatus::Unavailable)
            }
            Ok(AudioOutputWorkerInitResult::Error(reason)) => Err(reason),
            Err(error) => Err(format!(
                "failed to receive audio output worker init result: {error}"
            )),
        }
    }

    pub fn enqueue_mix_frame(&self, frame: &AudioMixFrame) {
        let mut state = self
            .state
            .lock()
            .expect("audio output mutex should not be poisoned");
        state.queued_samples.extend(frame.samples.iter().copied());
        trim_buffer(&mut state.queued_samples);
    }

    pub fn clear_buffer(&self) {
        self.state
            .lock()
            .expect("audio output mutex should not be poisoned")
            .queued_samples
            .clear();
    }

    pub fn snapshot(&self) -> AudioOutputBackendSnapshot {
        let state = self
            .state
            .lock()
            .expect("audio output mutex should not be poisoned");
        AudioOutputBackendSnapshot {
            backend_name: self.backend_name().to_owned(),
            device_name: state.device_name.clone(),
            sample_rate: state.sample_rate,
            channels: state.channels,
            started: state.started,
            buffered_samples: state.queued_samples.len(),
            last_error: state.last_error.clone(),
        }
    }
}


include!("output/worker.rs");
include!("output/plugin.rs");

#[cfg(test)]
include!("tests.rs");
