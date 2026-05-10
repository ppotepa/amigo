use super::*;
use amigo_runtime::{SystemPhase, SystemRegistry};

fn start_audio_output(runtime: &Runtime) -> AmigoResult<()> {
    let audio_backend = required::<AudioOutputBackendService>(runtime)?;
    match audio_backend.start_if_available() {
        Ok(AudioOutputStartStatus::Started) => {
            let snapshot = audio_backend.snapshot();
            println!(
                "audio init: backend={} device={} sample_rate={} channels={}",
                snapshot.backend_name,
                snapshot.device_name.as_deref().unwrap_or("unknown"),
                snapshot.sample_rate.unwrap_or_default(),
                snapshot.channels.unwrap_or_default()
            );
        }
        Ok(AudioOutputStartStatus::AlreadyStarted) => {}
        Ok(AudioOutputStartStatus::Unavailable) => {
            let snapshot = audio_backend.snapshot();
            println!(
                "audio init: backend={} unavailable ({})",
                snapshot.backend_name,
                snapshot
                    .last_error
                    .as_deref()
                    .unwrap_or("no audio output device")
            );
        }
        Err(error) => {
            println!("audio init failed: {error}");
        }
    }

    Ok(())
}

pub(crate) struct SummaryHostHandler {
    summary: BootstrapSummary,
    surface: Option<WgpuSurfaceState>,
    printed: bool,
}

impl SummaryHostHandler {
    pub(crate) fn new(summary: BootstrapSummary) -> Self {
        Self {
            summary,
            surface: None,
            printed: false,
        }
    }
}

pub(crate) struct InteractiveRuntimeHostHandler {
    pub(crate) runtime: Runtime,
    summary: BootstrapSummary,
    surface: Option<WgpuSurfaceState>,
    renderer: Option<WgpuSceneRenderer>,
    scene_ids: Vec<String>,
    printed_console_lines: usize,
    printed: bool,
}

impl InteractiveRuntimeHostHandler {
    pub(crate) fn new(runtime: Runtime, summary: BootstrapSummary) -> AmigoResult<Self> {
        let launch_selection = required::<LaunchSelection>(&runtime)?;
        let mod_catalog = required::<ModCatalog>(&runtime)?;
        let scene_ids =
            super::scene_ids_for_launch_selection(mod_catalog.as_ref(), launch_selection.as_ref());

        Ok(Self {
            runtime,
            printed_console_lines: summary.console_output.len(),
            summary,
            surface: None,
            renderer: None,
            scene_ids,
            printed: false,
        })
    }

    fn queue_scene_switch(&mut self, step: isize) -> AmigoResult<()> {
        let scene_service = required::<SceneService>(&self.runtime)?;
        let active_scene = scene_service.selected_scene();
        let Some(next_scene_id) = super::next_scene_id(
            &self.scene_ids,
            active_scene.as_ref().map(SceneKey::as_str),
            step,
        ) else {
            return Ok(());
        };

        required::<SceneCommandQueue>(&self.runtime)?.submit(SceneCommand::SelectScene {
            scene: SceneKey::new(next_scene_id.clone()),
        });

        Ok(())
    }

    fn queue_console_command(&mut self, line: &str) -> AmigoResult<()> {
        required::<DevConsoleQueue>(&self.runtime)?
            .submit(amigo_scripting_api::DevConsoleCommand::new(line));
        Ok(())
    }

    fn tick_runtime_pre_update(&self) -> AmigoResult<()> {
        let systems = required::<SystemRegistry>(&self.runtime)?;
        systems.run_phase(SystemPhase::PreUpdate, &self.runtime)?;
        Ok(())
    }

    fn tick_runtime_update(&self) -> AmigoResult<()> {
        let systems = required::<SystemRegistry>(&self.runtime)?;
        systems.run_phase(SystemPhase::Update, &self.runtime)?;

        Ok(())
    }

    fn tick_runtime_post_update(&self) -> AmigoResult<()> {
        let systems = required::<SystemRegistry>(&self.runtime)?;
        systems.run_phase(SystemPhase::PostUpdate, &self.runtime)
    }

    fn host_scene_switch_enabled(&self) -> bool {
        self.summary.startup_mod.as_deref() == Some("core-game") && self.scene_ids.len() > 1
    }

