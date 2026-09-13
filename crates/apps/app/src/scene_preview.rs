use std::path::PathBuf;

use amigo_core::{AmigoError, AmigoResult};
use amigo_render_wgpu::{WgpuOffscreenTarget, WgpuRenderBackend, WgpuSceneRenderer};
use amigo_runtime::Runtime;
use amigo_runtime::{SystemPhase, SystemRegistry};

use crate::{BootstrapOptions, BootstrapSummary, bootstrap_session_with_options};

#[derive(Debug, Clone)]
pub struct ScenePreviewOptions {
    pub mods_root: PathBuf,
    pub active_mods: Option<Vec<String>>,
    pub mod_id: String,
    pub scene_id: String,
    pub width: u32,
    pub height: u32,
    pub warmup_frames: u32,
    pub playback_delta_seconds: f32,
}

impl ScenePreviewOptions {
    pub fn new(
        mods_root: impl Into<PathBuf>,
        mod_id: impl Into<String>,
        scene_id: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            mods_root: mods_root.into(),
            active_mods: None,
            mod_id: mod_id.into(),
            scene_id: scene_id.into(),
            width,
            height,
            warmup_frames: 1,
            playback_delta_seconds: 1.0 / 60.0,
        }
    }

    pub fn with_active_mods(mut self, active_mods: impl Into<Vec<String>>) -> Self {
        self.active_mods = Some(active_mods.into());
        self
    }

    pub fn with_warmup_frames(mut self, warmup_frames: u32) -> Self {
        self.warmup_frames = warmup_frames;
        self
    }

    pub fn with_playback_delta_seconds(mut self, seconds: f32) -> Self {
        self.playback_delta_seconds = seconds.max(1.0 / 60.0).min(1.0);
        self
    }

    pub fn bootstrap_options(&self) -> BootstrapOptions {
        let mut options = BootstrapOptions::new(self.mods_root.clone())
            .with_startup_mod(self.mod_id.clone())
            .with_startup_scene(self.scene_id.clone())
            .with_dev_mode(false);

        if let Some(active_mods) = self.active_mods.clone() {
            options = options.with_active_mods(active_mods);
        }

        options
    }
}

#[derive(Debug, Clone)]
pub struct ScenePreviewFrame {
    pub width: u32,
    pub height: u32,
    pub pixels_rgba8: Vec<u8>,
    pub diagnostic_label: String,
}

pub struct ScenePreviewHost {
    options: ScenePreviewOptions,
    runtime: Option<Runtime>,
    summary: Option<BootstrapSummary>,
    runtime_ready_for_animation: bool,
    offscreen: Option<ScenePreviewOffscreen>,
}

struct ScenePreviewOffscreen {
    target: WgpuOffscreenTarget,
    renderer: WgpuSceneRenderer,
}

impl ScenePreviewHost {
    pub fn new(options: ScenePreviewOptions) -> Self {
        Self {
            options,
            runtime: None,
            summary: None,
            runtime_ready_for_animation: false,
            offscreen: None,
        }
    }

    pub fn bootstrap(&mut self) -> AmigoResult<&BootstrapSummary> {
        if self.summary.is_none() {
            let bootstrap = bootstrap_session_with_options(self.options.bootstrap_options())?;
            let summary = bootstrap.summary().clone();
            let (session, _) = bootstrap.into_parts();
            let runtime = session.into_runtime();
            amigo_runtime_bundles::set_runtime_ui_viewport_state(
                &runtime,
                self.options.width as f32,
                self.options.height as f32,
            )?;
            self.runtime = Some(runtime);
            self.summary = Some(summary);
        }

        Ok(self
            .summary
            .as_ref()
            .expect("preview summary is initialized"))
    }

    pub fn capture_rgba8(&mut self) -> AmigoResult<ScenePreviewFrame> {
        self.bootstrap()?;
        self.ensure_runtime_primed()?;
        let pixels_rgba8 = self.render_current_frame()?;
        let summary = self
            .summary
            .as_ref()
            .expect("preview summary is initialized");
        let mod_id = summary
            .startup_mod
            .as_deref()
            .unwrap_or(self.options.mod_id.as_str());
        let scene_id = summary
            .active_scene
            .as_deref()
            .unwrap_or(self.options.scene_id.as_str());
        Ok(ScenePreviewFrame {
            width: self.options.width,
            height: self.options.height,
            pixels_rgba8,
            diagnostic_label: format!("engine snapshot: {mod_id} / {scene_id}"),
        })
    }

