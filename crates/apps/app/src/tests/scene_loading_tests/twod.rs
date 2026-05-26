use super::super::*;
use amigo_render_api::VisualSourceKind2d;
use amigo_layered_image_2d_plugin::LayeredImageSceneService;
use amigo_particles_2d_plugin::Particle2dSceneService;
use amigo_state::SceneStateService;

#[test]
fn playground_2d_basic_scripting_demo_bootstraps() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("basic-scripting-demo")
            .with_dev_mode(true),
    )
    .expect("2d scripting demo bootstrap should succeed");

    assert_eq!(
        summary.active_scene.as_deref(),
        Some("basic-scripting-demo")
    );
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/basic-scripting-demo/scene.yml")
    );
    assert!(
        summary
            .sprite_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-demo-square")
    );
    assert!(
        summary
            .sprite_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-demo-spritesheet")
    );
    assert!(
        summary
            .text_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-demo-title")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/spritesheets/square (sprite-sheet-2d)")
    );
    assert!(summary.prepared_assets.iter().any(
        |asset| asset == "playground-2d/spritesheets/hello-world-spritesheet (sprite-sheet-2d)"
    ));
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
    );
    assert!(
        summary
            .processed_script_events
            .iter()
            .any(|event| event == "playground-2d.demo.entered(basic-scripting-demo)")
    );
    assert!(
        summary
            .processed_script_events
            .iter()
            .any(|event| event == "playground-2d.demo.component.attach(playground-2d-demo-square)")
    );
    assert!(summary.failed_assets.is_empty());
}

#[test]
fn playground_2d_main_scene_bootstraps() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("hello-world-spritesheet")
            .with_dev_mode(true),
    )
    .expect("2d main playground bootstrap should succeed");

    assert_eq!(
        summary.active_scene.as_deref(),
        Some("hello-world-spritesheet")
    );
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/hello-world-spritesheet/scene.yml")
    );
    assert!(
        summary
            .sprite_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-spritesheet")
    );
    assert!(
        summary
            .text_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-hello")
    );
    assert!(summary.prepared_assets.iter().any(
        |asset| asset == "playground-2d/spritesheets/hello-world-spritesheet (sprite-sheet-2d)"
    ));
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
    );
    assert!(summary.failed_assets.is_empty());
}

#[test]
fn rotten_club_main_menu_queues_layered_image_background() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_startup_mod("rotten-club")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("they are rotten main menu bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("main-menu"));
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "rotten-club/layered-images/neon-alley (layered-image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "rotten-club/visual-maps/neon-alley-highlight (image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "rotten-club/depth-maps/neon-alley-depth (depth-map-2d)")
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "rotten-club/layered-images/neon-alley")
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "rotten-club/visual-maps/neon-alley-highlight")
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "rotten-club/depth-maps/neon-alley-depth")
    );
    assert!(summary.failed_assets.is_empty());

    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    let commands = layered_images.commands();
    let background = commands
        .iter()
        .find(|command| command.entity_name == "background")
        .expect("main menu background layered image should be queued");

    assert_eq!(
        background.image.asset.as_str(),
        "rotten-club/layered-images/neon-alley"
    );
    assert_eq!(
        background
            .image
            .visual_maps
            .as_ref()
            .and_then(|maps| maps.highlight.as_ref())
            .map(|key| key.as_str()),
        Some("rotten-club/visual-maps/neon-alley-highlight")
    );
    assert_eq!(background.image.size, amigo_math::Vec2::new(1280.0, 720.0));
    assert_eq!(background.image.base_opacity, 0.0);
    assert!(
        background
            .image
            .layer_overrides
            .iter()
            .any(|override_entry| override_entry.id == "rain_relief_edges")
    );
    assert!(
        background
            .image
            .layer_overrides
            .iter()
            .all(|override_entry| !override_entry.id.starts_with("puddle_reflection"))
    );
}

