use super::*;

#[test]
fn bootstrap_reports_file_watch_backend() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("sprite playground bootstrap should succeed");

    assert!(summary.file_watch_backend.starts_with("notify"));
}

#[test]
fn runtime_detects_asset_file_changes_through_hot_reload_service() {
    let temp_mods = copied_mods_root("asset-hot-reload", &["core", "playground-2d"]);
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods.clone())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("sprite playground bootstrap should succeed");

    fs::write(
        temp_mods
            .join("playground-2d")
            .join("spritesheets")
            .join("sprite-lab")
            .join("spritesheet.yml"),
        "kind: spritesheet-2d\nschema_version: 1\nid: sprite-lab\nlabel: Reloaded Sprite\nformat: debug-placeholder\n",
    )
    .expect("asset file should be updated");

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should detect asset file changes");

    assert!(updated.console_output.iter().any(|line| {
        line.contains("detected asset change for `playground-2d/spritesheets/sprite-lab`")
    }));
    assert!(
        updated
            .processed_script_events
            .iter()
            .any(|event| event.starts_with("hot-reload.asset-changed("))
    );
    assert!(
        updated
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/spritesheets/sprite-lab (sprite-sheet-2d)")
    );
}

#[test]
fn runtime_detects_scene_document_file_changes_through_hot_reload_service() {
    let temp_mods = copied_mods_root("scene-hot-reload", &["core", "playground-2d"]);
    let scene_path = temp_mods
        .join("playground-2d")
        .join("scenes")
        .join("sprite-lab")
        .join("scene.yml");
    let original_scene =
        fs::read_to_string(&scene_path).expect("scene document should be readable");

    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("sprite playground bootstrap should succeed");

    fs::write(
        &scene_path,
        original_scene.replace("playground-2d-sprite", "playground-2d-sprite-live"),
    )
    .expect("scene document should be updated");

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should detect scene document changes");

    assert_eq!(updated.active_scene.as_deref(), Some("sprite-lab"));
    assert!(
        updated
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-2d-sprite-live")
    );
    assert!(
        updated
            .scene_entities
            .iter()
            .all(|entity| entity != "playground-2d-sprite")
    );
    assert!(updated.console_output.iter().any(|line| {
        line.contains("detected scene document change for `playground-2d:sprite-lab`")
    }));
    assert!(
        updated
            .processed_scene_commands
            .iter()
            .any(|command| command == "scene.reload_active")
    );
}

#[test]
fn runtime_detects_sidescroller_generated_audio_metadata_changes_through_hot_reload_service() {
    let temp_mods = copied_mods_root(
        "sidescroller-audio-hot-reload",
        &["core", "playground-sidescroller"],
    );
    let asset_path = temp_mods
        .join("playground-sidescroller")
        .join("audio")
        .join("proximity-beep")
        .join("audio.yml");
    let original_asset =
        fs::read_to_string(&asset_path).expect("audio metadata should be readable");

    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-sidescroller".to_owned(),
            ])
            .with_startup_mod("playground-sidescroller")
            .with_startup_scene("vertical-slice")
            .with_dev_mode(true),
    )
    .expect("sidescroller bootstrap should succeed");

    fs::write(
        &asset_path,
        original_asset.replace(
            "label: Sidescroller Proximity Beep",
            "label: Sidescroller Proximity Beep Reloaded",
        ),
    )
    .expect("audio metadata should be updated");

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should detect audio metadata changes");

    assert!(updated.console_output.iter().any(|line| {
        line.contains("detected asset change for `playground-sidescroller/audio/proximity-beep`")
    }));
    assert!(
        updated
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/audio/proximity-beep (generated-audio)")
    );

    let assets = runtime
        .resolve::<AssetCatalog>()
        .expect("asset catalog should exist");
    let prepared = assets
        .prepared_asset(&AssetKey::new(
            "playground-sidescroller/audio/proximity-beep",
        ))
        .expect("audio prepared asset should exist after reload");
    assert_eq!(
        prepared.label.as_deref(),
        Some("Sidescroller Proximity Beep Reloaded")
    );
}

