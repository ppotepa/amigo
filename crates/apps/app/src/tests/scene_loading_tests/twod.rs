use super::super::*;

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
            .any(|asset| asset == "playground-2d/textures/square (sprite-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/textures/hello-world-spritesheet (sprite-2d)")
    );
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
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/textures/hello-world-spritesheet (sprite-2d)")
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

    crate::systems::script_components::tick_script_components(&runtime, 0.5)
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
            .any(|asset| asset == "playground-2d/textures/sprite-lab")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-2d/textures/sprite-lab")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/textures/sprite-lab (sprite-2d)")
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