#[test]
fn rotten_club_main_menu_script_animates_layered_image_intro() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_startup_mod("rotten-club")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("they are rotten main menu bootstrap should succeed");
    let script_runtime = runtime
        .resolve::<amigo_scripting_api::ScriptRuntimeService>()
        .expect("script runtime should be registered");
    let scene_state = runtime
        .resolve::<SceneStateService>()
        .expect("scene state service should be registered");
    scene_state.set_float("lightning.next", 99.0);
    let particles = runtime
        .resolve::<Particle2dSceneService>()
        .expect("particle scene service should be registered");
    runtime
        .resolve::<amigo_runtime::SystemRegistry>()
        .expect("system registry should be registered")
        .run_phase(amigo_runtime::SystemPhase::PreUpdate, &runtime)
        .expect("focus targets should refresh before script update");

    script_runtime
        .call_update("scene:rotten-club:main-menu", 1.0)
        .expect("main menu script update should run");
    process_placeholder_bridges(&runtime).expect("intro update commands should dispatch");
    assert!(
        scene_state
            .get_float("intro.rain_intensity")
            .is_some_and(|rain| rain <= f64::EPSILON),
        "main menu rain should stay hidden during the first second"
    );
    assert_eq!(particles.intensity("rain-super-near"), 0.0);
    assert_eq!(
        scene_state.get_bool("intro.focus_hunt.near_started"),
        Some(true)
    );
    assert_eq!(
        scene_state.get_bool("intro.focus.skyline_started"),
        Some(false)
    );
    assert!(
        scene_state
            .get_float("camera_dof_controls.aperture_f_stop")
            .is_some_and(|f_stop| f_stop <= 1.8),
        "intro should keep the main camera in a visibly shallow depth-of-field range"
    );
    assert_eq!(particles.intensity("rain-10m"), 0.0);

    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    let commands = layered_images.commands();
    let command = commands
        .iter()
        .find(|command| command.entity_name == "background")
        .expect("background layered image command should exist");
    assert_eq!(command.image.base_opacity, 0.0);
    assert!(
        command
            .image
            .layer_overrides
            .iter()
            .any(|override_| override_.id == "skyline")
    );
    assert!(
        command
            .image
            .layer_overrides
            .iter()
            .any(|override_| { override_.id == "club_sign" && override_.opacity == Some(0.0) })
    );

    script_runtime
        .call_update("scene:rotten-club:main-menu", 6.0)
        .expect("main menu script update should run");
    process_placeholder_bridges(&runtime).expect("menu reveal commands should dispatch");

    let command = layered_images
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "background")
        .expect("background layered image command should exist");
    assert!(
        command
            .image
            .layer_overrides
            .iter()
            .any(|override_| override_.id == "bar_sign")
    );
    assert!(
        command
            .image
            .layer_overrides
            .iter()
            .any(|override_| override_.id == "club_sign")
    );

    script_runtime
        .call_update("scene:rotten-club:main-menu", 5.0)
        .expect("main menu script update should run");
    process_placeholder_bridges(&runtime).expect("rain reveal commands should dispatch");
    assert!(
        scene_state
            .get_float("intro.rain_intensity")
            .is_some_and(|rain| rain > 0.0),
        "main menu rain timing state should advance for the later weather pass"
    );
    assert!(
        particles.intensity("rain-super-near") > 0.0,
        "world-space rain should become visible once the intro reaches the weather pass"
    );
    assert!(particles.is_active("rain-super-near"));
}

