use super::*;

#[test]
fn particles_playground_menu_bootstraps() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("menu")
            .with_dev_mode(true),
    )
    .expect("particles menu should bootstrap");

    assert_eq!(summary.active_scene.as_deref(), Some("menu"));
    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    assert!(
        ui_scene
            .entity_names()
            .contains(&"playground-2d-particles-menu-ui".to_owned())
    );
}

#[test]
fn particles_showcase_dropdown_can_wheel_scroll_to_lava_sparks() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("particles showcase should bootstrap");
    process_placeholder_bridges(&runtime).expect("showcase ui sync commands should dispatch");

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
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-showcase-ui")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &showcase.overlay);
    let dropdown =
        find_layout_node_by_path_suffix(&layout, ".preset-options").expect("preset dropdown should be in layout");
    let options = match &dropdown.node.kind {
        UiOverlayNodeKind::Dropdown { options, .. } => options.clone(),
        other => panic!("preset-options should resolve as dropdown, got {other:?}"),
    };
    let lava_index = options
        .iter()
        .position(|option| option == "lava_sparks")
        .expect("lava_sparks should be present in the dropdown registry");

    ui_input.set_mouse_position(
        dropdown.rect.x + dropdown.rect.width * 0.5,
        dropdown.rect.y + dropdown.rect.height * 0.5,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime).expect("dropdown press should process");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime).expect("dropdown release should expand");
    ui_input.clear_frame_transients();

    ui_input.set_mouse_position(
        dropdown.rect.x + dropdown.rect.width * 0.5,
        dropdown.rect.y + 38.0 * 4.5,
    );
    let target_offset = (lava_index as f32 - 4.0).max(0.0);
    ui_input.add_mouse_wheel(-(target_offset / 0.65));
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("dropdown wheel should smooth-scroll");
    ui_input.clear_frame_transients();
    let actual_offset = ui_state.dropdown_scroll_offset(&dropdown.path);
    assert!(
        actual_offset > 0.0,
        "wheel scrolling over an expanded dropdown should update its own scroll offset"
    );

    let lava_row = (lava_index as f32 - actual_offset + 1.5).clamp(1.25, 10.75);
    ui_input.set_mouse_position(
        dropdown.rect.x + dropdown.rect.width * 0.5,
        dropdown.rect.y + 38.0 * lava_row,
    );
    ui_input.set_left_button(true);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("lava_sparks option press should process");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    crate::systems::ui_input::process_ui_input(&runtime)
        .expect("lava_sparks option release should select");
    ui_input.clear_frame_transients();
    process_placeholder_bridges(&runtime).expect("dropdown event should dispatch");

    let state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert_eq!(
        state.get_string("selected_preset").as_deref(),
        Some("lava_sparks")
    );
}

#[test]
fn particles_showcase_explosion_burst_work() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("particles showcase should bootstrap");

    let events = runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist");
    events.publish(ScriptEvent::new(
        "playground-2d-particles.showcase.select",
        vec!["explosion".to_owned()],
    ));
    process_placeholder_bridges(&runtime).expect("select event should dispatch");
    crate::systems::particles_2d::tick_particles_2d_world(&runtime, 1.0 / 60.0)
        .expect("particle runtime tick should succeed");

    let particles = runtime
        .resolve::<amigo_2d_particles::Particle2dSceneService>()
        .expect("particle scene service should exist");
    assert!(
        particles.particle_count("playground-2d-particles-preview-emitter") > 0,
        "explosion preset should emit particles through preview burst"
    );
}

