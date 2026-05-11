use super::*;
use amigo_runtime::{SystemPhase, SystemRegistry};
use amigo_session::RuntimeSession;

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
    pub(crate) session: RuntimeSession,
    summary: BootstrapSummary,
    surface: Option<WgpuSurfaceState>,
    renderer: Option<WgpuSceneRenderer>,
    scene_ids: Vec<String>,
    printed_console_lines: usize,
    printed: bool,
}

impl InteractiveRuntimeHostHandler {
    pub(crate) fn new(session: RuntimeSession, summary: BootstrapSummary) -> AmigoResult<Self> {
        let runtime = session.runtime();
        let launch_selection = required::<LaunchSelection>(runtime)?;
        let mod_catalog = required::<ModCatalog>(runtime)?;
        let scene_ids =
            super::scene_ids_for_launch_selection(mod_catalog.as_ref(), launch_selection.as_ref());

        Ok(Self {
            session,
            printed_console_lines: summary.console_output.len(),
            summary,
            surface: None,
            renderer: None,
            scene_ids,
            printed: false,
        })
    }

    fn runtime(&self) -> &Runtime {
        self.session.runtime()
    }

    fn queue_scene_switch(&mut self, step: isize) -> AmigoResult<()> {
        let scene_service = required::<SceneService>(self.runtime())?;
        let active_scene = scene_service.selected_scene();
        let Some(next_scene_id) = super::next_scene_id(
            &self.scene_ids,
            active_scene.as_ref().map(SceneKey::as_str),
            step,
        ) else {
            return Ok(());
        };

        required::<SceneCommandQueue>(self.runtime())?.submit(SceneCommand::SelectScene {
            scene: SceneKey::new(next_scene_id.clone()),
        });

        Ok(())
    }

    fn queue_console_command(&mut self, line: &str) -> AmigoResult<()> {
        required::<DevConsoleQueue>(self.runtime())?
            .submit(amigo_scripting_api::DevConsoleCommand::new(line));
        Ok(())
    }

    fn tick_runtime_pre_update(&self) -> AmigoResult<()> {
        let systems = required::<SystemRegistry>(self.runtime())?;
        systems.run_phase(SystemPhase::PreUpdate, self.runtime())?;
        Ok(())
    }

    fn tick_runtime_update(&self) -> AmigoResult<()> {
        let systems = required::<SystemRegistry>(self.runtime())?;
        systems.run_phase(SystemPhase::Update, self.runtime())?;

        Ok(())
    }

    fn tick_runtime_post_update(&self) -> AmigoResult<()> {
        let systems = required::<SystemRegistry>(self.runtime())?;
        systems.run_phase(SystemPhase::PostUpdate, self.runtime())
    }

    fn host_scene_switch_enabled(&self) -> bool {
        self.summary.startup_mod.as_deref() == Some("core-game") && self.scene_ids.len() > 1
    }