#[test]
fn rotten_club_main_menu_camera_capture_sees_world_sources() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_startup_mod("rotten-club")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("they are rotten main menu bootstrap should succeed");
    assert!(!summary.prepared_assets.iter().any(|asset| asset
        == "rotten-club/camera/rain/realistic-lens-rain (camera-rain-glass-profile-2d)"));

    runtime
        .resolve::<amigo_runtime::SystemRegistry>()
        .expect("system registry should be registered")
        .run_phase(amigo_runtime::SystemPhase::PreUpdate, &runtime)
        .expect("focus targets should refresh before script update");
    runtime
        .resolve::<amigo_scripting_api::ScriptRuntimeService>()
        .expect("script runtime should be registered")
        .call_update("scene:rotten-club:main-menu", 12.0)
        .expect("main menu script update should run");
    process_placeholder_bridges(&runtime).expect("intro reveal commands should dispatch");
    runtime
        .resolve::<amigo_session::RuntimeSchedulingService>()
        .expect("runtime scheduling service should be registered")
        .set_mode(amigo_runtime::EngineSchedulerMode::SingleThread);
    amigo_particles_2d_plugin::tick_particles_2d_world(&runtime, 0.5)
        .expect("rain particles should tick before render extraction");
    let particles = runtime
        .resolve::<Particle2dSceneService>()
        .expect("particle scene service should be registered");
    let emitter_names = particles
        .emitters()
        .into_iter()
        .map(|emitter| emitter.entity_name)
        .collect::<Vec<_>>();
    assert!(
        emitter_names.iter().any(|name| name == "rain-super-near"),
        "rain emitters should be hydrated, got {emitter_names:?}"
    );
    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should be registered");
    assert!(scene.is_visible("rain-super-near"));
    assert!(scene.is_simulation_enabled("rain-super-near"));
    assert!(
        particles.intensity("rain-super-near") > 0.0,
        "intro rain should drive the nearest world-space emitter"
    );
    assert!(particles.is_active("rain-super-near"));
    assert!(
        particles
            .effective_max_particles("rain-super-near")
            .unwrap_or(0)
            > 0,
        "rain emitter should have a positive particle budget"
    );
    assert!(
        particles
            .effective_spawn_rate("rain-super-near")
            .unwrap_or(0.0)
            > 0.0,
        "rain emitter should have a positive spawn rate"
    );
    assert!(
        particles.particle_count("rain-super-near") > 0,
        "active rain should spawn near-camera particles before extraction"
    );
    assert!(
        particles.particle_count("rain-10m") > 0,
        "active rain should spawn far lightmap-reactive particles before extraction"
    );

    let packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(&runtime)
            .extract_all(&runtime);
    let capture = packet
        .camera_capture_input_2d()
        .expect("rotten club camera post-fx should publish capture input");

    let focus_blur = packet
        .post_fx_stacks()
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|instance| instance.effect.as_focus_blur())
        .expect("rotten club main camera should contribute FocusBlur");
    assert_eq!(focus_blur.depth_map.as_deref(), Some("main-menu-depth"));
    assert!(focus_blur.max_blur_px >= 30.0);
    assert!(focus_blur.background_blur_boost >= 1.5);
    assert!(focus_blur.highlight_threshold <= 0.45);
    assert!(focus_blur.highlight_gain >= 2.5);
    assert!(
        packet.render_depth_maps_2d().any(|depth_map| {
            depth_map.id == "main-menu-depth"
                && depth_map.asset.as_str() == "rotten-club/depth-maps/neon-alley-depth"
        }),
        "camera depth-map bokeh should bind the authored neon alley depth map"
    );
    assert!(
        !packet.post_fx_stacks().iter().any(|stack| {
            stack
                .effects
                .iter()
                .any(|instance| instance.effect.as_rain_glass().is_some())
        }),
        "lens rain glass is intentionally disabled while bokeh is tuned"
    );
    assert!(
        capture
            .layers
            .iter()
            .any(|layer| layer.layer_id == "background.city")
    );
    assert!(
        capture
            .layers
            .iter()
            .any(|layer| layer.layer_id == "weather.rain.10m")
    );
    assert!(
        capture.source(VisualSourceKind2d::LayerMask).is_some(),
        "camera capture should see render layers before post-fx extraction"
    );
    assert!(
        capture.source(VisualSourceKind2d::SceneHighlight).is_some(),
        "camera bokeh should receive an explicit background highlight source"
    );
    assert!(
        packet.renderables_2d().iter().any(|item| {
            item.owner_entity() == "rain-10m" && item.component_kind() == "ParticleEmitter2D"
        }),
        "enabled rain should submit lightmap-reactive particle renderables"
    );
    assert!(
        packet.world_2d_light_sources().iter().any(|source| {
            source.emitter_kind.as_str() == "lightmap_channel"
                && source
                    .emitter_id
                    .as_deref()
                    .is_some_and(|id| id.contains("neon-alley-lightmap"))
        }),
        "camera post-fx should run after lightmap sources are extracted"
    );
}

#[test]
fn rotten_club_main_menu_preview_is_not_black_after_warmup() {
    let mut preview = crate::ScenePreviewHost::new(
        crate::ScenePreviewOptions::new(mods_root(), "rotten-club", "main-menu", 320, 180)
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_warmup_frames(180)
            .with_playback_delta_seconds(1.0 / 30.0),
    );

    let frame = preview
        .capture_rgba8()
        .expect("rotten club preview should render");
    let non_black_pixels = frame
        .pixels_rgba8
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 8 || pixel[1] > 8 || pixel[2] > 8)
        .count();

    assert!(
        non_black_pixels > (frame.width as usize * frame.height as usize) / 100,
        "rotten club preview should contain visible non-black pixels"
    );
}

#[test]
fn playground_2d_scene_selection_rehydrates_document_content() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("2d sprite playground bootstrap should succeed");

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(amigo_scripting_api::DevConsoleCommand::new(
            "scene select text-lab",
        ));

    let bridge = crate::orchestration::process_placeholder_bridges(&runtime)
        .expect("scene selection bridge should succeed");
    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let hydrated = runtime
        .resolve::<HydratedSceneState>()
        .expect("hydrated scene state should exist");
    let sprite = runtime
        .resolve::<SpriteSceneService>()
        .expect("sprite scene service should exist");
    let text = runtime
        .resolve::<Text2dSceneService>()
        .expect("text scene service should exist");

    assert_eq!(
        scene.selected_scene().as_ref().map(|scene| scene.as_str()),
        Some("text-lab")
    );
    assert!(scene.entity_by_name("playground-2d-sprite").is_none());
    assert!(scene.entity_by_name("playground-2d-label").is_some());
    assert!(sprite.entity_names().is_empty());
    assert_eq!(text.entity_names(), vec!["playground-2d-label".to_owned()]);
    assert_eq!(hydrated.snapshot().scene_id.as_deref(), Some("text-lab"));
    assert!(
        bridge
            .processed_scene_commands
            .iter()
            .any(|command| command == "scene.select(text-lab)")
    );
    assert!(
        bridge
            .processed_scene_commands
            .iter()
            .any(|command| command.starts_with("scene.plugin.text("))
    );
}

