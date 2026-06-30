use std::path::PathBuf;

use amigo_assets::{AssetCatalog, AssetKey, PreparedAssetKind};
use amigo_core::{AmigoError, AmigoResult};
use amigo_math::Vec3;
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

    pub fn prime(&mut self) -> AmigoResult<()> {
        self.bootstrap()?;
        self.ensure_runtime_primed()
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

    pub fn apply_mesh3d_npr_preset(
        &mut self,
        entity_name: impl Into<String>,
        preset_id: impl Into<String>,
    ) -> AmigoResult<()> {
        self.submit_script_command(
            "3d.mesh",
            "apply_npr_preset",
            vec![entity_name.into(), preset_id.into()],
        )
    }

    pub fn set_mesh3d_asset(
        &mut self,
        entity_name: impl Into<String>,
        mesh_key: impl Into<String>,
    ) -> AmigoResult<()> {
        self.submit_script_command(
            "3d.mesh",
            "set_mesh_asset",
            vec![entity_name.into(), mesh_key.into()],
        )
    }

    pub fn set_mesh3d_npr_render_strategy(
        &mut self,
        entity_name: impl Into<String>,
        strategy: impl Into<String>,
    ) -> AmigoResult<()> {
        self.submit_script_command(
            "3d.mesh",
            "set_npr_render_strategy",
            vec![entity_name.into(), strategy.into()],
        )
    }

    pub fn set_mesh3d_npr_gpu_debug_mode(
        &mut self,
        entity_name: impl Into<String>,
        debug_mode: impl Into<String>,
    ) -> AmigoResult<()> {
        self.submit_script_command(
            "3d.mesh",
            "set_npr_gpu_debug_mode",
            vec![entity_name.into(), debug_mode.into()],
        )
    }

    pub fn set_scene_entity_visible(
        &mut self,
        entity_name: impl AsRef<str>,
        visible: bool,
    ) -> AmigoResult<()> {
        self.bootstrap()?;
        let runtime = self.runtime()?;
        let scene_service = crate::runtime_context::required::<amigo_scene::SceneService>(runtime)?;
        if scene_service.set_visible(entity_name.as_ref(), visible) {
            Ok(())
        } else {
            Err(AmigoError::Message(format!(
                "scene entity `{}` was not found",
                entity_name.as_ref()
            )))
        }
    }

    pub fn set_scene_entity_transform_overrides(
        &mut self,
        entity_name: impl AsRef<str>,
        scale: Option<f32>,
        translation: Option<Vec3>,
        rotation_degrees: Option<Vec3>,
    ) -> AmigoResult<()> {
        self.bootstrap()?;
        let runtime = self.runtime()?;
        let scene_service = crate::runtime_context::required::<amigo_scene::SceneService>(runtime)?;
        let Some(mut transform) = scene_service.transform_of(entity_name.as_ref()) else {
            return Err(AmigoError::Message(format!(
                "scene entity `{}` was not found",
                entity_name.as_ref()
            )));
        };

        if let Some(scale) = scale {
            transform.scale = Vec3::new(scale, scale, scale);
        }
        if let Some(translation) = translation {
            transform.translation = translation;
        }
        if let Some(rotation_degrees) = rotation_degrees {
            transform.rotation_euler = Vec3::new(
                rotation_degrees.x.to_radians(),
                rotation_degrees.y.to_radians(),
                rotation_degrees.z.to_radians(),
            );
        }

        if scene_service.set_transform(entity_name.as_ref(), transform) {
            Ok(())
        } else {
            Err(AmigoError::Message(format!(
                "scene entity `{}` was not found",
                entity_name.as_ref()
            )))
        }
    }

    pub fn submit_script_command(
        &mut self,
        namespace: impl Into<String>,
        name: impl Into<String>,
        arguments: Vec<String>,
    ) -> AmigoResult<()> {
        self.bootstrap()?;
        let runtime = self.runtime()?;
        let script_commands =
            crate::runtime_context::required::<amigo_scripting_api::ScriptCommandQueue>(runtime)?;
        script_commands.submit(amigo_scripting_api::ScriptCommand::new(
            namespace.into(),
            name.into(),
            arguments,
        ));
        crate::orchestration::stabilize_runtime_queues(runtime)?;
        Ok(())
    }

    pub fn mesh3d_asset_key_for_query(&mut self, query: &str) -> AmigoResult<Option<String>> {
        self.bootstrap()?;
        let query = query.trim();
        if query.is_empty() {
            return Ok(None);
        }

        let runtime = self.runtime()?;
        let catalog = crate::runtime_context::required::<AssetCatalog>(runtime)?;
        if catalog.is_prepared(&AssetKey::new(query.to_owned())) {
            return Ok(Some(query.to_owned()));
        }

        let query_lower = query.to_ascii_lowercase();
        let mut mesh_assets = catalog
            .prepared_assets()
            .into_iter()
            .filter(|asset| matches!(asset.kind, PreparedAssetKind::Mesh3d))
            .collect::<Vec<_>>();
        mesh_assets.sort_by(|left, right| left.key.cmp(&right.key));

        let mut matches = mesh_assets
            .iter()
            .filter_map(|asset| {
                mesh3d_asset_query_score(
                    asset.key.as_str(),
                    asset.label.as_deref(),
                    query,
                    &query_lower,
                )
                .map(|score| (score, asset.key.as_str().to_owned()))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

        Ok(matches.into_iter().next().map(|(_, key)| key))
    }

    #[cfg(test)]
    pub(crate) fn register_mesh3d_npr_preset_for_test(
        &mut self,
        preset_id: impl Into<String>,
        settings: amigo_render_api::NprLineSettings3d,
    ) -> AmigoResult<()> {
        self.bootstrap()?;
        let runtime = self.runtime()?;
        let mesh_service =
            crate::runtime_context::required::<amigo_3d_mesh::MeshSceneService>(runtime)?;
        mesh_service.register_npr_preset(preset_id, settings);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn mesh3d_npr_preset_for_test(
        &mut self,
        preset_id: &str,
    ) -> AmigoResult<amigo_render_api::NprLineSettings3d> {
        self.bootstrap()?;
        let runtime = self.runtime()?;
        let mesh_service =
            crate::runtime_context::required::<amigo_3d_mesh::MeshSceneService>(runtime)?;
        mesh_service
            .npr_preset(preset_id)
            .ok_or_else(|| AmigoError::Message(format!("missing Mesh3D NPR preset `{preset_id}`")))
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

    fn runtime(&self) -> AmigoResult<&Runtime> {
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

fn mesh3d_asset_query_score(
    key: &str,
    label: Option<&str>,
    query: &str,
    query_lower: &str,
) -> Option<u8> {
    if key == query {
        return Some(0);
    }

    let key_lower = key.to_ascii_lowercase();
    let authored_mesh = key_lower.contains("/meshes/");
    let discovered_mesh = key_lower.contains("/discovered-models/");
    let source_penalty = if authored_mesh {
        0
    } else if discovered_mesh {
        2
    } else {
        1
    };

    if authored_mesh
        && (key_lower.ends_with(&format!("/meshes/{query_lower}"))
            || key_lower.ends_with(&format!("/meshes/{query_lower}.glb")))
    {
        return Some(5);
    }

    if authored_mesh && key_lower.contains(query_lower) {
        return Some(10);
    }

    if label.is_some_and(|label| label.to_ascii_lowercase() == query_lower) {
        return Some(20 + source_penalty);
    }

    if label.is_some_and(|label| label.to_ascii_lowercase().contains(query_lower))
        || key_lower.contains(query_lower)
    {
        return Some(40 + source_penalty);
    }

    None
}

pub fn capture_scene_preview(options: ScenePreviewOptions) -> AmigoResult<ScenePreviewFrame> {
    ScenePreviewHost::new(options).capture_rgba8()
}
