//! Explicit NPR packet bridge to a companion-owned offscreen target.
#[cfg(windows)]
mod native;
use amigo_npr_playground_plugin::{
    NprPlaygroundRenderService, NprPlaygroundState, playground::NprPlaygroundService,
    state::Settings,
};
use amigo_playground_api::*;
use amigo_render_wgpu::{
    WgpuNprRenderer, WgpuOffscreenTarget, WgpuReadbackPool, WgpuRenderBackend,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

struct Job {
    settings: Settings,
    size: [u32; 2],
    revision: u64,
    permit: PlaygroundFramePermit,
    frames: Arc<PlaygroundFrameQueue>,
    input_id: u64,
    mode: PlaygroundViewportMode,
}
struct Worker {
    sender: mpsc::SyncSender<Job>,
    failures: mpsc::Receiver<(u64, String)>,
    diagnostics: mpsc::Receiver<serde_json::Value>,
}
#[derive(Default)]
struct ViewportState {
    key: Option<String>,
    worker: Option<Worker>,
    policy: PlaygroundFramePolicy,
    previous: Option<(Settings, [u32; 2], u64)>,
    settle_until: Option<Instant>,
    camera_until: Option<Instant>,
}
#[derive(Default)]
pub struct NprPlaygroundViewportRenderer(Mutex<ViewportState>);
pub(crate) struct NprPlaygroundViewportPlugin;
impl amigo_runtime::RuntimePlugin for NprPlaygroundViewportPlugin {
    fn name(&self) -> &'static str {
        "amigo-npr-playground-viewport"
    }
    fn register(
        &self,
        registry: &mut amigo_runtime::ServiceRegistry,
    ) -> amigo_core::AmigoResult<()> {
        registry.register(NprPlaygroundViewportRenderer::default())?;
        registry
            .required::<amigo_runtime::SystemRegistry>()?
            .register_fn(
                amigo_runtime::SystemPhase::PostUpdate,
                "npr_playground_viewport",
                |runtime| {
                    let companions = runtime.required::<crate::PlaygroundCompanionService>()?;
                    let id = PlaygroundId("npr-playground".into());
                    let bridge = runtime.required::<NprPlaygroundViewportRenderer>()?;
                    let mut bridge = bridge.0.lock().unwrap();
                    let (Some(request), Some(frames)) =
                        (companions.viewport(&id), companions.frame_queue(&id))
                    else {
                        bridge.worker = None;
                        bridge.key = None;
                        bridge.previous = None;
                        bridge.settle_until = None;
                        bridge.camera_until = None;
                        return Ok(());
                    };
                    let snapshot = runtime
                        .required::<amigo_session::SceneSessionService>()?
                        .snapshot();
                    let Some(doc) = snapshot.loaded_scene_document() else {
                        return Ok(());
                    };
                    let key = format!(
                        "{}:{}:{}",
                        doc.source_mod,
                        doc.scene_id,
                        snapshot.lifecycle_summary().clear_count
                    );
                    if bridge.key.as_ref() != Some(&key) {
                        let mods = runtime.required::<amigo_modding::ModCatalog>()?;
                        let root = mods
                            .mod_by_id(&doc.source_mod)
                            .ok_or_else(|| {
                                amigo_core::AmigoError::Message("missing active mod".into())
                            })?
                            .root_path
                            .clone();
                        bridge.worker = Some(start_worker(
                            root,
                            runtime.required::<amigo_assets::AssetCatalog>()?,
                            runtime.required::<NprPlaygroundRenderService>()?,
                            #[cfg(windows)]
                            companions.native(&id),
                        ));
                        bridge.key = Some(key);
                        bridge.previous = None;
                        bridge.settle_until = None;
                        bridge.camera_until = None;
                        bridge.policy = Default::default();
                    }
                    let failures = bridge
                        .worker
                        .as_ref()
                        .unwrap()
                        .failures
                        .try_iter()
                        .collect::<Vec<_>>();
                    for (generation, error) in failures {
                        companions.send_viewport_error(
                            &id,
                            if generation == 0 { request.generation } else { generation },
                            error,
                        );
                    }
                    if let Some(stats) = bridge
                        .worker
                        .as_ref()
                        .unwrap()
                        .diagnostics
                        .try_iter()
                        .last()
                    {
                        companions.send_viewport_diagnostics(&id, stats);
                    }
                    // Capture requests carry the full (non-adaptive) DPR, while
                    // preserving this viewport's aspect ratio and output limit.
                    let size = request
                        .effective_size([1280, 720])
                        .map_err(|e| amigo_core::AmigoError::Message(e.into()))?;
                    let settings = runtime.required::<NprPlaygroundState>()?.render_snapshot();
                    let dirty = bridge.previous.as_ref()
                        != Some(&(settings.clone(), size, request.generation))
                        || request.high_quality_capture
                        || !frames.produced();
                    let animated =
                        settings.needs_temporal_frames() || bridge.settle_until.is_some();
                    let now = Instant::now();
                    let camera_changed =
                        bridge.previous.as_ref().is_some_and(|(previous, _, _)| {
                            previous.camera_target != settings.camera_target
                                || previous.camera_yaw != settings.camera_yaw
                                || previous.camera_pitch != settings.camera_pitch
                                || previous.camera_distance != settings.camera_distance
                                || previous.camera_fov != settings.camera_fov
                        });
                    if request.interacting || camera_changed {
                        bridge.camera_until = Some(now + Duration::from_millis(500));
                    }
                    if bridge.camera_until.is_some_and(|deadline| deadline <= now) {
                        bridge.camera_until = None;
                    }
                    let interacting = request.interacting || bridge.camera_until.is_some();
                    if bridge.policy.due(now, dirty, animated, interacting) {
                        let Some(permit) = frames.acquire() else {
                            return Ok(());
                        };
                        let revision = runtime.required::<NprPlaygroundService>()?.revision();
                        let job = Job {
                            settings: settings.clone(),
                            size,
                            revision,
                            permit,
                            frames,
                            input_id: companions.input_id(&id),
                            mode: request.mode,
                        };
                        if bridge.worker.as_ref().unwrap().sender.try_send(job).is_ok() {
                            if dirty {
                                bridge.settle_until = Some(
                                    now + Duration::from_secs_f32(
                                        settings.motion.appearance_fade_seconds,
                                    ),
                                );
                            } else if bridge.settle_until.is_some_and(|deadline| now >= deadline) {
                                // Keep requesting until a frame at/after the deadline is accepted,
                                // including when readback is slower than the fade itself.
                                bridge.settle_until = None;
                            }
                            bridge.previous = Some((settings, size, request.generation));
                            bridge.policy.committed(now);
                            companions.capture_submitted(&id);
                        }
                    }
                    Ok(())
                },
            );
        Ok(())
    }
}