    pub fn capture_next_frame(&mut self) -> AmigoResult<ScenePreviewFrame> {
        self.bootstrap()?;
        self.ensure_runtime_primed()?;
        self.advance_runtime_by(self.options.playback_delta_seconds)?;
        let pixels_rgba8 = self.render_current_frame()?;
        let summary = self
            .summary
            .as_ref()
            .expect("preview summary is initialized");
        let mod_id = summary
            .startup_mod
            .as_deref()
            .unwrap_or(self.options.mod_id.as_str());
        let scene_id = summary
            .active_scene
            .as_deref()
            .unwrap_or(self.options.scene_id.as_str());
        Ok(ScenePreviewFrame {
            width: self.options.width,
            height: self.options.height,
            pixels_rgba8,
            diagnostic_label: format!("engine frame: {mod_id} / {scene_id}"),
        })
    }

    fn advance_runtime_by(&mut self, seconds: f32) -> AmigoResult<()> {
        let step = 1.0 / 60.0;
        let steps = (seconds / step).round().max(1.0) as u32;

        for _ in 0..steps {
            self.tick_runtime_frame(step)?;
        }

        Ok(())
    }

    pub fn reset_runtime(&mut self) {
        self.runtime = None;
        self.summary = None;
        self.runtime_ready_for_animation = false;
    }

    fn ensure_runtime_primed(&mut self) -> AmigoResult<()> {
        if self.runtime_ready_for_animation {
            return Ok(());
        }

        self.warmup(self.options.warmup_frames)?;
        self.runtime_ready_for_animation = true;
        Ok(())
    }

    pub fn warmup(&mut self, frames: u32) -> AmigoResult<()> {
        self.bootstrap()?;
        for _ in 0..frames {
            self.tick_runtime_frame(self.options.playback_delta_seconds)?;
        }
        let updated = {
            let runtime = self.runtime()?;
            crate::summary::refresh_runtime_summary(runtime)?
        };
        self.summary = Some(updated);
        Ok(())
    }

    fn tick_runtime_frame(&mut self, delta_seconds: f32) -> AmigoResult<()> {
        {
            let runtime = self.runtime()?;
            if let Some(clock) = runtime.resolve::<amigo_session::RuntimeFrameClockService>() {
                clock.force_single_simulation_tick(delta_seconds);
            }
            let systems = crate::runtime_context::required::<SystemRegistry>(runtime)?;
            systems.run_phase(SystemPhase::PreUpdate, runtime)?;
            systems.run_phase(SystemPhase::FixedUpdate, runtime)?;
            systems.run_phase(SystemPhase::Update, runtime)?;
            systems.run_phase(SystemPhase::PostUpdate, runtime)?;
            amigo_runtime_bundles::clear_runtime_frame_transients(runtime);
            crate::orchestration::stabilize_runtime_queues(runtime)?;
        }
        Ok(())
    }

    fn render_current_frame(&mut self) -> AmigoResult<Vec<u8>> {
        let width = self.options.width;
        let height = self.options.height;
        self.ensure_offscreen(width, height)?;

        let runtime = self.runtime.as_ref().ok_or_else(|| {
            AmigoError::Message("scene preview runtime is not bootstrapped".to_owned())
        })?;
        let emergency_overlay = crate::render_runtime::emergency_overlay_lines(runtime);

        let offscreen = self.offscreen.as_mut().ok_or_else(|| {
            AmigoError::Message("scene preview offscreen is not initialized".to_owned())
        })?;

        amigo_runtime_bundles::render_wgpu_runtime_frame_to_offscreen(
            amigo_runtime_bundles::WgpuOffscreenRuntimeFrameInput {
                runtime,
                target: &mut offscreen.target,
                renderer: &mut offscreen.renderer,
                emergency_overlay: emergency_overlay.as_slice(),
            },
        )?;

        offscreen.target.read_rgba8_blocking()
    }

    fn ensure_offscreen(&mut self, width: u32, height: u32) -> AmigoResult<()> {
        let width = width.max(1);
        let height = height.max(1);
        let recreate = self
            .offscreen
            .as_ref()
            .is_none_or(|state| state.target.width != width || state.target.height != height);

        if recreate {
            let backend = WgpuRenderBackend::default();
            let target = backend.initialize_offscreen(width, height)?;
            let renderer = WgpuSceneRenderer::new_for_offscreen(&target);
            self.offscreen = Some(ScenePreviewOffscreen { target, renderer });
        }

        Ok(())
    }

    pub(crate) fn runtime(&self) -> AmigoResult<&Runtime> {
        self.runtime.as_ref().ok_or_else(|| {
            AmigoError::Message("scene preview runtime is not bootstrapped".to_owned())
        })
    }
    #[allow(dead_code)]
    fn runtime_mut(&mut self) -> AmigoResult<&mut Runtime> {
        self.runtime.as_mut().ok_or_else(|| {
            AmigoError::Message("scene preview runtime is not bootstrapped".to_owned())
        })
    }
}

pub fn capture_scene_preview(options: ScenePreviewOptions) -> AmigoResult<ScenePreviewFrame> {
    ScenePreviewHost::new(options).capture_rgba8()
}