#[test]
fn runtime_detects_sidescroller_scene_document_changes_through_hot_reload_service() {
    let temp_mods = copied_mods_root(
        "sidescroller-scene-hot-reload",
        &["core", "playground-sidescroller"],
    );
    let scene_path = temp_mods
        .join("playground-sidescroller")
        .join("scenes")
        .join("vertical-slice")
        .join("scene.yml");
    let original_scene =
        fs::read_to_string(&scene_path).expect("scene document should be readable");

    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-sidescroller".to_owned(),
            ])
            .with_startup_mod("playground-sidescroller")
            .with_startup_scene("vertical-slice")
            .with_dev_mode(true),
    )
    .expect("sidescroller bootstrap should succeed");

    fs::write(
        &scene_path,
        original_scene.replace("PLAYGROUND SIDESCROLLER", "PLAYGROUND SIDESCROLLER LIVE"),
    )
    .expect("scene document should be updated");

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should detect sidescroller scene changes");

    assert_eq!(updated.active_scene.as_deref(), Some("vertical-slice"));
    assert!(updated.console_output.iter().any(|line| {
        line.contains("detected scene document change for `playground-sidescroller:vertical-slice`")
    }));
    assert!(
        updated
            .processed_scene_commands
            .iter()
            .any(|command| command == "scene.reload_active")
    );

    let ui_scene = runtime
        .resolve::<UiSceneService>()
        .expect("ui scene service should exist");
    let title = ui_scene
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "playground-sidescroller-hud")
        .and_then(|command| {
            command.document.root.children.into_iter().find_map(|node| {
                match (node.id.as_deref(), node.kind) {
                    (Some("title"), amigo_ui::UiNodeKind::Text { content, .. }) => Some(content),
                    _ => None,
                }
            })
        });
    assert_eq!(title.as_deref(), Some("PLAYGROUND SIDESCROLLER LIVE"));
}

#[test]
fn runtime_detects_sidescroller_tile_ruleset_changes_through_hot_reload_service() {
    let temp_mods = copied_mods_root(
        "sidescroller-ruleset-hot-reload",
        &["core", "playground-sidescroller"],
    );
    let asset_path = temp_mods
        .join("playground-sidescroller")
        .join("spritesheets")
        .join("platformer")
        .join("rulesets")
        .join("platform")
        .join("rules.yml");
    let original_asset =
        fs::read_to_string(&asset_path).expect("ruleset metadata should be readable");

    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-sidescroller".to_owned(),
            ])
            .with_startup_mod("playground-sidescroller")
            .with_startup_scene("vertical-slice")
            .with_dev_mode(true),
    )
    .expect("sidescroller bootstrap should succeed");

    let initial_center = first_resolved_tile_id_for_variant(&runtime, TileVariantKind2d::Center)
        .expect("initial center tile id should exist");
    assert_eq!(initial_center, 6);

    fs::write(
        &asset_path,
        original_asset.replace("center: 6", "center: 0"),
    )
    .expect("ruleset metadata should be updated");

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should detect ruleset metadata changes");

    assert!(updated.console_output.iter().any(|line| {
        line.contains(
            "detected asset change for `playground-sidescroller/spritesheets/platformer/rulesets/platform/rules`",
        )
    }));
    assert!(updated.prepared_assets.iter().any(|asset| {
        asset == "playground-sidescroller/spritesheets/platformer/rulesets/platform/rules (tile-ruleset-2d)"
    }));

    let updated_center = first_resolved_tile_id_for_variant(&runtime, TileVariantKind2d::Center)
        .expect("updated center tile id should exist");
    assert_eq!(updated_center, 0);
}

#[test]
fn runtime_detects_sidescroller_visual_asset_metadata_changes_through_hot_reload_service() {
    let temp_mods = copied_mods_root(
        "sidescroller-player-hot-reload",
        &["core", "playground-sidescroller"],
    );
    let asset_path = temp_mods
        .join("playground-sidescroller")
        .join("spritesheets")
        .join("player")
        .join("spritesheet.yml");
    let original_asset =
        fs::read_to_string(&asset_path).expect("player metadata should be readable");

    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-sidescroller".to_owned(),
            ])
            .with_startup_mod("playground-sidescroller")
            .with_startup_scene("vertical-slice")
            .with_dev_mode(true),
    )
    .expect("sidescroller bootstrap should succeed");

    fs::write(
        &asset_path,
        original_asset.replace(
            "label: Sidescroller Player",
            "label: Sidescroller Player Reloaded",
        ),
    )
    .expect("player metadata should be updated");

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should detect player metadata changes");

    assert!(updated.console_output.iter().any(|line| {
        line.contains("detected asset change for `playground-sidescroller/spritesheets/player`")
    }));
    assert!(
        updated
            .processed_script_events
            .iter()
            .any(|event| event.starts_with("hot-reload.asset-changed("))
    );
    assert!(
        updated
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/player (sprite-sheet-2d)")
    );

    let assets = runtime
        .resolve::<AssetCatalog>()
        .expect("asset catalog should exist");
    let prepared = assets
        .prepared_asset(&AssetKey::new("playground-sidescroller/spritesheets/player"))
        .expect("player prepared asset should exist after reload");
    assert_eq!(
        prepared.label.as_deref(),
        Some("Sidescroller Player Reloaded")
    );
}
