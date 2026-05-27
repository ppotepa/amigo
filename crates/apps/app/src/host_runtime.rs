use super::*;
use amigo_input_api::InputModifiers;
use amigo_runtime::SystemPhase;
use amigo_runtime_bundles::{AudioOutputBackendService, AudioOutputStartStatus};
use amigo_runtime_bundles::{
    add_ui_input_mouse_wheel, clear_ui_input_frame_transients, set_ui_input_left_button,
    set_ui_input_mouse_position,
};
use amigo_session::RuntimeSession;

const HOT_RELOAD_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

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
    editor_mode: bool,
    surface: Option<WgpuSurfaceState>,
    renderer: Option<WgpuSceneRenderer>,
    game_frame_cache: Option<CachedGameFrame>,
    scene_ids: Vec<String>,
    printed_console_lines: usize,
    last_hot_reload_poll: Option<std::time::Instant>,
    printed: bool,
    modifiers: InputModifiers,
}

struct CachedGameFrame {
    target: amigo_render_wgpu::WgpuOffscreenTarget,
    width: u32,
    height: u32,
    frame_index: u64,
}

impl InteractiveRuntimeHostHandler {
    #[cfg(test)]
    pub(crate) fn new(session: RuntimeSession, summary: BootstrapSummary) -> AmigoResult<Self> {
        Self::new_with_editor_mode(session, summary, false)
    }

