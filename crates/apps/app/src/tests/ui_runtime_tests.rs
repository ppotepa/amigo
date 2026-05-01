use super::*;

#[test]
fn playground_hud_ui_click_switches_theme() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
            .with_startup_mod("playground-hud-ui")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("hud ui showcase bootstrap should succeed");

    runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1280.0, 720.0)));

    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let ui_state = runtime
        .resolve::<UiStateService>()
        .expect("ui state service should exist");
    let ui_theme = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme service should exist");
    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);
    fn find_path_ending<'a>(
        node: &'a OverlayUiLayoutNode,
        suffix: &str,
    ) -> Option<&'a OverlayUiLayoutNode> {
        if node.path.ends_with(suffix) {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = find_path_ending(child, suffix) {
                return Some(found);
            }
        }
        None
    }

    let button = find_path_ending(&layout, ".theme-clean-dev")
        .expect("clean theme button should be in layout");
    let button_path = button.path.clone();
    assert!(
        showcase.click_bindings.contains_key(&button_path),
        "button path should have click binding: {button_path}; known={:?}",
        showcase.click_bindings.keys().collect::<Vec<_>>()
    );
    let click_x = button.rect.x + button.rect.width * 0.5;
    let click_y = button.rect.y + button.rect.height * 0.5;

    let ui_input = runtime
        .resolve::<UiInputService>()
        .expect("ui input service should exist");
    ui_input.set_mouse_position(click_x, click_y);
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime).expect("ui press should be processed");
    assert!(
        ui_state.background_override(&button_path).is_some(),
        "pressing a button should apply a transient pressed background"
    );

    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime).expect("ui release should be processed");
    let bridge = process_placeholder_bridges(&runtime).expect("ui click event should dispatch");
    assert!(
        bridge
            .processed_script_events
            .iter()
            .any(|event| event.contains("playground-hud-ui.theme.clean-dev")),
        "ui click should publish the clean theme script event: {:?}",
        bridge.processed_script_events
    );

    assert_eq!(ui_theme.active_theme_id().as_deref(), Some("clean_dev"));
    assert!(
        ui_state.background_override(&button_path).is_none(),
        "pressed background should clear after release"
    );
}

#[test]
fn playground_hud_ui_dropdown_changes_swatch_color() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
            .with_startup_mod("playground-hud-ui")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("hud ui showcase should bootstrap");

    runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1280.0, 720.0)));

    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let ui_state = runtime
        .resolve::<UiStateService>()
        .expect("ui state service should exist");
    let ui_theme = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme service should exist");
    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

    fn find_path_ending<'a>(
        node: &'a OverlayUiLayoutNode,
        suffix: &str,
    ) -> Option<&'a OverlayUiLayoutNode> {
        if node.path.ends_with(suffix) {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = find_path_ending(child, suffix) {
                return Some(found);
            }
        }
        None
    }

    let dropdown =
        find_path_ending(&layout, ".color-dropdown").expect("color dropdown should be in layout");
    let ui_input = runtime
        .resolve::<UiInputService>()
        .expect("ui input service should exist");
    ui_input.set_mouse_position(
        dropdown.rect.x + dropdown.rect.width * 0.5,
        dropdown.rect.y + dropdown.rect.height * 0.5,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("dropdown press should be processed");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime).expect("dropdown should open on click");
    ui_input.clear_frame_transients();
    assert_eq!(
        ui_state.expanded_override(&dropdown.path),
        Some(true),
        "first click should expand dropdown"
    );

    let expanded = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let expanded_showcase = expanded
        .iter()
        .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
        .expect("showcase ui should resolve");
    let expanded_layout = build_ui_layout_tree(
        UiViewportSize::new(1280.0, 720.0),
        &expanded_showcase.overlay,
    );
    let expanded_dropdown = find_path_ending(&expanded_layout, ".color-dropdown")
        .expect("expanded dropdown should be in layout");
    ui_input.set_mouse_position(
        expanded_dropdown.rect.x + expanded_dropdown.rect.width * 0.5,
        expanded_dropdown.rect.y + 38.0 * 2.5,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("dropdown option press should be processed");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime).expect("dropdown option should select");
    ui_input.clear_frame_transients();
    process_placeholder_bridges(&runtime).expect("dropdown event should dispatch");

    let scene_state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(
        scene_state.get_string("color_preset").as_deref(),
        Some("Orange")
    );
    assert!(
        ui_state
            .background_override(
                "playground-hud-ui-showcase.root.main.editor.dropdown-row.color-swatch"
            )
            .is_some(),
        "dropdown selection should update color swatch background"
    );
}