    fn handle_dev_console_input(&mut self, event: &InputEvent) -> AmigoResult<bool> {
        let console = required::<DevConsoleState>(&self.runtime)?;

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::Backquote,
                pressed: true,
            }
        ) {
            console.toggle_open();
            return Ok(true);
        }

        if !console.is_open() {
            return Ok(false);
        }

        match event {
            InputEvent::TextInput { text } => {
                console.push_input_text(text);
                Ok(true)
            }
            InputEvent::MouseWheel { delta_y } => {
                let rows = if *delta_y > 0.0 {
                    3
                } else if *delta_y < 0.0 {
                    -3
                } else {
                    0
                };
                console.scroll_output(rows);
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Backspace,
                pressed: true,
            } => {
                console.backspace_input();
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Enter,
                pressed: true,
            } => {
                let line = console.input();
                console.clear_input();
                if !line.trim().is_empty() {
                    console.reset_output_scroll();
                    required::<DevConsoleQueue>(&self.runtime)?
                        .submit(amigo_scripting_api::DevConsoleCommand::new(line));
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Escape,
                pressed: true,
            } => {
                console.set_open(false);
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Up,
                pressed: true,
            } => {
                if let Some(previous) = console.history_previous() {
                    console.set_input(previous);
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Down,
                pressed: true,
            } => {
                if let Some(next) = console.history_next() {
                    console.set_input(next);
                }
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn pump_runtime(&mut self) -> AmigoResult<()> {
        let previous_scene = self.summary.active_scene.clone();
        let previous_document = self.summary.loaded_scene_document.clone();
        let previous_entities = self.summary.scene_entities.clone();
        let updated = refresh_runtime_summary(&self.runtime)?;

        if updated.active_scene != previous_scene {
            println!(
                "active scene: {}",
                updated.active_scene.as_deref().unwrap_or("none")
            );
        }

        if updated.loaded_scene_document != previous_document {
            println!(
                "scene document: {}",
                updated
                    .loaded_scene_document
                    .as_ref()
                    .map(|document| format!(
                        "{}:{}",
                        document.source_mod,
                        document.relative_path.display()
                    ))
                    .unwrap_or_else(|| "none".to_owned())
            );
        }

        if updated.scene_entities != previous_entities {
            println!(
                "scene entities: {}",
                crate::app_helpers::display_string_list(&updated.scene_entities)
            );
        }

        for line in updated
            .console_output
            .iter()
            .skip(self.printed_console_lines)
        {
            println!("console: {line}");
        }

        self.printed_console_lines = updated.console_output.len();
        self.summary = updated;

        Ok(())
    }
}

impl HostHandler for SummaryHostHandler {
    fn config(&self) -> HostConfig {
        HostConfig {
            window: WindowDescriptor {
                title: "Amigo Hosted".to_owned(),
                ..WindowDescriptor::default()
            },
            exit_strategy: HostExitStrategy::AfterFirstRedraw,
        }
    }

    fn on_lifecycle(&mut self, event: HostLifecycleEvent) -> AmigoResult<HostControl> {
        if matches!(event, HostLifecycleEvent::WindowCreated) && !self.printed {
            println!("{}", self.summary);
            self.printed = true;
        }

        Ok(HostControl::Continue)
    }

    fn on_window_event(&mut self, event: WindowEvent) -> AmigoResult<HostControl> {
        if let WindowEvent::Resized(size) = event {
            if let Some(surface) = &mut self.surface {
                surface.resize(size);
            }
        }

        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(HostControl::Exit);
        }

        Ok(HostControl::Continue)
    }

    fn on_window_ready(&mut self, handles: WindowSurfaceHandles) -> AmigoResult<HostControl> {
        let backend = WgpuRenderBackend::default();
        let surface = backend.initialize_for_window(handles)?;

        println!(
            "render init: backend={} adapter={} adapter_backend={} device_type={} queue_ready={}",
            surface.report.backend_name,
            surface.report.adapter_name,
            surface.report.adapter_backend,
            surface.report.device_type,
            surface.report.queue_ready
        );

        self.surface = Some(surface);

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        if let Some(surface) = &mut self.surface {
            surface.render_default_frame()?;
        }

        Ok(HostControl::Continue)
    }
}

impl HostHandler for InteractiveRuntimeHostHandler {
    fn config(&self) -> HostConfig {
        HostConfig {
            window: WindowDescriptor {
                title: "Amigo Hosted Dev".to_owned(),
                ..WindowDescriptor::default()
            },
            exit_strategy: HostExitStrategy::Manual,
        }
    }

    fn on_lifecycle(&mut self, event: HostLifecycleEvent) -> AmigoResult<HostControl> {
        if matches!(event, HostLifecycleEvent::WindowCreated) && !self.printed {
            println!("{}", self.summary);
            if self.host_scene_switch_enabled() {
                println!(
                    "host controls: Left/Right switch scenes, Enter help, Space diagnostics, Escape exits"
                );
            } else {
                println!(
                    "host controls: arrow keys flow into InputState, Enter help, Space diagnostics, Escape exits"
                );
            }
            self.printed = true;
        }

        if matches!(event, HostLifecycleEvent::AboutToWait) {
            self.tick_runtime_pre_update()?;
            self.tick_runtime_update()?;
            self.pump_runtime()?;
            self.tick_runtime_post_update()?;
            if let Some(input_state) = self.runtime.resolve::<InputState>() {
                input_state.clear_frame_transients();
            }
            if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                ui_input.clear_frame_transients();
            }
        }

        Ok(HostControl::Continue)
    }

    fn on_input_event(&mut self, event: InputEvent) -> AmigoResult<HostControl> {
        if self.handle_dev_console_input(&event)? {
            return Ok(HostControl::Continue);
        }

        match event {
            InputEvent::CursorMoved { x, y } => {
                if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                    ui_input.set_mouse_position(x as f32, y as f32);
                }
            }
            InputEvent::MouseButton {
                button: amigo_input_api::MouseButton::Left,
                pressed,
            } => {
                if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                    ui_input.set_left_button(pressed);
                }
            }
            InputEvent::MouseWheel { delta_y } => {
                if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                    ui_input.add_mouse_wheel(delta_y);
                }
            }
            _ => {}
        }

        if let InputEvent::Key { key, pressed } = event {
            if let Some(input_state) = self.runtime.resolve::<InputState>() {
                input_state.set_key(key, pressed);
            }

            if key == KeyCode::Escape && pressed {
                return Ok(HostControl::Exit);
            }

            if !pressed {
                return Ok(HostControl::Continue);
            }

            if self.host_scene_switch_enabled() {
                match key {
                    KeyCode::Right | KeyCode::Down => self.queue_scene_switch(1)?,
                    KeyCode::Left | KeyCode::Up => self.queue_scene_switch(-1)?,
                    KeyCode::Enter => self.queue_console_command("help")?,
                    KeyCode::Space => self.queue_console_command("diagnostics")?,
                    _ => {}
                }

                return Ok(HostControl::Continue);
            }
        }

        match event {
            InputEvent::Key {
                key: KeyCode::Enter,
                pressed: true,
            } => self.queue_console_command("help")?,
            InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            } => self.queue_console_command("diagnostics")?,
            _ => {}
        }

        Ok(HostControl::Continue)
    }

    fn on_window_event(&mut self, event: WindowEvent) -> AmigoResult<HostControl> {
        if let WindowEvent::Resized(size) = event {
            if let Some(surface) = &mut self.surface {
                surface.resize(size);
            }
            required::<systems::UiInputViewportState>(&self.runtime)?.set(Some(
                UiViewportSize::new(size.width as f32, size.height as f32),
            ));
        }

        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(HostControl::Exit);
        }

        Ok(HostControl::Continue)
    }

    fn on_window_ready(&mut self, handles: WindowSurfaceHandles) -> AmigoResult<HostControl> {
        let backend = WgpuRenderBackend::default();
        let surface = backend.initialize_for_window(handles)?;
        let renderer = WgpuSceneRenderer::new(&surface);

        println!(
            "render init: backend={} adapter={} adapter_backend={} device_type={} queue_ready={}",
            surface.report.backend_name,
            surface.report.adapter_name,
            surface.report.adapter_backend,
            surface.report.device_type,
            surface.report.queue_ready
        );

        self.surface = Some(surface);
        self.renderer = Some(renderer);
        if let Some(surface) = &self.surface {
            let size = surface.size();
            required::<systems::UiInputViewportState>(&self.runtime)?.set(Some(
                UiViewportSize::new(size.width as f32, size.height as f32),
            ));
        }
        start_audio_output(&self.runtime)?;
        self.summary = refresh_runtime_summary(&self.runtime)?;

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        if let Some(surface) = &mut self.surface {
            if let Some(renderer) = &mut self.renderer {
                let scene = required::<SceneService>(&self.runtime)?;
                let assets = required::<AssetCatalog>(&self.runtime)?;
                let tilemaps = required::<TileMap2dSceneService>(&self.runtime)?;
                let sprites = required::<SpriteSceneService>(&self.runtime)?;
                let layered_images =
                    required::<amigo_2d_layered_image::LayeredImageSceneService>(&self.runtime)?;
                let render_layers =
                    required::<amigo_2d_composition::RenderLayer2dSceneService>(&self.runtime)?;
                let light_routes =
                    required::<amigo_2d_composition::LightRoute2dSceneService>(&self.runtime)?;
                let global_lights =
                    required::<amigo_2d_lighting::GlobalLight2dSceneService>(&self.runtime)?;
                let lightmaps =
                    required::<amigo_2d_lighting::LightMap2dSceneService>(&self.runtime)?;
                let light_groups =
                    required::<amigo_2d_lighting::LightGroup2dSceneService>(&self.runtime)?;
                let text2d = required::<Text2dSceneService>(&self.runtime)?;
                let vectors = required::<VectorSceneService>(&self.runtime)?;
                let particles = required::<Particle2dSceneService>(&self.runtime)?;
                let meshes = required::<MeshSceneService>(&self.runtime)?;
                let text3d = required::<Text3dSceneService>(&self.runtime)?;
                let materials = required::<MaterialSceneService>(&self.runtime)?;
                let ui_scene = required::<UiSceneService>(&self.runtime)?;
                let ui_state = required::<UiStateService>(&self.runtime)?;
                let ui_theme = required::<UiThemeService>(&self.runtime)?;
                let dev_console_state = required::<DevConsoleState>(&self.runtime)?;
                let ui_viewport_state = required::<systems::UiInputViewportState>(&self.runtime)?;
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
                        dev_console_state: dev_console_state.as_ref(),
                        ui_viewport_state: ui_viewport_state.as_ref(),
                    });
                if let Ok(stats_service) =
                    required::<crate::render_runtime::RenderFrameStatsService>(&self.runtime)
                {
                    let size = surface.size();
                    let previous = stats_service.snapshot();
                    stats_service.set(crate::render_runtime::RenderFrameStats {
                        frame_index: previous.frame_index + 1,
                        window_width: size.width,
                        window_height: size.height,
                        world_2d_tilemaps: render_packet.world_2d_tilemaps().len(),
                        world_2d_sprites: render_packet.world_2d_sprites().len(),
                        world_2d_layered_images: render_packet.world_2d_layered_images().len(),
                        world_2d_render_layers: render_packet.world_2d_render_layers().len(),
                        world_2d_light_routes: render_packet.world_2d_light_routes().len(),
                        world_2d_global_lights: render_packet.world_2d_global_lights().len(),
                        world_2d_lightmaps: render_packet.world_2d_lightmaps().len(),
                        world_2d_light_groups: render_packet.world_2d_light_groups().len(),
                        world_2d_vectors: render_packet.world_2d_vectors().len(),
                        world_2d_text: render_packet.world_2d_text().len(),
                        world_2d_particles: render_packet.world_2d_particles().len(),
                        world_3d_meshes: render_packet.world_3d_meshes().len(),
                        world_3d_materials: render_packet.world_3d_materials().len(),
                        world_3d_text: render_packet.world_3d_text().len(),
                        ui_overlays: render_packet.overlay().len(),
                    });
                }
                let extracted_tilemaps =
                    crate::render_runtime::build_tilemap_scene_service_from_packet(&render_packet);
                let extracted_sprites =
                    crate::render_runtime::build_sprite_scene_service_from_packet(&render_packet);
                let extracted_layered_images =
                    crate::render_runtime::build_layered_image_scene_service_from_packet(
                        &render_packet,
                    );
                let extracted_render_layers =
                    crate::render_runtime::build_render_layer2d_scene_service_from_packet(
                        &render_packet,
                    );
                let extracted_light_routes =
                    crate::render_runtime::build_light_route2d_scene_service_from_packet(
                        &render_packet,
                    );
                let extracted_global_lights =
                    crate::render_runtime::build_global_light2d_scene_service_from_packet(
                        &render_packet,
                    );
                let extracted_lightmaps =
                    crate::render_runtime::build_lightmap2d_scene_service_from_packet(
                        &render_packet,
                    );
                let extracted_text2d =
                    crate::render_runtime::build_text2d_scene_service_from_packet(&render_packet);
                let extracted_vectors =
                    crate::render_runtime::build_vector_scene_service_from_packet(&render_packet);
                renderer.render_scene_with_ui_documents_and_3d_commands(
                    surface,
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
                    render_packet.overlay(),
                )?;
            } else {
                surface.render_default_frame()?;
            }
        }

        Ok(HostControl::Continue)
    }
}
