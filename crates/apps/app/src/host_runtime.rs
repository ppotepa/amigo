use super::*;
use amigo_input_api::InputModifiers;
use amigo_runtime::SystemPhase;
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
    editor_mode: bool,
    surface: Option<WgpuSurfaceState>,
    renderer: Option<WgpuSceneRenderer>,
    scene_ids: Vec<String>,
    printed_console_lines: usize,
    printed: bool,
    modifiers: InputModifiers,
}

impl InteractiveRuntimeHostHandler {
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
            scene_ids,
            printed: false,
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

    fn host_scene_switch_enabled(&self) -> bool {
        self.summary.startup_mod.as_deref() == Some("core-game") && self.scene_ids.len() > 1
    }

    fn handle_dev_console_input(&mut self, event: &InputEvent) -> AmigoResult<bool> {
        let console = required::<DevConsoleState>(self.runtime())?;
        let completion = required::<amigo_devtools::ConsoleCompletionState>(self.runtime())?;
        let registry = required::<amigo_devtools::RuntimeConsoleCommandRegistry>(self.runtime())?;

        if let InputEvent::ModifiersChanged(modifiers) = event {
            self.modifiers = *modifiers;
            return Ok(console.is_open());
        }

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::Backquote,
                pressed: true,
            }
        ) {
            console.toggle_open();
            if console.is_open() {
                refresh_console_completion(
                    self.runtime(),
                    completion.as_ref(),
                    registry.as_ref(),
                    console.as_ref(),
                );
            } else {
                completion.clear();
            }
            return Ok(true);
        }

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::F1,
                pressed: true,
            }
        ) {
            console.set_open(true);
            refresh_console_completion(
                self.runtime(),
                completion.as_ref(),
                registry.as_ref(),
                console.as_ref(),
            );
            return Ok(true);
        }

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::F2,
                pressed: true,
            }
        ) {
            self.queue_console_command("reload")?;
            return Ok(true);
        }

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::R,
                pressed: true,
            }
        ) && (self.modifiers.control || self.modifiers.super_key)
        {
            self.queue_console_command("reload")?;
            return Ok(true);
        }

        if matches!(
            event,
            InputEvent::Key {
                key: KeyCode::D,
                pressed: true,
            }
        ) && (self.modifiers.control || self.modifiers.super_key)
        {
            self.queue_console_command("diagnostics")?;
            return Ok(true);
        }

        if !console.is_open() {
            return Ok(false);
        }

        match event {
            InputEvent::TextInput { text } => {
                if !self.modifiers.control && !self.modifiers.super_key {
                    console.insert_input_text(text);
                    refresh_after_console_edit(
                        self.runtime(),
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
                }
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
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Delete,
                pressed: true,
            } => {
                console.delete_input();
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Left,
                pressed: true,
            } => {
                console.move_input_left(self.modifiers.shift, self.modifiers.control);
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            } => {
                console.move_input_right(self.modifiers.shift, self.modifiers.control);
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Home,
                pressed: true,
            } => {
                console.move_input_home(self.modifiers.shift);
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::End,
                pressed: true,
            } => {
                console.move_input_end(self.modifiers.shift);
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::A,
                pressed: true,
            } if self.modifiers.control => {
                console.select_all_input();
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::C,
                pressed: true,
            } if self.modifiers.control => {
                console.copy_input_selection();
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::X,
                pressed: true,
            } if self.modifiers.control => {
                console.cut_input_selection();
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::V,
                pressed: true,
            } if self.modifiers.control => {
                console.paste_input_clipboard();
                refresh_after_console_edit(
                    self.runtime(),
                    console.as_ref(),
                    completion.as_ref(),
                    registry.as_ref(),
                );
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Tab,
                pressed: true,
            } => {
                refresh_console_completion(
                    self.runtime(),
                    completion.as_ref(),
                    registry.as_ref(),
                    console.as_ref(),
                );
                let snapshot = console.input_snapshot();
                if let Some(edit) = completion.accept_tab(&snapshot.text, snapshot.cursor) {
                    console.set_input_with_cursor(edit.input, edit.cursor_index);
                    refresh_after_console_edit(
                        self.runtime(),
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
                }
                Ok(true)
            }
            InputEvent::Key {
                key: KeyCode::Enter,
                pressed: true,
            } => {
                let snapshot = console.input_snapshot();
                if completion.snapshot().is_some() {
                    if let Some(edit) = completion.accept_tab(&snapshot.text, snapshot.cursor) {
                        console.set_input_with_cursor(edit.input, edit.cursor_index);
                        refresh_after_console_edit(
                            self.runtime(),
                            console.as_ref(),
                            completion.as_ref(),
                            registry.as_ref(),
                        );
                        return Ok(true);
                    }
                }

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
                    refresh_after_console_edit(
                        self.runtime(),
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
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
                    refresh_after_console_edit(
                        self.runtime(),
                        console.as_ref(),
                        completion.as_ref(),
                        registry.as_ref(),
                    );
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

fn refresh_console_completion(
    runtime: &Runtime,
    completion: &amigo_devtools::ConsoleCompletionState,
    registry: &amigo_devtools::RuntimeConsoleCommandRegistry,
    console: &DevConsoleState,
) {
    let snapshot = console.input_snapshot();
    let descriptors = registry.descriptors();
    let context = console_completion_context(runtime, console);
    completion.refresh(&snapshot.text, snapshot.cursor, &descriptors, &context);
}

fn console_completion_context(
    runtime: &Runtime,
    console: &DevConsoleState,
) -> amigo_devtools::ConsoleCompletionContext {
    let entity_names = runtime
        .resolve::<SceneService>()
        .map(|scene| scene.entity_names())
        .unwrap_or_default();

    let postfx_indices = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>()
        .map(|postfx| {
            (0..postfx.scene_effect_count())
                .map(|index| index.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let rhai_symbols = console
        .command_history()
        .into_iter()
        .flat_map(|line| amigo_devtools::collect_console_rhai_symbols_from_source(&line))
        .collect();

    amigo_devtools::ConsoleCompletionContext {
        entity_names,
        postfx_kinds: vec![
            "blur".to_owned(),
            "crt".to_owned(),
            "dirty_bloom".to_owned(),
        ],
        postfx_indices,
        rhai_symbols,
    }
}

fn refresh_after_console_edit(
    runtime: &Runtime,
    console: &DevConsoleState,
    completion: &amigo_devtools::ConsoleCompletionState,
    registry: &amigo_devtools::RuntimeConsoleCommandRegistry,
) {
    refresh_console_completion(runtime, completion, registry, console);
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
                maximized: self.editor_mode,
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
            self.pump_runtime()?;
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

        if amigo_editor_ingame::handle_editor_input(self.runtime(), &event, self.modifiers)? {
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
            required::<amigo_runtime_bundles::amigo_ui::UiInputViewportState>(self.runtime())?.set(
                Some(UiViewportSize::new(size.width as f32, size.height as f32)),
            );
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
            required::<amigo_runtime_bundles::amigo_ui::UiInputViewportState>(self.runtime())?.set(
                Some(UiViewportSize::new(size.width as f32, size.height as f32)),
            );
        }
        start_audio_output(self.runtime())?;
        self.summary = refresh_runtime_summary(self.runtime())?;

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        self.tick_runtime_pre_update()?;
        self.tick_runtime_update()?;
        self.tick_runtime_post_update()?;
        self.pump_runtime()?;

        if let Some(surface) = &mut self.surface {
            if let Some(renderer) = &mut self.renderer {
                if let Err(error) = crate::render_runtime::build_render_frame_for_session(
                    &self.session,
                    surface,
                    renderer,
                ) {
                    self.session
                        .mark_render_error(format!("render frame failed: {error}"));
                    return Err(error);
                }
                self.session.complete_render_present();
            } else {
                surface.render_default_frame()?;
            }
        }

        Ok(HostControl::Continue)
    }
}