#[test]
fn playground_hud_ui_f_keys_and_option_set_work() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
            .with_startup_mod("playground-hud-ui")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("hud ui showcase host should bootstrap");
    let mut host = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host should build");

    host.on_input_event(InputEvent::Key {
        key: KeyCode::F2,
        pressed: true,
    })
    .expect("F2 should be accepted");
    host.on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime should tick");

    let themes = host
        .runtime
        .resolve::<UiThemeService>()
        .expect("ui theme service should exist");
    assert_eq!(themes.active_theme_id().as_deref(), Some("clean_dev"));

    host.runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1280.0, 720.0)));

    let ui_scene = host
        .runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let ui_state = host
        .runtime
        .resolve::<UiStateService>()
        .expect("ui state service should exist");
    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        themes.as_ref(),
    );
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

    fn find_path_ending<'a>(
        node: &'a OverlayUiLayoutNode,
        suffix: &str,
    ) -> Option<&'a OverlayUiLayoutNode> {
        if node.path.ends_with(suffix) {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = find_path_ending(child, suffix) {
                return Some(found);
            }
        }
        None
    }

    let option_set =
        find_path_ending(&layout, ".color-options").expect("color option set should be in layout");
    let click_x = option_set.rect.x + option_set.rect.width * 0.5;
    let click_y = option_set.rect.y + option_set.rect.height * 0.5;
    let ui_input = host
        .runtime
        .resolve::<UiInputService>()
        .expect("ui input service should exist");
    ui_input.set_mouse_position(click_x, click_y);
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&host.runtime)
        .expect("option set press should be processed");
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&host.runtime)
        .expect("option set release should be processed");
    process_placeholder_bridges(&host.runtime).expect("option set event should dispatch");

    let scene_state = host
        .runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(
        scene_state.get_string("color_preset").as_deref(),
        Some("Orange")
    );
}

#[test]
fn playground_hud_ui_showcase_bootstraps() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
            .with_startup_mod("playground-hud-ui")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("hud ui showcase bootstrap should succeed");

    let themes = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme service should exist");

    assert_eq!(summary.active_scene.as_deref(), Some("showcase"));
    assert_eq!(themes.active_theme_id().as_deref(), Some("space_dark"));
    assert!(summary.failed_assets.is_empty());
    assert!(
        summary
            .ui_entities
            .iter()
            .any(|entity| entity == "playground-hud-ui-showcase")
    );
}

#[test]
fn playground_hud_ui_slider_drag_updates_without_crashing() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
            .with_startup_mod("playground-hud-ui")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("hud ui showcase bootstrap should succeed");

    runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1280.0, 720.0)));

    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let ui_state = runtime
        .resolve::<UiStateService>()
        .expect("ui state service should exist");
    let ui_theme = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme service should exist");
    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

    fn find_path_ending<'a>(
        node: &'a OverlayUiLayoutNode,
        suffix: &str,
    ) -> Option<&'a OverlayUiLayoutNode> {
        if node.path.ends_with(suffix) {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = find_path_ending(child, suffix) {
                return Some(found);
            }
        }
        None
    }

    let slider =
        find_path_ending(&layout, ".volume-slider").expect("volume slider should be in layout");
    let drag_x = slider.rect.x + slider.rect.width * 0.25;
    let drag_y = slider.rect.y + slider.rect.height * 0.5;

    let ui_input = runtime
        .resolve::<UiInputService>()
        .expect("ui input service should exist");
    ui_input.set_mouse_position(drag_x, drag_y);
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime).expect("ui slider drag should process");
    process_placeholder_bridges(&runtime).expect("slider change event should dispatch");

    let scene_state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert!(
        (scene_state.get_float("volume").unwrap_or_default() - 0.25).abs() < 0.02,
        "slider drag should update Rhai scene volume"
    );
}

#[test]
fn playground_hud_ui_tabs_change_editor_panel() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
            .with_startup_mod("playground-hud-ui")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("hud ui showcase should bootstrap");

    runtime
        .resolve::<crate::systems::UiInputViewportState>()
        .expect("ui viewport should exist")
        .set(Some(UiViewportSize::new(1280.0, 720.0)));

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
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

    fn find_path_ending<'a>(
        node: &'a OverlayUiLayoutNode,
        suffix: &str,
    ) -> Option<&'a OverlayUiLayoutNode> {
        if node.path.ends_with(suffix) {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = find_path_ending(child, suffix) {
                return Some(found);
            }
        }
        None
    }

    let tabs =
        find_path_ending(&layout, ".tab-options").expect("tab option set should be in layout");
    let click_x = tabs.rect.x + tabs.rect.width * (5.5 / 7.0);
    let click_y = tabs.rect.y + tabs.rect.height * 0.5;
    let ui_input = runtime
        .resolve::<UiInputService>()
        .expect("ui input service should exist");
    ui_input.set_mouse_position(click_x, click_y);
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("tab option press should be processed");
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("tab option release should be processed");
    process_placeholder_bridges(&runtime).expect("tab change event should dispatch");

    let scene_state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(
        scene_state.get_string("selected_tab").as_deref(),
        Some("Forces")
    );
}