fn start_worker(
    root: PathBuf,
    assets: Arc<amigo_assets::AssetCatalog>,
    render: Arc<NprPlaygroundRenderService>,
    #[cfg(windows)] native_link: Option<Arc<amigo_playground_native::NativeLink>>,
) -> Worker {
    // A rendezvous channel has no backlog while extraction/readback/encoding runs.
    let (sender, jobs) = mpsc::sync_channel::<Job>(0);
    let (failure, failures) = mpsc::sync_channel(2);
    let (diagnostic, diagnostics) = mpsc::sync_channel(2);
    std::thread::spawn(move || {
        #[cfg(windows)]
        let mut native = native_link.map(native::NativeProducer::new);
        let native_diagnostic = diagnostic.clone();
        let (encode, encoding) =
            mpsc::sync_channel::<(Job, PlaygroundFrameHeader, Vec<u8>, serde_json::Value)>(1);
        let encode_failure = failure.clone();
        std::thread::spawn(move || {
            let mut generation = 0;
            let result = (|| -> Result<(), String> {
                let mut compressor = turbojpeg::Compressor::new().map_err(|e| e.to_string())?;
                compressor.set_quality(92).map_err(|e| e.to_string())?;
                compressor
                    .set_subsamp(turbojpeg::Subsamp::None)
                    .map_err(|e| e.to_string())?;
                while let Ok((job, mut header, rgba, mut stats)) = encoding.recv() {
                    generation = header.generation;
                    let started = Instant::now();
                    let payload = compressor
                        .compress_to_vec(turbojpeg::Image {
                            pixels: rgba.as_slice(),
                            width: job.size[0] as usize,
                            height: job.size[1] as usize,
                            pitch: job.size[0] as usize * 4,
                            format: turbojpeg::PixelFormat::RGBA,
                        })
                        .map_err(|e| e.to_string())?;
                    header.stages.encode_ms = started.elapsed().as_secs_f64() * 1000.0;
                    stats["stages"] = serde_json::json!(header.stages);
                    let _ = diagnostic.try_send(stats);
                    job.permit
                        .publish(PlaygroundViewportFrame { header, payload });
                }
                Ok(())
            })();
            if let Err(error) = result {
                let _ = encode_failure.try_send((generation, error));
            }
        });
        let extractor = render.fork_view();
        if let Err(error) = extractor.load_models(&root) {
            let _ = failure.try_send((0, error));
            return;
        }
        let mut target: Option<WgpuOffscreenTarget> = None;
        let mut renderer: Option<WgpuNprRenderer> = None;
        let mut readbacks = WgpuReadbackPool::default();
        let mut pending = std::collections::BTreeMap::<
            u64,
            (Job, PlaygroundFrameHeader, serde_json::Value),
        >::new();
        let mut stale_readback_drops = 0u64;
        let mut previous_time = Instant::now();
        let mut using_dx12 = false;
        let mut failed_configuration = None;
        let mut active_configuration = None;
        let mut device_failure: Option<Arc<Mutex<Option<String>>>> = None;
        loop {
            if let Some(error) = device_failure
                .as_ref()
                .and_then(|state| state.lock().unwrap().take())
            {
                // End the device session. A user mode selection/resize creates
                // another configuration; never replay the failed frame.
                pending.clear();
                readbacks = WgpuReadbackPool::default();
                renderer = None;
                target = None;
                device_failure = None;
                failed_configuration = active_configuration;
                #[cfg(windows)]
                if let Some(native) = &mut native {
                    native.retire_failed();
                }
                let _ = failure.try_send((active_configuration.map_or(0, |(_, generation, _)| generation), error));
            }
            #[cfg(windows)]
            if let Some(native) = &mut native {
                if let Err(error) = native.poll() {
                    let _ = failure.try_send((native.generation(), error));
                }
            }
            if let Some(target) = target.as_ref() {
                match readbacks.poll(&target.device) {
                    Ok(mut ready) => {
                        // Presentation is latest-frame-wins. Readback may
                        // complete several frames after the viewport has
                        // moved on; sending each one serializes stale work
                        // through the companion and inflates input latency.
                        // Dropping the superseded jobs releases their permits
                        // without changing the authored document or image
                        // semantics.
                        if let Some(latest_ticket) = ready.iter().map(|frame| frame.ticket).max() {
                            let stale: Vec<_> = pending
                                .keys()
                                .copied()
                                .filter(|ticket| *ticket < latest_ticket)
                                .collect();
                            for ticket in stale {
                                if pending.remove(&ticket).is_some() {
                                    stale_readback_drops = stale_readback_drops.saturating_add(1);
                                }
                            }
                            ready.retain(|frame| frame.ticket == latest_ticket);
                        }
                        for frame in ready {
                            if let Some((job, mut header, stats)) = pending.remove(&frame.ticket) {
                                header.stages.readback_ms = frame.milliseconds;
                                header.stages.stale_readback_drops = stale_readback_drops;
                                let mut stats = stats;
                                stats["stale_readback_drops"] = serde_json::json!(stale_readback_drops);
                                if header.mode == PlaygroundViewportMode::Jpeg {
                                    let _ = encode.try_send((job, header, frame.rgba, stats));
                                } else {
                                    #[cfg(windows)]
                                    if let Some(native) = &mut native {
                                        let _ = native_diagnostic.try_send(
                                            serde_json::json!({"npr":stats,"stages":header.stages}),
                                        );
                                        let generation = header.generation;
                                        if let Err(error) = native.present_rgba(
                                            &frame.rgba,
                                            header,
                                            job.permit,
                                            job.frames,
                                        ) {
                                            let _ = failure.try_send((generation, error));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        if let Some(state) = &device_failure {
                            *state.lock().unwrap() = Some(error);
                        }
                        continue;
                    }
                }
            }
            let job = match jobs.recv_timeout(Duration::from_millis(1)) {
                Ok(job) => job,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };
            let configuration = (job.mode, job.permit.generation, job.size);
            if failed_configuration == Some(configuration) {
                continue;
            }
            let result = (|| -> Result<(), String> {
                let needs_dx12 = cfg!(windows) && job.mode == PlaygroundViewportMode::NativeGpu;
                if target.is_none() || (needs_dx12 && !using_dx12) {
                    if !pending.is_empty() {
                        return Ok(());
                    }
                    // Staging buffers belong to their creating device. The
                    // first explicit Native GPU selection may replace Vulkan
                    // with DX12 after a restored JPEG/Local RGBA preference.
                    readbacks = WgpuReadbackPool::default();
                    let backend = if needs_dx12 {
                        WgpuRenderBackend::dx12()
                    } else {
                        WgpuRenderBackend::default()
                    };
                    let new = backend
                        .initialize_offscreen(job.size[0], job.size[1])
                        .map_err(|e| e.to_string())?;
                    let errors = Arc::new(Mutex::new(None));
                    let lost = errors.clone();
                    new.device.set_device_lost_callback(move |reason, message| {
                        *lost.lock().unwrap() =
                            Some(format!("Viewport GPU device lost ({reason:?}): {message}"));
                    });
                    let uncaptured = errors.clone();
                    new.device.on_uncaptured_error(Arc::new(move |error| {
                        *uncaptured.lock().unwrap() = Some(format!("Viewport GPU error: {error}"));
                    }));
                    device_failure = Some(errors);
                    renderer = Some(WgpuNprRenderer::new(&new.device, new.format));
                    target = Some(new);
                    using_dx12 = needs_dx12;
                }
                // Remember the active configuration so a device-loss event
                // requires an explicit new presentation request.
                failed_configuration = None;
                active_configuration = Some(configuration);
                target.as_mut().unwrap().resize(job.size[0], job.size[1]);
                if job.mode != PlaygroundViewportMode::Jpeg {
                    #[cfg(windows)]
                    if !native
                        .as_mut()
                        .ok_or("Native companion channel is unavailable")?
                        .prepare(job.mode, job.permit.generation, target.as_ref().unwrap())?
                    {
                        return Ok(());
                    }
                    #[cfg(not(windows))]
                    return Err(
                        "Native GPU and Local RGBA are currently implemented only on Windows"
                            .into(),
                    );
                }
                let dt = previous_time.elapsed().as_secs_f32();
                previous_time = Instant::now();
                for asset in assets.prepared_assets() {
                    if let Some(model) = asset.metadata.get("npr.model") {
                        if job
                            .settings
                            .objects
                            .values()
                            .any(|object| &object.model == model)
                            && asset.format.as_deref() == Some("gltf")
                        {
                            extractor.load_model(model, &asset.resolved_path)?;
                        }
                    }
                }
                let extraction = Instant::now();
                extractor.rebuild_with_delta(&job.settings, job.size, dt)?;
                let mut stages = PlaygroundFrameStages {
                    extraction_ms: extraction.elapsed().as_secs_f64() * 1000.0,
                    ..Default::default()
                };
                let target = target.as_mut().unwrap();
                let submit = Instant::now();
                extractor.with_commands(|commands, background| renderer.as_mut().unwrap().render(target, commands, background))
                    .map_err(|e| e.to_string())?;
                if let Some(error) = device_failure
                    .as_ref()
                    .and_then(|state| state.lock().unwrap().clone())
                {
                    return Err(error);
                }
                stages.submit_ms = submit.elapsed().as_secs_f64() * 1000.0;
                let header = PlaygroundFrameHeader {
                    sequence: job.permit.sequence,
                    generation: job.permit.generation,
                    revision: job.revision,
                    size: job.size,
                    mode: job.mode,
                    input_id: job.input_id,
                    format: PlaygroundPixelFormat::Rgba8Srgb,
                    stages,
                };
                if job.mode == PlaygroundViewportMode::NativeGpu {
                    #[cfg(windows)]
                    {
                        let _ = native_diagnostic.try_send(
                            serde_json::json!({"npr":extractor.stats(),"stages":header.stages}),
                        );
                        native
                            .as_mut()
                            .ok_or("Native companion channel is unavailable")?
                            .present_gpu(target, header, job.permit, job.frames)?;
                    }
                } else if readbacks.submit(target, header.sequence) {
                    pending.insert(
                        header.sequence,
                        (job, header, serde_json::json!(extractor.stats())),
                    );
                }
                Ok(())
            })();
            if let Err(error) = result {
                failed_configuration = Some(configuration);
                let _ = failure.try_send((configuration.1, error));
            }
        }
    });
    Worker {
        sender,
        failures,
        diagnostics,
    }
}
