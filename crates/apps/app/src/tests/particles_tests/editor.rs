use super::*;

#[test]
fn particles_editor_applies_registry_preset() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.preset",
        vec!["smoke".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("preset event should dispatch");

    let presets = runtime
        .resolve::<amigo_2d_particles::ParticlePreset2dService>()
        .expect("preset service should exist");
    let smoke = presets.preset("smoke").expect("smoke preset should exist");
    let particles = runtime
        .resolve::<amigo_2d_particles::Particle2dSceneService>()
        .expect("particle scene service should exist");
    let emitter = particles
        .emitter("playground-2d-particles-editor-preview-emitter")
        .expect("editor preview emitter should exist");
    assert_eq!(emitter.emitter.spawn_rate, smoke.emitter.spawn_rate);
    assert_eq!(emitter.emitter.spawn_area, smoke.emitter.spawn_area);

    let state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(
        state.get_string("selected_preset").as_deref(),
        Some("smoke")
    );
}

#[test]
fn particles_editor_color_ramp_preset_updates_emitter_ramp() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.ramp-preset",
        vec!["Fire".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("ramp preset event should dispatch");

    let particles = runtime
        .resolve::<amigo_2d_particles::Particle2dSceneService>()
        .expect("particle scene service should exist");
    let emitter = particles
        .emitter("playground-2d-particles-editor-preview-emitter")
        .expect("editor preview emitter should exist");
    let ramp = emitter
        .emitter
        .color_ramp
        .expect("ramp preset should set color_ramp");
    assert_eq!(ramp.stops.len(), 4);
    assert!(ramp.stops[1].color.r > 0.9);
    assert!(ramp.stops[1].color.g > 0.75);
}

#[test]
fn particles_editor_dropdown_can_select_deep_color_options() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.tab",
        vec!["Color".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("color tab event should dispatch");

    runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1440.0, 900.0)));

    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let ui_state = runtime
        .resolve::<UiStateService>()
        .expect("ui state should exist");
    let ui_theme = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme should exist");
    let ui_input = runtime
        .resolve::<UiInputService>()
        .expect("ui input should exist");

    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let editor = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-editor-ui")
        .expect("editor ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &editor.overlay);
    let dropdown = find_layout_node_by_path_suffix(&layout, ".color-dropdown")
        .expect("color dropdown should be in layout");

    ui_input.set_mouse_position(
        dropdown.rect.x + dropdown.rect.width * 0.5,
        dropdown.rect.y + dropdown.rect.height * 0.5,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("dropdown press should be processed");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime).expect("dropdown release should expand");
    ui_input.clear_frame_transients();

    let expanded = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let expanded_editor = expanded
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-editor-ui")
        .expect("expanded editor ui should resolve");
    let expanded_layout =
        build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &expanded_editor.overlay);
    let expanded_dropdown = find_layout_node_by_path_suffix(&expanded_layout, ".color-dropdown")
        .expect("expanded color dropdown should be in layout");

    ui_input.set_mouse_position(
        expanded_dropdown.rect.x + expanded_dropdown.rect.width * 0.5,
        expanded_dropdown.rect.y + 38.0 * 4.5,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("deep dropdown option press should be processed");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("deep dropdown option release should select");
    ui_input.clear_frame_transients();
    process_placeholder_bridges(&runtime).expect("dropdown event should dispatch");

    let state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(state.get_string("color").as_deref(), Some("Purple"));
}

