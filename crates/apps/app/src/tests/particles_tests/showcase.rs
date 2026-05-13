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
        .resolve::<amigo_runtime_bundles::amigo_ui::UiInputViewportState>()
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

    let resolved = amigo_runtime_bundles::amigo_ui::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let showcase = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-showcase-ui")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &showcase.overlay);
    let dropdown = find_layout_node_by_path_suffix(&layout, ".preset-options")
        .expect("preset dropdown should be in layout");
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
    amigo_runtime_bundles::amigo_ui::process_ui_input(&runtime).expect("dropdown press should process");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    amigo_runtime_bundles::amigo_ui::process_ui_input(&runtime).expect("dropdown release should expand");
    ui_input.clear_frame_transients();

    ui_input.set_mouse_position(
        dropdown.rect.x + dropdown.rect.width * 0.5,
        dropdown.rect.y + 38.0 * 4.5,
    );
    let target_offset = (lava_index as f32 - 4.0).max(0.0);
    ui_input.add_mouse_wheel(-(target_offset / 0.65));
    amigo_runtime_bundles::amigo_ui::process_ui_input(&runtime)
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
    amigo_runtime_bundles::amigo_ui::process_ui_input(&runtime)
        .expect("lava_sparks option press should process");
    ui_input.clear_frame_transients();
    ui_input.set_left_button(false);
    amigo_runtime_bundles::amigo_ui::process_ui_input(&runtime)
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
    amigo_runtime_bundles::amigo_2d_particles::tick_particles_2d_world(&runtime, 1.0 / 60.0)
        .expect("particle runtime tick should succeed");

    let particles = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_particles::Particle2dSceneService>()
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
        .resolve::<amigo_runtime_bundles::amigo_2d_particles::Particle2dSceneService>()
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
        .resolve::<amigo_runtime_bundles::amigo_2d_particles::ParticlePreset2dService>()
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
    let resolved = amigo_runtime_bundles::amigo_ui::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    let showcase_ui = resolved
        .iter()
        .find(|document| document.overlay.entity_name == "playground-2d-particles-showcase-ui")
        .expect("showcase ui should resolve");
    let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &showcase_ui.overlay);
    let dropdown = find_layout_node_by_path_suffix(&layout, ".preset-options")
        .expect("preset dropdown should exist");
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

    amigo_runtime_bundles::amigo_2d_particles::tick_particles_2d_world(&runtime, 1.0 / 10.0)
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
        .resolve::<amigo_runtime_bundles::amigo_2d_vector::VectorSceneService>()
        .expect("vector service should exist");
    let mesh_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_3d_mesh::MeshSceneService>()
        .expect("mesh service should exist");
    let material_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_3d_material::MaterialSceneService>()
        .expect("material service should exist");
    let text3d_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_3d_text::Text3dSceneService>()
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
    let layered_image_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService>()
        .expect("layered image service should exist");
    let global_light2d_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_lighting::GlobalLight2dSceneService>()
        .expect("global light2d service should exist");
    let lightmap2d_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_lighting::LightMap2dSceneService>()
        .expect("lightmap2d service should exist");
    let render_layer2d_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dSceneService>()
        .expect("render layer2d service should exist");
    let light_route2d_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_composition::LightRoute2dSceneService>()
        .expect("light route2d service should exist");
    let light_group2d_scene_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_lighting::LightGroup2dSceneService>()
        .expect("light group2d service should exist");
    let dev_console_state = runtime
        .resolve::<amigo_scripting_api::DevConsoleState>()
        .expect("dev console state should exist");
    let dev_console_completion = runtime
        .resolve::<crate::dev_console::completion::ConsoleCompletionState>()
        .expect("dev console completion should exist");
    let debug_overlay_service = runtime
        .resolve::<crate::debug_overlay::DebugOverlayService>()
        .expect("debug overlay service should exist");
    let post_fx_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>()
        .expect("post-fx service should exist");
    let ui_viewport_state = runtime
        .resolve::<amigo_runtime_bundles::amigo_ui::UiInputViewportState>()
        .expect("ui viewport state should exist");
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(&runtime);
    assert!(
        !packet.world_2d_particles().is_empty(),
        "render extraction should include generated particles"
    );
}









