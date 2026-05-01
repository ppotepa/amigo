use super::super::*;

#[test]
fn core_game_console_scene_processes_placeholder_queues() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
            .with_startup_mod("core-game")
            .with_startup_scene("console")
            .with_dev_mode(true),
    )
    .expect("console bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("console"));
    assert!(
        summary
            .console_commands
            .iter()
            .any(|command| command == "help")
    );
    assert!(
        summary
            .console_output
            .iter()
            .any(|line| line.contains("available placeholder commands"))
    );
    assert!(
        summary
            .processed_script_events
            .iter()
            .any(|event| event == "core-game.bootstrapped(console)")
    );
}

#[test]
fn core_game_diagnostics_scene_writes_refresh_output() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
            .with_startup_mod("core-game")
            .with_startup_scene("diagnostics")
            .with_dev_mode(true),
    )
    .expect("diagnostics bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("diagnostics"));
    assert!(
        summary
            .processed_script_commands
            .iter()
            .any(|command| command == "dev-shell.refresh-diagnostics(core-game)")
    );
    assert!(
        summary
            .console_output
            .iter()
            .any(|line| line.contains("diagnostics refreshed for mod=core-game"))
    );
    assert!(
        summary
            .processed_script_events
            .iter()
            .any(|event| event == "dev-shell.diagnostics-refreshed(core-game)")
    );
}

#[test]
fn playground_2d_asteroids_main_menu_bootstraps() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-asteroids".to_owned(),
            ])
            .with_startup_mod("playground-2d-asteroids")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("asteroids main menu bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("main-menu"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/main-menu/scene.yml")
    );
    assert!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document
                .component_kinds
                .iter()
                .any(|kind| kind.starts_with("UiDocument x")))
            .unwrap_or(false)
    );
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d-asteroids/fonts/debug-ui (font-2d)")
    );
    assert!(
        summary
            .ui_entities
            .iter()
            .any(|entity| entity == "playground-2d-asteroids-main-menu")
    );
}