#[test]
fn particles_editor_export_logs_preset_yaml() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.print-yaml",
        Vec::new(),
    ));
    process_placeholder_bridges(&runtime).expect("print yaml event should dispatch");

    let output = runtime
        .resolve::<DevConsoleState>()
        .expect("dev console should exist")
        .output_lines()
        .join("\n");
    assert!(output.contains("kind: particle-preset-2d"));
    assert!(output.contains("id: plasma-edited"));
    assert!(output.contains("label: Plasma Edited"));
    assert!(output.contains("category: energy"));
    assert!(output.contains("tags: [continuous, directional, energy, editor, edited]"));
    assert!(output.contains("emitter:"));
    assert!(output.contains("  type: ParticleEmitter2D"));
    assert!(output.contains("  max_particles: 160"));
    assert!(output.contains("  color_ramp:"));
    assert!(output.contains("  spawn_area:"));
    assert!(output.contains("  forces:"));
    assert!(output.contains("--- particle preset export end ---"));

    let export_path = PathBuf::from("target")
        .join("amigo-dev-exports")
        .join("particle-presets")
        .join("plasma-edited.yml");
    let exported = fs::read_to_string(&export_path).unwrap_or_else(|error| {
        panic!(
            "export `{}` should be readable: {error}",
            export_path.display()
        )
    });
    assert!(exported.contains("id: plasma-edited"));
    assert!(exported.contains("label: Plasma Edited"));
}

#[test]
fn particles_editor_mutates_emitter_from_script_event() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    assert_eq!(summary.active_scene.as_deref(), Some("editor"));
    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.spawn-rate",
        vec!["0.7500".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("spawn-rate event should dispatch");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.shape-kind",
        vec!["line".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("shape-kind event should dispatch");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.shape-mode",
        vec!["random_mix".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("shape-mode event should dispatch");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.align-kind",
        vec!["emitter".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("align-kind event should dispatch");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.blend-kind",
        vec!["additive".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("blend-kind event should dispatch");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.z-index",
        vec!["0.7000".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("z-index event should dispatch");

    let particles = runtime
        .resolve::<amigo_2d_particles::Particle2dSceneService>()
        .expect("particle scene service should exist");
    let emitter = particles
        .emitter("playground-2d-particles-editor-preview-emitter")
        .expect("editor preview emitter should exist");
    assert!(
        (emitter.emitter.spawn_rate - 150.0).abs() < 0.01,
        "expected spawn_rate to mutate to 150, got {}",
        emitter.emitter.spawn_rate
    );
    assert_eq!(emitter.emitter.shape_choices.len(), 3);
    assert_eq!(
        emitter.emitter.align,
        amigo_2d_particles::ParticleAlignMode2d::Emitter
    );
    assert_eq!(
        emitter.emitter.blend_mode,
        amigo_2d_particles::ParticleBlendMode2d::Additive
    );
    assert!((emitter.emitter.z_index - 50.0).abs() < 0.01);
}

#[test]
fn particles_editor_rgb_color_picker_updates_emitter_color() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.tab",
        vec!["Color".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("color tab event should dispatch");

    runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1440.0, 900.0)));

    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let ui_state = runtime
        .resolve::<UiStateService>()
        .expect("ui state should exist");
    let ui_theme = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme should exist");
    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let editor = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-editor-ui")
        .expect("editor ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &editor.overlay);
    let picker = find_layout_node_by_path_suffix(&layout, ".rgb-picker")
        .expect("rgb picker should be visible");

    let slider_start_x = picker.rect.x + 8.0 + 54.0 + 10.0 + 24.0;
    let slider_width = picker.rect.x + picker.rect.width - 8.0 - slider_start_x;
    let ui_input = runtime
        .resolve::<UiInputService>()
        .expect("ui input service should exist");
    ui_input.set_mouse_position(
        slider_start_x + slider_width * 0.82,
        picker.rect.y + 8.0 + 11.0,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("rgb picker input should be processed");
    process_placeholder_bridges(&runtime).expect("rgb picker event should dispatch");

    let particles = runtime
        .resolve::<amigo_2d_particles::Particle2dSceneService>()
        .expect("particle scene service should exist");
    let emitter = particles
        .emitter("playground-2d-particles-editor-preview-emitter")
        .expect("editor preview emitter should exist");
    assert!(
        (emitter.emitter.color.r - 0.82).abs() < 0.02,
        "expected red channel to update from rgb picker, got {:?}",
        emitter.emitter.color
    );
}

#[test]
fn particles_editor_tabs_switch_panels() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("editor")
            .with_dev_mode(true),
    )
    .expect("particles editor should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.editor.tab",
        vec!["Spawn".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("tab event should dispatch");

    let state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(state.get_string("selected_tab").as_deref(), Some("Spawn"));
}