    pub(crate) fn new_with_editor_mode(
        session: RuntimeSession,
        summary: BootstrapSummary,
        editor_mode: bool,
    ) -> AmigoResult<Self> {
        let runtime = session.runtime();
        let launch_selection = required::<LaunchSelection>(runtime)?;
        let mod_catalog = required::<ModCatalog>(runtime)?;
        let scene_ids =
            super::scene_ids_for_launch_selection(mod_catalog.as_ref(), launch_selection.as_ref());

        Ok(Self {
            session,
            printed_console_lines: summary.console_output.len(),
            summary,
            editor_mode,
            surface: None,
            renderer: None,
            game_frame_cache: None,
            scene_ids,
            printed: false,
            last_hot_reload_poll: None,
            modifiers: InputModifiers::default(),
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
        self.session.run_phase(SystemPhase::PreUpdate)
    }

    fn tick_runtime_update(&self) -> AmigoResult<()> {
        self.session.run_phase(SystemPhase::Update)
    }

    fn tick_runtime_post_update(&self) -> AmigoResult<()> {
        self.session.run_phase(SystemPhase::PostUpdate)
    }

    fn tick_host_frame(&mut self, now: std::time::Instant) -> AmigoResult<()> {
        let clock = required::<amigo_session::RuntimeFrameClockService>(self.runtime())?;
        clock.begin_host_frame(now);

        self.tick_runtime_pre_update()?;
        let (simulation_tick_count, _simulation_dt) = clock.take_pending_simulation_tick_count();
        for _ in 0..simulation_tick_count {
            self.tick_runtime_update()?;
        }
        self.pump_runtime()?;
        self.tick_runtime_post_update()?;
        self.poll_hot_reload(now)?;

        Ok(())
    }

    fn clear_host_frame_transients(&self) {
        if let Some(input_state) = self.runtime().resolve::<InputState>() {
            input_state.clear_frame_transients();
        }
        clear_ui_input_frame_transients(self.runtime());
    }

    fn render_or_present_frame(&mut self) -> AmigoResult<()> {
        let clock = required::<amigo_session::RuntimeFrameClockService>(self.runtime())?;
        let config = clock.config();

        let Some(surface_size) = self.surface.as_ref().map(|surface| surface.size()) else {
            return Ok(());
        };

        if self.renderer.is_none() {
            if let Some(surface) = &mut self.surface {
                surface.render_default_frame()?;
            }
            return Ok(());
        }

        if !config.presentation.cache_game_frame {
            if let (Some(surface), Some(renderer)) = (&mut self.surface, &mut self.renderer) {
                crate::render_runtime::build_render_frame_for_session(
                    &self.session,
                    surface,
                    renderer,
                )?;
                clock.mark_game_frame_rendered();
            }

            if let Ok(debug_overlay_service) =
                required::<crate::debug_overlay::DebugOverlayService>(self.runtime())
            {
                if debug_overlay_service.is_enabled() {
                    debug_overlay_service.record_frame_clock_snapshot(clock.snapshot());
                }
            }

            self.session.complete_render_present();
            return Ok(());
        }

        let cache_invalid = self.game_frame_cache.as_ref().is_none_or(|cache| {
            cache.width != surface_size.width || cache.height != surface_size.height
        });

        if cache_invalid {
            if let Some(surface) = &self.surface {
                self.game_frame_cache = Some(CachedGameFrame {
                    target: surface.create_compatible_offscreen_target(
                        surface_size.width,
                        surface_size.height,
                        "amigo-cached-game-frame",
                    ),
                    width: surface_size.width,
                    height: surface_size.height,
                    frame_index: 0,
                });
                clock.mark_game_frame_cache_invalid();
            }
        }

        let should_render_game =
            !config.presentation.cache_game_frame || clock.should_render_game_frame();

        if should_render_game {
            if let (Some(cache), Some(renderer)) = (&mut self.game_frame_cache, &mut self.renderer)
            {
                crate::render_runtime::render_game_frame_to_cache(
                    &self.session,
                    &mut cache.target,
                    renderer,
                    matches!(
                        config.presentation.game_ui,
                        amigo_session::ResolvedPresentationLayerMode::Cached
                    ),
                )?;
                cache.frame_index += 1;
                clock.mark_game_frame_rendered();
            }
        }

        if let Ok(debug_overlay_service) =
            required::<crate::debug_overlay::DebugOverlayService>(self.runtime())
        {
            if debug_overlay_service.is_enabled() {
                debug_overlay_service.record_frame_clock_snapshot(clock.snapshot());
            }
        }

        let overlay_packet = if self.needs_live_host_overlay(&config) {
            crate::render_runtime::extract_live_host_overlay_packet(&self.session)?
        } else {
            Default::default()
        };
        let emergency_overlay = crate::render_runtime::emergency_overlay_lines(self.runtime());
        let editor_game_viewport = crate::render_runtime::editor_game_viewport_placement(
            self.runtime(),
            surface_size.width,
            surface_size.height,
        );
        let assets = required::<AssetCatalog>(self.runtime())?;

        if let (Some(cache), Some(surface), Some(renderer)) = (
            &self.game_frame_cache,
            &mut self.surface,
            &mut self.renderer,
        ) {
            renderer.present_cached_frame_to_surface(
                surface,
                &cache.target,
                assets.as_ref(),
                overlay_packet.debug_overlay(),
                editor_game_viewport,
                emergency_overlay.as_slice(),
            )?;
            clock.mark_host_presented_cached_frame();
        }

        self.session.complete_render_present();
        Ok(())
    }

    fn needs_live_host_overlay(&self, config: &amigo_session::ResolvedFrameClockConfig) -> bool {
        config.presentation.devtools_live
            || config.presentation.editor_live
            || config.presentation.debug_overlay_live
    }

    fn host_scene_switch_enabled(&self) -> bool {
        self.summary.startup_mod.as_deref() == Some("core-game") && self.scene_ids.len() > 1
    }

    fn handle_dev_console_input(&mut self, event: &InputEvent) -> AmigoResult<bool> {
        let mut modifiers = self.modifiers;
        let outcome = amigo_devtools::DevConsoleInputController::handle_event(
            self.runtime(),
            event,
            &mut modifiers,
        )?;
        self.modifiers = modifiers;
        Ok(outcome.is_consumed())
    }

    fn pump_runtime(&mut self) -> AmigoResult<()> {
        let previous_scene = self.summary.active_scene.clone();
        let previous_document = self.summary.loaded_scene_document.clone();
        let previous_entities = self.summary.scene_entities.clone();
        let bridge_summary =
            crate::orchestration::stabilize_runtime_queues_for_session(&self.session)?;
        let scene_service = required::<SceneService>(self.runtime())?;
        let active_scene = scene_service
            .selected_scene()
            .map(|scene| scene.as_str().to_owned());
        let loaded_scene_document =
            crate::scene_runtime::current_loaded_scene_document_summary(self.runtime())?;
        let scene_entities = scene_service.entity_names();

        if active_scene != previous_scene {
            println!(
                "active scene: {}",
                active_scene.as_deref().unwrap_or("none")
            );
        }

        if loaded_scene_document != previous_document {
            println!(
                "scene document: {}",
                loaded_scene_document
                    .as_ref()
                    .map(|document| format!(
                        "{}:{}",
                        document.source_mod,
                        document.relative_path.display()
                    ))
                    .unwrap_or_else(|| "none".to_owned())
            );
        }

        if scene_entities != previous_entities {
            println!(
                "scene entities: {}",
                crate::app_helpers::display_string_list(&scene_entities)
            );
        }

        for line in bridge_summary
            .console_output
            .iter()
            .skip(self.printed_console_lines)
        {
            println!("console: {line}");
        }

        self.printed_console_lines = bridge_summary.console_output.len();
        self.summary.active_scene = active_scene;
        self.summary.loaded_scene_document = loaded_scene_document;
        self.summary.scene_entities = scene_entities;
        self.summary.processed_script_commands = bridge_summary.processed_script_commands;
        self.summary.processed_audio_commands = bridge_summary.processed_audio_commands;
        self.summary.processed_scene_commands = bridge_summary.processed_scene_commands;
        self.summary.processed_script_events = bridge_summary.processed_script_events;
        self.summary.console_commands = bridge_summary.console_commands;
        self.summary.console_output = bridge_summary.console_output;

        Ok(())
    }

    fn poll_hot_reload(&mut self, now: std::time::Instant) -> AmigoResult<()> {
        if !self.summary.dev_mode {
            return Ok(());
        }

        if self
            .last_hot_reload_poll
            .is_some_and(|last| now.duration_since(last) < HOT_RELOAD_POLL_INTERVAL)
        {
            return Ok(());
        }

        self.last_hot_reload_poll = Some(now);
        if crate::orchestration::poll_runtime_hot_reload(self.runtime())? > 0 {
            self.pump_runtime()?;
        }
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
            max_frame_rate_fps: self.summary.frame_cap_fps,
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
                maximized: self.editor_mode,
                ..WindowDescriptor::default()
            },
            exit_strategy: HostExitStrategy::Manual,
            max_frame_rate_fps: self.summary.frame_cap_fps,
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

        Ok(HostControl::Continue)
    }

    fn on_input_event(&mut self, event: InputEvent) -> AmigoResult<HostControl> {
        if self.handle_dev_console_input(&event)? {
            return Ok(HostControl::Continue);
        }

        if amigo_editor_ingame::handle_editor_input(self.runtime(), &event, self.modifiers)? {
            return Ok(HostControl::Continue);
        }

        match event {
            InputEvent::CursorMoved { x, y } => {
                set_ui_input_mouse_position(self.runtime(), x as f32, y as f32);
                if let Some(input_state) = self.runtime().resolve::<InputState>() {
                    input_state.set_cursor_position(x as f32, y as f32);
                }
            }
            InputEvent::MouseButton {
                button: amigo_input_api::MouseButton::Left,
                pressed,
            } => {
                set_ui_input_left_button(self.runtime(), pressed);
                if let Some(input_state) = self.runtime().resolve::<InputState>() {
                    input_state.set_mouse_button(amigo_input_api::MouseButton::Left, pressed);
                }
            }
            InputEvent::MouseButton { button, pressed } => {
                if let Some(input_state) = self.runtime().resolve::<InputState>() {
                    input_state.set_mouse_button(button, pressed);
                }
            }
            InputEvent::MouseWheel { delta_y } => {
                add_ui_input_mouse_wheel(self.runtime(), delta_y);
                if let Some(input_state) = self.runtime().resolve::<InputState>() {
                    input_state.add_mouse_wheel_delta(delta_y);
                }
            }
            InputEvent::ModifiersChanged(modifiers) => {
                if let Some(input_state) = self.runtime().resolve::<InputState>() {
                    input_state.set_modifiers(modifiers);
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
            amigo_runtime_bundles::update_ui_input_viewport_state(
                self.runtime(),
                size.width as f32,
                size.height as f32,
            );
            if let Some(input_state) = self.runtime().resolve::<InputState>() {
                input_state.set_viewport_size(size.width as f32, size.height as f32);
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
            amigo_runtime_bundles::update_ui_input_viewport_state(
                self.runtime(),
                size.width as f32,
                size.height as f32,
            );
            if let Some(input_state) = self.runtime().resolve::<InputState>() {
                input_state.set_viewport_size(size.width as f32, size.height as f32);
            }
        }
        start_audio_output(self.runtime())?;
        self.summary = refresh_runtime_summary(self.runtime())?;

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        self.tick_host_frame(std::time::Instant::now())?;
        if let Err(error) = self.render_or_present_frame() {
            self.session
                .mark_render_error(format!("render frame failed: {error}"));
            return Err(error);
        }

        self.clear_host_frame_transients();
        Ok(HostControl::Continue)
    }
}
