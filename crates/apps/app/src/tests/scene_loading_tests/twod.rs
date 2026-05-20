use super::super::*;
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
            .registered_assets
            .iter()
            .any(|asset| asset == "rotten-club/layered-images/neon-alley")
    );
    assert!(summary.failed_assets.is_empty());

    let layered_images = runtime
        .resolve::<amigo_runtime_bundles::LayeredImageSceneService>()
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

    script_runtime
        .call_update("scene:rotten-club:main-menu", 1.0)
        .expect("main menu script update should run");
    process_placeholder_bridges(&runtime).expect("intro update commands should dispatch");

    let layered_images = runtime
        .resolve::<amigo_runtime_bundles::LayeredImageSceneService>()
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
            .any(|command| command.starts_with("scene.2d.text("))
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
            .any(|command| command.starts_with("scene.2d.sprite("))
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
            .any(|command| command.starts_with("scene.2d.text("))
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
