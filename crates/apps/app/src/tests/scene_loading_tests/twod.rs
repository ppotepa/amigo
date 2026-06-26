use super::super::*;
use amigo_camera_core_plugin::{CameraId, CameraService};
use amigo_layered_image_2d_plugin::LayeredImageSceneService;
use amigo_particles_2d_plugin::Particle2dSceneService;

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
    assert!(summary
        .sprite_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-demo-square"));
    assert!(summary
        .sprite_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-demo-spritesheet"));
    assert!(summary
        .text_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-demo-title"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-2d/spritesheets/square (sprite-sheet-2d)"));
    assert!(summary.prepared_assets.iter().any(
        |asset| asset == "playground-2d/spritesheets/hello-world-spritesheet (sprite-sheet-2d)"
    ));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)"));
    assert!(summary
        .processed_script_events
        .iter()
        .any(|event| event == "playground-2d.demo.entered(basic-scripting-demo)"));
    assert!(summary
        .processed_script_events
        .iter()
        .any(|event| event == "playground-2d.demo.component.attach(playground-2d-demo-square)"));
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
    assert!(summary
        .sprite_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-spritesheet"));
    assert!(summary
        .text_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-hello"));
    assert!(summary.prepared_assets.iter().any(
        |asset| asset == "playground-2d/spritesheets/hello-world-spritesheet (sprite-sheet-2d)"
    ));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)"));
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
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "rotten-club/layered-images/neon-alley (layered-image-2d)"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "rotten-club/depth-maps/neon-alley-depth (depth-map-2d)"));
    assert!(summary.failed_assets.is_empty());

    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    let background = layered_images
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "rotten-club-background")
        .expect("single-plate Rotten Club background should be queued");
    assert_eq!(background.image.asset.as_str(), "rotten-club/layered-images/neon-alley");
    assert_eq!(background.image.size, amigo_math::Vec2::new(1672.0, 941.0));
    assert!(background.image.base_opacity <= 0.05);
    assert!(background
        .image
        .layer_overrides
        .iter()
        .any(|override_| override_.id == "skyline" && override_.opacity.unwrap_or(0.0) > 0.0));
    for layer_id in [
        "club_sign",
        "club_entry",
        "bar_sign",
        "bar_lanterns",
        "pharmacy_cross",
        "menu2_bar_warm_interior",
        "menu2_bar_sign",
        "menu2_apteka_interior",
        "menu2_apteka_cross",
        "menu2_club_main_sign",
        "menu2_klub_vertical",
    ] {
        assert!(
            background
                .image
                .layer_overrides
                .iter()
                .any(|override_| override_.id == layer_id && override_.opacity.unwrap_or(1.0) == 0.0),
            "foreground light layer `{layer_id}` should stay off in the opening checkpoint"
        );
    }
    for layer_id in [
        "menu2_background_palace_cool",
        "menu2_background_palace_pinlights",
        "menu2_misc_windows",
        "menu2_wet_reflections",
    ] {
        assert!(
            background
                .image
                .layer_overrides
                .iter()
                .any(|override_| override_.id == layer_id && override_.opacity.unwrap_or(0.0) > 0.0),
            "background light layer `{layer_id}` should be visible in the opening checkpoint"
        );
    }
}

#[test]
fn rotten_club_main_menu_timeline_animates_layered_image_intro() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_startup_mod("rotten-club")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("they are rotten main menu bootstrap should succeed");
    let particles = runtime
        .resolve::<Particle2dSceneService>()
        .expect("particle scene service should be registered");
    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    assert!(particles.emitters().is_empty());
    assert_eq!(layered_images.commands().len(), 1);
}

#[test]
fn rotten_club_timeline_drives_weather_camera_lighting_and_title() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_startup_mod("rotten-club")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("Rotten Club main menu bootstrap should succeed");
    let particles = runtime
        .resolve::<Particle2dSceneService>()
        .expect("particle scene service should be registered");
    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    let render_layers = runtime
        .resolve::<amigo_2d_composition::RenderLayer2dSceneService>()
        .expect("render layer scene service should be registered");
    let cameras = runtime
        .resolve::<CameraService>()
        .expect("camera service should be registered");

    process_placeholder_bridges(&runtime).expect("scene commands should dispatch");
    assert!(particles.emitters().is_empty());
    assert_eq!(layered_images.commands().len(), 1);
    assert!(render_layers.commands().iter().any(|layer| layer.id == "background.city"));
    let camera = cameras
        .get_2d(&CameraId::new("main"))
        .expect("single-plate checkpoint should hydrate the main camera");
    assert!((camera.aperture.focus_distance_m - 18.0).abs() < 0.01);
}

#[test]
fn rotten_club_authored_timeline_matches_script_beats() {
    let timeline_path = mods_root()
        .join("rotten-club")
        .join("scenes")
        .join("main-menu")
        .join("timeline")
        .join("intro.yml");
    let timeline_source =
        std::fs::read_to_string(&timeline_path).expect("Rotten Club timeline should be readable");
    let timeline: serde_yaml::Value =
        serde_yaml::from_str(&timeline_source).expect("Rotten Club timeline should parse");

    assert_eq!(timeline.get("kind").and_then(serde_yaml::Value::as_str), Some("scene-fragment"));
}