#[test]
fn particles_showcase_hydrates_emitters() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-particles".to_owned(),
            ])
            .with_startup_mod("playground-2d-particles")
            .with_startup_scene("showcase")
            .with_dev_mode(true),
    )
    .expect("particles showcase should bootstrap");

    assert_eq!(summary.active_scene.as_deref(), Some("showcase"));
    let particles = runtime
        .resolve::<amigo_2d_particles::Particle2dSceneService>()
        .expect("particle scene service should exist");
    let emitters = particles
        .emitters()
        .into_iter()
        .map(|command| command.entity_name)
        .collect::<Vec<_>>();
    assert_eq!(
        emitters,
        vec!["playground-2d-particles-preview-emitter".to_owned()],
        "showcase should hydrate only the preview emitter; preset data comes from registry"
    );
    let presets = runtime
        .resolve::<amigo_2d_particles::ParticlePreset2dService>()
        .expect("particle preset service should exist");
    let fire = presets.preset("fire").expect("fire preset should exist");
    assert!(
        fire.emitter.color_ramp.is_some(),
        "fire preset should hydrate a color ramp"
    );
    let preview = particles
        .emitter("playground-2d-particles-preview-emitter")
        .expect("preview emitter should exist");
    assert_eq!(preview.emitter.spawn_rate, fire.emitter.spawn_rate);
    assert_eq!(preview.emitter.shape, fire.emitter.shape);
    process_placeholder_bridges(&runtime).expect("showcase ui sync commands should dispatch");
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
    let showcase_ui = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-showcase-ui")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &showcase_ui.overlay);
    let dropdown = find_layout_node_by_path_suffix(&layout, ".preset-options").expect("preset dropdown should exist");
    match &dropdown.node.kind {
        UiOverlayNodeKind::Dropdown { options, .. } => {
            assert_eq!(
                options,
                &presets.ids(),
                "showcase dropdown should be hydrated from the preset registry"
            );
        }
        other => panic!("preset-options should resolve as dropdown, got {other:?}"),
    }

    crate::systems::particles_2d::tick_particles_2d_world(&runtime, 1.0 / 10.0)
        .expect("particle runtime tick should succeed");
    assert!(
        !particles.draw_commands().is_empty(),
        "showcase emitters should produce particle draw commands after a tick"
    );
    let scene_service = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let tilemap_scene_service = runtime
        .resolve::<TileMap2dSceneService>()
        .expect("tilemap service should exist");
    let sprite_scene_service = runtime
        .resolve::<SpriteSceneService>()
        .expect("sprite service should exist");
    let text2d_scene_service = runtime
        .resolve::<Text2dSceneService>()
        .expect("text2d service should exist");
    let vector_scene_service = runtime
        .resolve::<amigo_2d_vector::VectorSceneService>()
        .expect("vector service should exist");
    let mesh_scene_service = runtime
        .resolve::<amigo_3d_mesh::MeshSceneService>()
        .expect("mesh service should exist");
    let material_scene_service = runtime
        .resolve::<amigo_3d_material::MaterialSceneService>()
        .expect("material service should exist");
    let text3d_scene_service = runtime
        .resolve::<amigo_3d_text::Text3dSceneService>()
        .expect("text3d service should exist");
    let ui_scene_service = runtime
        .resolve::<UiSceneService>()
        .expect("ui service should exist");
    let ui_state_service = runtime
        .resolve::<UiStateService>()
        .expect("ui state should exist");
    let ui_theme_service = runtime
        .resolve::<UiThemeService>()
        .expect("ui theme should exist");
    let context = crate::render_runtime::AppRenderExtractContext {
        scene_service: scene_service.as_ref(),
        tilemap_scene_service: tilemap_scene_service.as_ref(),
        sprite_scene_service: sprite_scene_service.as_ref(),
        text2d_scene_service: text2d_scene_service.as_ref(),
        vector_scene_service: vector_scene_service.as_ref(),
        particle2d_scene_service: particles.as_ref(),
        mesh_scene_service: mesh_scene_service.as_ref(),
        material_scene_service: material_scene_service.as_ref(),
        text3d_scene_service: text3d_scene_service.as_ref(),
        ui_scene_service: ui_scene_service.as_ref(),
        ui_state_service: ui_state_service.as_ref(),
        ui_theme_service: ui_theme_service.as_ref(),
    };
    let packet = crate::render_runtime::default_app_render_extractor_registry().extract_all(&context);
    assert!(
        !packet.world_2d_particles().is_empty(),
        "render extraction should include generated particles"
    );
}