    fn handle_dev_console_input(&mut self, event: &InputEvent) -> AmigoResult<bool> {
        let console = required::<DevConsoleState>(self.runtime())?;
        let completion =
            required::<crate::dev_console::completion::ConsoleCompletionState>(self.runtime())?;
        let registry =
            required::<crate::dev_console::registry::ConsoleCommandRegistry>(self.runtime())?;

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::Backquote,
                pressed: true,
            }
        ) {
            console.toggle_open();
            if console.is_open() {
                completion.refresh(&console.input(), registry.as_ref());
            } else {
                completion.clear();
            }
            return Ok(true);
        }

        if !console.is_open() {
            return Ok(false);
        }

        match event {
            InputEvent::TextInput { text } => {
                console.push_input_text(text);
                completion.refresh(&console.input(), registry.as_ref());
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
                completion.refresh(&console.input(), registry.as_ref());
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Tab,
                pressed: true,
            } => {
                completion.refresh(&console.input(), registry.as_ref());
                if let Some(next_input) = completion.accept_tab(&console.input()) {
                    console.set_input(next_input);
                    completion.refresh(&console.input(), registry.as_ref());
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Enter,
                pressed: true,
            } => {
                let line = console.input();
                completion.clear();
                console.clear_input();
                if !line.trim().is_empty() {
                    console.reset_output_scroll();
                    required::<DevConsoleQueue>(self.runtime())?
                        .submit(amigo_scripting_api::DevConsoleCommand::new(line));
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Escape,
                pressed: true,
            } => {
                if completion.snapshot().is_some() {
                    completion.clear();
                } else {
                    console.set_open(false);
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Up,
                pressed: true,
            } => {
                if completion.select_previous() {
                    return Ok(true);
                }
                if let Some(previous) = console.history_previous() {
                    console.set_input(previous);
                    completion.refresh(&console.input(), registry.as_ref());
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Down,
                pressed: true,
            } => {
                if completion.select_next() {
                    return Ok(true);
                }
                if let Some(next) = console.history_next() {
                    console.set_input(next);
                    completion.refresh(&console.input(), registry.as_ref());
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
        let updated = refresh_runtime_summary(self.runtime())?;

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
            if let Some(input_state) = self.runtime().resolve::<InputState>() {
                input_state.clear_frame_transients();
            }
            if let Some(ui_input) = self.runtime().resolve::<UiInputService>() {
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
                if let Some(ui_input) = self.runtime().resolve::<UiInputService>() {
                    ui_input.set_mouse_position(x as f32, y as f32);
                }
            }
            InputEvent::MouseButton {
                button: amigo_input_api::MouseButton::Left,
                pressed,
            } => {
                if let Some(ui_input) = self.runtime().resolve::<UiInputService>() {
                    ui_input.set_left_button(pressed);
                }
            }
            InputEvent::MouseWheel { delta_y } => {
                if let Some(ui_input) = self.runtime().resolve::<UiInputService>() {
                    ui_input.add_mouse_wheel(delta_y);
                }
            }
            _ => {}
        }

        if let InputEvent::Key { key, pressed } = event {
            if let Some(input_state) = self.runtime().resolve::<InputState>() {
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
            required::<systems::UiInputViewportState>(self.runtime())?.set(Some(
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
            required::<systems::UiInputViewportState>(self.runtime())?.set(Some(
                UiViewportSize::new(size.width as f32, size.height as f32),
            ));
        }
        start_audio_output(self.runtime())?;
        self.summary = refresh_runtime_summary(self.runtime())?;

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        let runtime = self.session.runtime();
        if let Some(surface) = &mut self.surface {
            if let Some(renderer) = &mut self.renderer {
                let scene = required::<SceneService>(runtime)?;
                let assets = required::<AssetCatalog>(runtime)?;
                let tilemaps = required::<TileMap2dSceneService>(runtime)?;
                let sprites = required::<SpriteSceneService>(runtime)?;
                let layered_images =
                    required::<amigo_2d_layered_image::LayeredImageSceneService>(runtime)?;
                let render_layers =
                    required::<amigo_2d_composition::RenderLayer2dSceneService>(runtime)?;
                let light_routes =
                    required::<amigo_2d_composition::LightRoute2dSceneService>(runtime)?;
                let global_lights =
                    required::<amigo_2d_lighting::GlobalLight2dSceneService>(runtime)?;
                let lightmaps =
                    required::<amigo_2d_lighting::LightMap2dSceneService>(runtime)?;
                let light_groups =
                    required::<amigo_2d_lighting::LightGroup2dSceneService>(runtime)?;
                let text2d = required::<Text2dSceneService>(runtime)?;
                let vectors = required::<VectorSceneService>(runtime)?;
                let particles = required::<Particle2dSceneService>(runtime)?;
                let meshes = required::<MeshSceneService>(runtime)?;
                let text3d = required::<Text3dSceneService>(runtime)?;
                let materials = required::<MaterialSceneService>(runtime)?;
                let ui_scene = required::<UiSceneService>(runtime)?;
                let ui_state = required::<UiStateService>(runtime)?;
                let ui_theme = required::<UiThemeService>(runtime)?;
                let post_fx_service = required::<amigo_2d_post_fx::PostFx2dService>(runtime)?;
                let dev_console_state = required::<DevConsoleState>(runtime)?;
                let dev_console_completion = required::<
                    crate::dev_console::completion::ConsoleCompletionState,
                >(runtime)?;
                let debug_overlay_service =
                    required::<crate::debug_overlay::DebugOverlayService>(runtime)?;
                let ui_viewport_state = required::<systems::UiInputViewportState>(runtime)?;
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
                let surface_size = surface.size();
                let composition_plan =
                    crate::render_runtime::AppFrameCompositionBuilder::build(&render_packet);
                let frame_graph = crate::render_runtime::build_frame_graph_from_plan(
                    &composition_plan,
                    crate::render_runtime::AppFrameGraphBuildInfo {
                        width: surface_size.width,
                        height: surface_size.height,
                    },
                );
                if let Ok(render_diagnostics) = required::<
                    crate::render_runtime::RenderCompositionDiagnosticsService,
                >(runtime)
                {
                    render_diagnostics.set(&composition_plan, &frame_graph);
                }
                if let Ok(stats_service) =
                    required::<crate::render_runtime::RenderFrameStatsService>(runtime)
                {
                    let previous = stats_service.snapshot();
                    let stats = crate::render_runtime::RenderFrameStats {
                        frame_index: previous.frame_index + 1,
                        window_width: surface_size.width,
                        window_height: surface_size.height,
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
                        game_ui_overlays: render_packet.game_ui_overlay().len(),
                        debug_overlays: render_packet.debug_overlay().len(),
                        ui_overlays: render_packet.all_overlay_count(),
                        render_graph_nodes: frame_graph.nodes.len(),
                        post_fx_effects: render_packet
                            .post_fx_stack()
                            .map(|stack| stack.effects.len())
                            .unwrap_or(0),
                    };
                    stats_service.set(stats.clone());
                    debug_overlay_service.record_render_frame(stats);
                }
                if let Ok(scheduling) =
                    required::<crate::scheduling::AppSchedulingService>(runtime)
                {
                    debug_overlay_service.record_scheduling_stats(scheduling.stats());
                }
                if let Ok(audio_output) = required::<AudioOutputBackendService>(runtime) {
                    let audio_snapshot = audio_output.snapshot();
                    let (master_volume, active_sources, pending_commands, bus_count) =
                        if let Ok(audio_state) = required::<AudioStateService>(runtime) {
                            (
                                audio_state.master_volume(),
                                audio_state.playing_sources().len(),
                                audio_state.pending_runtime_commands().len(),
                                audio_state.bus_volumes().len(),
                            )
                        } else {
                            (1.0, 0, 0, 0)
                        };
                    debug_overlay_service.record_audio_snapshot(
                        audio_snapshot,
                        master_volume,
                        active_sources,
                        pending_commands,
                        bus_count,
                    );
                }
                if let Ok(input_state) = required::<InputState>(runtime) {
                    let pressed_keys = input_state
                        .pressed_keys()
                        .into_iter()
                        .map(|key| format!("{key:?}"))
                        .collect::<Vec<_>>();
                    let backend_name = runtime
                        .resolve::<InputServiceInfo>()
                        .map(|info| info.backend_name.to_owned());
                    let (active_map, active_actions) =
                        if let Ok(actions) = required::<InputActionService>(runtime) {
                            let active_map = actions.active_map_id();
                            let active_actions = active_map
                                .as_deref()
                                .and_then(|map_id| actions.map(map_id))
                                .map(|map| {
                                    let mut names = map
                                        .actions
                                        .keys()
                                        .filter_map(|action| {
                                            let name = action.as_str();
                                            actions
                                                .down(input_state.as_ref(), name)
                                                .then(|| name.to_owned())
                                        })
                                        .collect::<Vec<_>>();
                                    names.sort();
                                    names
                                })
                                .unwrap_or_default();
                            (active_map, active_actions)
                        } else {
                            (None, Vec::new())
                        };
                    debug_overlay_service.record_input_snapshot(
                        backend_name,
                        pressed_keys,
                        active_map,
                        active_actions,
                    );
                }
                debug_overlay_service.record_particle_snapshot(
                    particles.emitters().len(),
                    particles
                        .emitters()
                        .iter()
                        .filter(|emitter| particles.is_active(&emitter.entity_name))
                        .count(),
                );
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
                if let Ok(post_fx_service) =
                    required::<amigo_2d_post_fx::PostFx2dService>(runtime)
                {
                    let has_post_fx = render_packet
                        .post_fx_stack()
                        .is_some_and(|stack| !stack.is_empty());
                    let renderer_mode = if has_post_fx {
                        "frame_graph_postfx"
                    } else {
                        "frame_graph"
                    };
                    post_fx_service.set_renderer_mode(renderer_mode);
                }
                let extracted_render_layer_commands = extracted_render_layers.commands();
                let extracted_light_route_commands = extracted_light_routes.commands();
                let render_request = amigo_render_wgpu::WgpuFrameRenderRequest {
                    target: amigo_render_wgpu::WgpuFrameRenderTarget::Surface(surface),
                    scene: scene.as_ref(),
                    assets: assets.as_ref(),
                    world_2d: amigo_render_wgpu::WgpuWorld2dRenderInput {
                        tilemaps: &extracted_tilemaps,
                        sprites: &extracted_sprites,
                        layered_images: &extracted_layered_images,
                        global_lights: &extracted_global_lights,
                        lightmaps: &extracted_lightmaps,
                        text2d: &extracted_text2d,
                        vectors: &extracted_vectors,
                        render_layers: extracted_render_layer_commands.as_slice(),
                        light_routes: extracted_light_route_commands.as_slice(),
                        light_groups: render_packet.world_2d_light_groups(),
                        particles: render_packet.world_2d_particles(),
                    },
                    world_3d: amigo_render_wgpu::WgpuWorld3dRenderInput {
                        meshes: render_packet.world_3d_meshes(),
                        materials: render_packet.world_3d_materials(),
                        text3d: Some(render_packet.world_3d_text()),
                    },
                    game_ui: render_packet.game_ui_overlay(),
                    debug_ui: render_packet.debug_overlay(),
                    post_fx_stack: render_packet.post_fx_stack(),
                    composition_plan: &composition_plan,
                    frame_graph: &frame_graph,
                };
                renderer.render_frame_request(render_request)?;
            } else {
                surface.render_default_frame()?;
            }
        }

        Ok(HostControl::Continue)
    }
}