#[test]
fn rotten_club_main_menu_camera_capture_sees_world_sources() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_startup_mod("rotten-club")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("they are rotten main menu bootstrap should succeed");

    let packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(&runtime)
            .extract_all(&runtime);
    assert!(packet.renderables_2d().iter().any(|item| {
        item.owner_entity() == "rotten-club-background"
            && item.component_kind() == "LayeredImage2D"
    }));
    assert!(packet.render_depth_maps_2d().any(|depth_map| {
        depth_map.id == "main-depth"
            && depth_map.asset.as_str() == "rotten-club/depth-maps/neon-alley-depth"
    }));
    assert!(packet.post_fx_stacks().iter().any(|stack| {
        stack
            .effects
            .iter()
            .any(|instance| instance.effect.as_focus_blur().is_some())
    }));
    let focus_blur = packet
        .post_fx_stacks()
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|instance| instance.effect.as_focus_blur())
        .expect("main camera should contribute high-quality focus blur");
    assert_eq!(focus_blur.depth_map.as_deref(), Some("main-depth"));
    assert!(focus_blur.max_blur_px <= 8.0);
}

#[test]
fn rotten_club_main_menu_preview_is_not_black_after_warmup() {
    let mut preview = crate::ScenePreviewHost::new(
        crate::ScenePreviewOptions::new(mods_root(), "rotten-club", "main-menu", 320, 180)
            .with_active_mods(vec!["core".to_owned(), "rotten-club".to_owned()])
            .with_warmup_frames(900)
            .with_playback_delta_seconds(1.0 / 30.0),
    );

    let frame = preview
        .capture_rgba8()
        .expect("rotten club preview should render");
    assert_eq!(frame.pixels_rgba8.len(), frame.width as usize * frame.height as usize * 4);
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
    assert!(bridge
        .processed_scene_commands
        .iter()
        .any(|command| command == "scene.select(text-lab)"));
    assert!(bridge
        .processed_scene_commands
        .iter()
        .any(|command| command.starts_with("scene.plugin.text(")));
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
    assert!(summary
        .loaded_scene_document
        .as_ref()
        .expect("loaded scene document should exist")
        .component_kinds
        .iter()
        .any(|kind| kind == "UiDocument x1"));
    assert!(summary
        .ui_entities
        .iter()
        .any(|entity| entity == "playground-2d-ui-preview"));
    assert!(summary
        .sprite_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-ui-preview-square"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)"));
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
    assert!(scene_state
        .get_float("playground-2d-demo-square.component.elapsed")
        .is_some_and(|elapsed| elapsed >= 0.5));

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
    assert!(summary
        .processed_scene_commands
        .iter()
        .any(|command| command.starts_with("scene.plugin.sprite(")));
    assert!(summary
        .registered_assets
        .iter()
        .any(|asset| asset == "playground-2d/spritesheets/sprite-lab"));
    assert!(summary
        .loaded_assets
        .iter()
        .any(|asset| asset == "playground-2d/spritesheets/sprite-lab"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-2d/spritesheets/sprite-lab (sprite-sheet-2d)"));
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(summary
        .sprite_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-sprite"));
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
    assert!(summary
        .processed_scene_commands
        .iter()
        .any(|command| command.starts_with("scene.plugin.text(")));
    assert!(summary
        .registered_assets
        .iter()
        .any(|asset| asset == "playground-2d/fonts/debug-ui"));
    assert!(summary
        .loaded_assets
        .iter()
        .any(|asset| asset == "playground-2d/fonts/debug-ui"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)"));
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(summary
        .text_entities_2d
        .iter()
        .any(|entity| entity == "playground-2d-label"));
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
    assert!(summary
        .sprite_entities_2d
        .iter()
        .any(|entity| entity == "playground-sidescroller-player"));
    assert!(summary
        .registered_assets
        .iter()
        .any(|asset| asset == "playground-sidescroller/spritesheets/player"));
    assert!(summary
        .registered_assets
        .iter()
        .any(|asset| asset == "playground-sidescroller/spritesheets/platformer"));
    assert!(summary
        .registered_assets
        .iter()
        .any(|asset| asset
            == "playground-sidescroller/spritesheets/platformer/tilesets/platform/base"));
    assert!(summary
        .loaded_assets
        .iter()
        .any(|asset| asset == "playground-sidescroller/spritesheets/player"));
    assert!(summary
        .loaded_assets
        .iter()
        .any(|asset| asset == "playground-sidescroller/spritesheets/platformer"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-sidescroller/spritesheets/player (sprite-sheet-2d)"));
    assert!(summary
        .prepared_assets
        .iter()
        .any(|asset| asset == "playground-sidescroller/spritesheets/platformer (sprite-sheet-2d)"));
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/platformer/tilesets/platform/base (tileset-2d)"));
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
}
