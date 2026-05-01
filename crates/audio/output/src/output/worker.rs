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