#[test]
fn playground_2d_screen_space_preview_bootstraps() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("screen-space-preview")
            .with_dev_mode(true),
    )
    .expect("screen-space preview bootstrap should succeed");

    assert_eq!(
        summary.active_scene.as_deref(),
        Some("screen-space-preview")
    );
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/screen-space-preview/scene.yml")
    );
    assert!(
        summary
            .loaded_scene_document
            .as_ref()
            .expect("loaded scene document should exist")
            .component_kinds
            .iter()
            .any(|kind| kind == "UiDocument x1")
    );
    assert!(
        summary
            .ui_entities
            .iter()
            .any(|entity| entity == "playground-2d-ui-preview")
    );
    assert!(
        summary
            .sprite_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-ui-preview-square")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
    );
    assert!(summary.failed_assets.is_empty());
}

#[test]
fn playground_2d_script_component_updates_and_detaches() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("basic-scripting-demo")
            .with_dev_mode(true),
    )
    .expect("2d scripting demo bootstrap should succeed");

    amigo_runtime_bundles::tick_script_components(&runtime, 0.5)
        .expect("script component update should run");

    let scene_state = runtime
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state should exist");
    assert!(
        scene_state
            .get_float("playground-2d-demo-square.component.elapsed")
            .is_some_and(|elapsed| elapsed >= 0.5)
    );

    runtime
        .resolve::<SceneCommandQueue>()
        .expect("scene command queue should exist")
        .submit(SceneCommand::SelectScene {
            scene: SceneKey::new("hello-world-square"),
        });
    let updated =
        refresh_runtime_summary(&runtime).expect("runtime refresh should process scene transition");

    assert!(updated.processed_script_events.iter().any(|event| {
        event == "playground-2d.demo.component.detach(playground-2d-demo-square)"
    }));
}

#[test]
fn playground_2d_sprite_scene_populates_2d_domain_and_assets() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("2d sprite playground bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("sprite-lab"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/sprite-lab/scene.yml")
    );
    assert!(
        summary
            .processed_scene_commands
            .iter()
            .any(|command| command.starts_with("scene.plugin.sprite("))
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-2d/spritesheets/sprite-lab")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-2d/spritesheets/sprite-lab")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/spritesheets/sprite-lab (sprite-sheet-2d)")
    );
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(
        summary
            .sprite_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-sprite")
    );
    assert!(summary.text_entities_2d.is_empty());
}

#[test]
fn playground_2d_text_scene_populates_2d_text_domain_and_assets() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("text-lab")
            .with_dev_mode(true),
    )
    .expect("2d text playground bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("text-lab"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/text-lab/scene.yml")
    );
    assert!(
        summary
            .processed_scene_commands
            .iter()
            .any(|command| command.starts_with("scene.plugin.text("))
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-2d/fonts/debug-ui")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-2d/fonts/debug-ui")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
    );
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(
        summary
            .text_entities_2d
            .iter()
            .any(|entity| entity == "playground-2d-label")
    );
    assert!(summary.sprite_entities_2d.is_empty());
}

#[test]
fn playground_sidescroller_bootstraps_and_prepares_tile_and_sprite_assets() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-sidescroller".to_owned(),
            ])
            .with_startup_mod("playground-sidescroller")
            .with_startup_scene("vertical-slice")
            .with_dev_mode(true),
    )
    .expect("sidescroller bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("vertical-slice"));
    assert!(
        summary
            .sprite_entities_2d
            .iter()
            .any(|entity| entity == "playground-sidescroller-player")
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/player")
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/platformer")
    );
    assert!(
        summary.registered_assets.iter().any(|asset| asset
            == "playground-sidescroller/spritesheets/platformer/tilesets/platform/base")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/player")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/platformer")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/player (sprite-sheet-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset
                == "playground-sidescroller/spritesheets/platformer (sprite-sheet-2d)")
    );
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/platformer/tilesets/platform/base (tileset-2d)"));
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
}
