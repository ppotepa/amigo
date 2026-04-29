use super::*;

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

    fn tick_active_scripts(&mut self, delta_seconds: f32) -> AmigoResult<()> {
        let script_runtime = required::<ScriptRuntimeService>(&self.runtime)?;
        for script in crate::scripting_runtime::current_executed_scripts(&self.runtime)? {
            match script.role {
                ScriptExecutionRole::ModPersistent | ScriptExecutionRole::Scene => {
                    script_runtime.call_update(&script.source_name, delta_seconds)?;
                }
                ScriptExecutionRole::ModBootstrap => {}
            }
        }

        Ok(())
    }

    fn tick_scene_transitions(&mut self, delta_seconds: f32) -> AmigoResult<()> {
        let scene_transition_service = required::<SceneTransitionService>(&self.runtime)?;
        let scene_command_queue = required::<SceneCommandQueue>(&self.runtime)?;

        for command in scene_transition_service.tick(delta_seconds) {
            scene_command_queue.submit(command);
        }

        Ok(())
    }

    fn host_scene_switch_enabled(&self) -> bool {
        self.summary.startup_mod.as_deref() == Some("core-game") && self.scene_ids.len() > 1
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

        for line in updated.console_output.iter().skip(self.printed_console_lines) {
            println!("console: {line}");
        }

        self.printed_console_lines = updated.console_output.len();
        self.summary = updated;

        Ok(())
    }

    fn process_ui_input(&mut self) -> AmigoResult<()> {
        let Some(surface) = self.surface.as_ref() else {
            return Ok(());
        };

        let ui_input = required::<UiInputService>(&self.runtime)?;
        let snapshot = ui_input.snapshot();
        if !snapshot.mouse_left_released {
            return Ok(());
        }

        let Some(mouse_position) = snapshot.mouse_position else {
            return Ok(());
        };

        let ui_scene = required::<UiSceneService>(&self.runtime)?;
        let ui_state = required::<UiStateService>(&self.runtime)?;
        let script_event_queue = required::<ScriptEventQueue>(&self.runtime)?;
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
        );
        let size = surface.size();
        let viewport = UiViewportSize::new(size.width as f32, size.height as f32);

        for document in resolved.iter().rev() {
            let layout = build_ui_layout_tree(viewport, &document.overlay);
            let Some(path) =
                crate::ui_runtime::hit_test_ui_layout(&layout, mouse_position.x, mouse_position.y)
            else {
                continue;
            };

            if !ui_state.is_enabled(&path) {
                continue;
            }

            if let Some(binding) = document.click_bindings.get(&path) {
                script_event_queue.publish(ScriptEvent::new(
                    binding.event.clone(),
                    binding.payload.clone(),
                ));
                break;
            }
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
            self.process_ui_input()?;
            self.tick_active_scripts(1.0 / 60.0)?;
            systems::motion_2d::tick_motion_2d_world(&self.runtime, 1.0 / 60.0)?;
            systems::camera_follow_2d::tick_camera_follow_world(&self.runtime)?;
            systems::parallax_2d::tick_parallax_world(&self.runtime)?;
            self.tick_scene_transitions(1.0 / 60.0)?;
            self.pump_runtime()?;
            systems::audio::tick_audio_runtime(&self.runtime, 1.0 / 60.0)?;
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
                let text2d = required::<Text2dSceneService>(&self.runtime)?;
                let meshes = required::<MeshSceneService>(&self.runtime)?;
                let text3d = required::<Text3dSceneService>(&self.runtime)?;
                let materials = required::<MaterialSceneService>(&self.runtime)?;
                let ui_scene = required::<UiSceneService>(&self.runtime)?;
                let ui_state = required::<UiStateService>(&self.runtime)?;
                let ui_documents = crate::ui_runtime::resolve_ui_overlay_documents(
                    ui_scene.as_ref(),
                    ui_state.as_ref(),
                );
                let ui_overlays = ui_documents
                    .iter()
                    .map(|document| document.overlay.clone())
                    .collect::<Vec<_>>();

                renderer.render_scene_with_ui_documents(
                    surface,
                    scene.as_ref(),
                    assets.as_ref(),
                    tilemaps.as_ref(),
                    sprites.as_ref(),
                    text2d.as_ref(),
                    meshes.as_ref(),
                    materials.as_ref(),
                    Some(text3d.as_ref()),
                    &ui_overlays,
                )?;
            } else {
                surface.render_default_frame()?;
            }
        }

        Ok(HostControl::Continue)
    }
}
