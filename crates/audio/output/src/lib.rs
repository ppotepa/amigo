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

#[derive(Debug)]
enum AudioOutputWorkerInitResult {
    Started,
    Unavailable(String),
    Error(String),
}

fn run_audio_output_worker(
    state: Arc<Mutex<AudioOutputBackendState>>,
    tx: mpsc::Sender<AudioOutputWorkerInitResult>,
) {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        let reason = "no default output audio device available".to_owned();
        let mut locked = state
            .lock()
            .expect("audio output mutex should not be poisoned");
        locked.started = false;
        locked.worker_active = false;
        locked.last_error = Some(reason.clone());
        let _ = tx.send(AudioOutputWorkerInitResult::Unavailable(reason));
        return;
    };

    let device_name = device
        .name()
        .unwrap_or_else(|_| "unknown-output-device".to_owned());
    let supported_config = match device.default_output_config() {
        Ok(config) => config,
        Err(error) => {
            let reason =
                format!("failed to query default output config for `{device_name}`: {error}");
            let mut locked = state
                .lock()
                .expect("audio output mutex should not be poisoned");
            locked.started = false;
            locked.worker_active = false;
            locked.last_error = Some(reason.clone());
            let _ = tx.send(AudioOutputWorkerInitResult::Error(reason));
            return;
        }
    };

    let sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels();
    let sample_format = supported_config.sample_format();
    let stream_config: StreamConfig = supported_config.into();
    let stream_state = Arc::clone(&state);
    let error_state = Arc::clone(&state);

    let stream = match sample_format {
        SampleFormat::F32 => {
            build_output_stream::<f32>(&device, &stream_config, stream_state, error_state)
        }
        SampleFormat::I16 => {
            build_output_stream::<i16>(&device, &stream_config, stream_state, error_state)
        }
        SampleFormat::U16 => {
            build_output_stream::<u16>(&device, &stream_config, stream_state, error_state)
        }
        other => Err(format!(
            "unsupported output sample format `{other:?}` for `{device_name}`"
        )),
    };

    let stream = match stream {
        Ok(stream) => stream,
        Err(reason) => {
            let mut locked = state
                .lock()
                .expect("audio output mutex should not be poisoned");
            locked.started = false;
            locked.worker_active = false;
            locked.last_error = Some(reason.clone());
            let _ = tx.send(AudioOutputWorkerInitResult::Error(reason));
            return;
        }
    };

    if let Err(error) = stream.play() {
        let reason = format!("failed to start audio stream for `{device_name}`: {error}");
        let mut locked = state
            .lock()
            .expect("audio output mutex should not be poisoned");
        locked.started = false;
        locked.worker_active = false;
        locked.last_error = Some(reason.clone());
        let _ = tx.send(AudioOutputWorkerInitResult::Error(reason));
        return;
    }

    {
        let mut locked = state
            .lock()
            .expect("audio output mutex should not be poisoned");
        locked.device_name = Some(device_name);
        locked.sample_rate = Some(sample_rate);
        locked.channels = Some(channels);
        locked.started = true;
        locked.worker_active = true;
        locked.last_error = None;
    }
    let _ = tx.send(AudioOutputWorkerInitResult::Started);

    loop {
        thread::sleep(Duration::from_millis(250));
        let started = state
            .lock()
            .expect("audio output mutex should not be poisoned")
            .started;
        if !started {
            break;
        }
    }

    drop(stream);
    let mut locked = state
        .lock()
        .expect("audio output mutex should not be poisoned");
    locked.worker_active = false;
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    state: Arc<Mutex<AudioOutputBackendState>>,
    error_state: Arc<Mutex<AudioOutputBackendState>>,
) -> Result<cpal::Stream, String>
where
    T: Sample + FromSample<f32> + SizedSample,
{
    let channels = usize::from(config.channels);
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _| {
                write_output_data(output, channels, &state);
            },
            move |error| {
                let mut state = error_state
                    .lock()
                    .expect("audio output mutex should not be poisoned");
                state.last_error = Some(format!("audio stream error: {error}"));
                state.started = false;
                state.worker_active = false;
            },
            None,
        )
        .map_err(|error| format!("failed to build output stream: {error}"))
}

fn write_output_data<T>(
    output: &mut [T],
    channels: usize,
    state: &Arc<Mutex<AudioOutputBackendState>>,
) where
    T: Sample + FromSample<f32> + SizedSample,
{
    let mut state = state
        .lock()
        .expect("audio output mutex should not be poisoned");

    for frame in output.chunks_mut(channels) {
        let sample = state.queued_samples.pop_front().unwrap_or(0.0);
        let value = T::from_sample(sample);
        for channel in frame {
            *channel = value;
        }
    }
}

fn trim_buffer(queue: &mut VecDeque<f32>) {
    while queue.len() > MAX_BUFFERED_SAMPLES {
        let _ = queue.pop_front();
    }
}

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
        })
    }
}

#[cfg(test)]
mod tests {
    use amigo_audio_mixer::AudioMixFrame;

    use super::{AudioOutputBackendService, MAX_BUFFERED_SAMPLES};

    #[test]
    fn queues_audio_frames_without_starting_stream() {
        let service = AudioOutputBackendService::default();
        service.enqueue_mix_frame(&AudioMixFrame {
            sample_rate: 44_100,
            samples: vec![0.1, -0.2, 0.3, -0.4],
            sources: vec!["jump".to_owned()],
        });

        let snapshot = service.snapshot();
        assert_eq!(snapshot.backend_name, "system-audio");
        assert!(!snapshot.started);
        assert_eq!(snapshot.buffered_samples, 4);
    }

    #[test]
    fn trims_queued_samples_to_reasonable_limit() {
        let service = AudioOutputBackendService::default();
        service.enqueue_mix_frame(&AudioMixFrame {
            sample_rate: 44_100,
            samples: vec![0.0; MAX_BUFFERED_SAMPLES + 1024],
            sources: vec!["overflow".to_owned()],
        });

        let snapshot = service.snapshot();
        assert_eq!(snapshot.buffered_samples, MAX_BUFFERED_SAMPLES);
    }
}
