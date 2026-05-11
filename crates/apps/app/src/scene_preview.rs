use std::path::PathBuf;

use amigo_2d_layered_image::LayeredImageSceneService;
use amigo_2d_particles::Particle2dSceneService;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::TileMap2dSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_3d_material::MaterialSceneService;
use amigo_3d_mesh::MeshSceneService;
use amigo_3d_text::Text3dSceneService;
use amigo_assets::AssetCatalog;
use amigo_core::{AmigoError, AmigoResult};
use amigo_input_api::InputState;
use amigo_render_wgpu::{
    UiViewportSize, WgpuOffscreenTarget, WgpuRenderBackend, WgpuSceneRenderer,
};
use amigo_runtime::Runtime;
use amigo_runtime::{SystemPhase, SystemRegistry};
use amigo_scene::SceneService;
use amigo_ui::{UiInputService, UiSceneService, UiStateService, UiThemeService};

use crate::{BootstrapOptions, BootstrapSummary, bootstrap_with_options};

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
            playback_delta_seconds: crate::systems::HOST_DELTA_SECONDS,
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
        self.playback_delta_seconds = seconds.max(crate::systems::HOST_DELTA_SECONDS).min(1.0);
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
            let (runtime, summary) = bootstrap_with_options(self.options.bootstrap_options())?;
            crate::runtime_context::required::<crate::systems::UiInputViewportState>(&runtime)?
                .set(Some(UiViewportSize::new(
                    self.options.width as f32,
                    self.options.height as f32,
                )));
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
        let step = crate::systems::HOST_DELTA_SECONDS;
        let steps = (seconds / step).round().max(1.0) as u32;

        for _ in 0..steps {
            self.tick_runtime_frame()?;
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
            self.tick_runtime_frame()?;
        }
        Ok(())
    }

    fn tick_runtime_frame(&mut self) -> AmigoResult<()> {
        let updated = {
            let runtime = self.runtime()?;
            let systems = crate::runtime_context::required::<SystemRegistry>(runtime)?;
            systems.run_phase(SystemPhase::PreUpdate, runtime)?;
            systems.run_phase(SystemPhase::FixedUpdate, runtime)?;
            systems.run_phase(SystemPhase::Update, runtime)?;
            systems.run_phase(SystemPhase::PostUpdate, runtime)?;
            if let Some(input_state) = runtime.resolve::<InputState>() {
                input_state.clear_frame_transients();
            }
            if let Some(ui_input) = runtime.resolve::<UiInputService>() {
                ui_input.clear_frame_transients();
            }
            crate::summary::refresh_runtime_summary(runtime)?
        };
        self.summary = Some(updated);
        Ok(())
    }

    fn render_current_frame(&mut self) -> AmigoResult<Vec<u8>> {
        let width = self.options.width;
        let height = self.options.height;
        self.ensure_offscreen(width, height)?;

        let runtime = self.runtime()?;
        let scene = crate::runtime_context::required::<SceneService>(runtime)?;
        let assets = crate::runtime_context::required::<AssetCatalog>(runtime)?;
        let tilemaps = crate::runtime_context::required::<TileMap2dSceneService>(runtime)?;
        let sprites = crate::runtime_context::required::<SpriteSceneService>(runtime)?;
        let layered_images = crate::runtime_context::required::<LayeredImageSceneService>(runtime)?;
        let render_layers = crate::runtime_context::required::<
            amigo_2d_composition::RenderLayer2dSceneService,
        >(runtime)?;
        let light_routes = crate::runtime_context::required::<
            amigo_2d_composition::LightRoute2dSceneService,
        >(runtime)?;
        let global_lights = crate::runtime_context::required::<
            amigo_2d_lighting::GlobalLight2dSceneService,
        >(runtime)?;
        let lightmaps =
            crate::runtime_context::required::<amigo_2d_lighting::LightMap2dSceneService>(runtime)?;
        let light_groups = crate::runtime_context::required::<
            amigo_2d_lighting::LightGroup2dSceneService,
        >(runtime)?;
        let text2d = crate::runtime_context::required::<Text2dSceneService>(runtime)?;
        let vectors = crate::runtime_context::required::<VectorSceneService>(runtime)?;
        let particles = crate::runtime_context::required::<Particle2dSceneService>(runtime)?;
        let meshes = crate::runtime_context::required::<MeshSceneService>(runtime)?;
        let text3d = crate::runtime_context::required::<Text3dSceneService>(runtime)?;
        let materials = crate::runtime_context::required::<MaterialSceneService>(runtime)?;
        let ui_scene = crate::runtime_context::required::<UiSceneService>(runtime)?;
        let ui_state = crate::runtime_context::required::<UiStateService>(runtime)?;
        let ui_theme = crate::runtime_context::required::<UiThemeService>(runtime)?;
        let post_fx_service =
            crate::runtime_context::required::<amigo_2d_post_fx::PostFx2dService>(runtime)?;
        let dev_console_state =
            crate::runtime_context::required::<amigo_scripting_api::DevConsoleState>(runtime)?;
        let dev_console_completion =
            crate::runtime_context::required::<crate::dev_console::completion::ConsoleCompletionState>(runtime)?;
        let debug_overlay_service =
            crate::runtime_context::required::<crate::debug_overlay::DebugOverlayService>(
                runtime,
            )?;
        let ui_viewport_state =
            crate::runtime_context::required::<crate::systems::UiInputViewportState>(runtime)?;
        let render_packet = crate::render_runtime::default_app_render_extractor_registry()
            .extract_all(&crate::render_runtime::AppRenderExtractContext {
                scene_service: scene.as_ref(),
                tilemap_scene_service: tilemaps.as_ref(),
                sprite_scene_service: sprites.as_ref(),
                layered_image_scene_service: layered_images.as_ref(),
                render_layer2d_scene_service: render_layers.as_ref(),
                light_route2d_scene_service: light_routes.as_ref(),
                global_light2d_scene_service: global_lights.as_ref(),
                lightmap2d_scene_service: lightmaps.as_ref(),
                light_group2d_scene_service: light_groups.as_ref(),
                text2d_scene_service: text2d.as_ref(),
                vector_scene_service: vectors.as_ref(),
                particle2d_scene_service: particles.as_ref(),
                mesh_scene_service: meshes.as_ref(),
                material_scene_service: materials.as_ref(),
                text3d_scene_service: text3d.as_ref(),
                ui_scene_service: ui_scene.as_ref(),
                ui_state_service: ui_state.as_ref(),
                ui_theme_service: ui_theme.as_ref(),
                post_fx_service: post_fx_service.as_ref(),
                dev_console_state: dev_console_state.as_ref(),
                dev_console_completion: dev_console_completion.as_ref(),
                debug_overlay_service: debug_overlay_service.as_ref(),
                ui_viewport_state: ui_viewport_state.as_ref(),
            });
        let extracted_tilemaps =
            crate::render_runtime::build_tilemap_scene_service_from_packet(&render_packet);
        let extracted_sprites =
            crate::render_runtime::build_sprite_scene_service_from_packet(&render_packet);
        let extracted_layered_images =
            crate::render_runtime::build_layered_image_scene_service_from_packet(&render_packet);
        let extracted_render_layers =
            crate::render_runtime::build_render_layer2d_scene_service_from_packet(&render_packet);
        let extracted_light_routes =
            crate::render_runtime::build_light_route2d_scene_service_from_packet(&render_packet);
        let extracted_global_lights =
            crate::render_runtime::build_global_light2d_scene_service_from_packet(&render_packet);
        let extracted_lightmaps =
            crate::render_runtime::build_lightmap2d_scene_service_from_packet(&render_packet);
        let extracted_text2d =
            crate::render_runtime::build_text2d_scene_service_from_packet(&render_packet);
        let extracted_vectors =
            crate::render_runtime::build_vector_scene_service_from_packet(&render_packet);

        let offscreen = self.offscreen.as_mut().ok_or_else(|| {
            AmigoError::Message("scene preview offscreen is not initialized".to_owned())
        })?;

        let ui_overlay_documents = render_packet.overlay();
        offscreen
            .renderer
            .render_scene_with_ui_documents_and_3d_commands_offscreen(
                &mut offscreen.target,
                scene.as_ref(),
                assets.as_ref(),
                &extracted_tilemaps,
                &extracted_sprites,
                &extracted_layered_images,
                &extracted_global_lights,
                &extracted_lightmaps,
                &extracted_text2d,
                &extracted_vectors,
                render_packet.world_3d_meshes(),
                render_packet.world_3d_materials(),
                Some(render_packet.world_3d_text()),
                extracted_render_layers.commands().as_slice(),
                extracted_light_routes.commands().as_slice(),
                render_packet.world_2d_light_groups(),
                render_packet.world_2d_particles(),
                &ui_overlay_documents,
            )?;

        offscreen.target.read_rgba8()
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

pub fn capture_scene_preview(options: ScenePreviewOptions) -> AmigoResult<ScenePreviewFrame> {
    ScenePreviewHost::new(options).capture_rgba8()
}
