use super::super::*;

use std::fs;

#[test]
fn playground_3d_main_scene_bootstraps() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
            .with_startup_mod("playground-3d")
            .with_startup_scene("hello-world-cube")
            .with_dev_mode(true),
    )
    .expect("3d main playground bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("hello-world-cube"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/hello-world-cube/scene.yml")
    );
    assert!(
        summary
            .mesh_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-cube")
    );
    assert!(
        summary
            .material_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-cube")
    );
    assert!(
        summary
            .text_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-hello")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/cube (mesh-3d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-3d/materials/cube-material (material-3d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-3d/fonts/debug-3d (font-3d)")
    );
    assert!(summary.failed_assets.is_empty());
}

#[test]
fn playground_3d_material_scene_populates_3d_material_domain_and_assets() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
            .with_startup_mod("playground-3d")
            .with_startup_scene("material-lab")
            .with_dev_mode(true),
    )
    .expect("3d material playground bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("material-lab"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/material-lab/scene.yml")
    );
    assert!(
        summary
            .processed_scene_commands
            .iter()
            .any(|command| command.starts_with("scene.3d.material("))
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/material-probe")
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-3d/materials/debug-surface")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/material-probe")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-3d/materials/debug-surface")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/material-probe (mesh-3d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-3d/materials/debug-surface (material-3d)")
    );
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(
        summary
            .mesh_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-material-probe")
    );
    assert!(
        summary
            .material_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-material-probe")
    );
}

#[test]
fn playground_3d_mesh_scene_populates_3d_domain_and_assets() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
            .with_startup_mod("playground-3d")
            .with_startup_scene("mesh-lab")
            .with_dev_mode(true),
    )
    .expect("3d mesh playground bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("mesh-lab"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/mesh-lab/scene.yml")
    );
    assert!(
        summary
            .processed_scene_commands
            .iter()
            .any(|command| command.starts_with("scene.3d.mesh("))
    );
    assert!(
        summary
            .registered_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/probe")
    );
    assert!(
        summary
            .loaded_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/probe")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-3d/meshes/probe (mesh-3d)")
    );
    assert!(summary.failed_assets.is_empty());
    assert!(summary.pending_asset_loads.is_empty());
    assert!(
        summary
            .mesh_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-probe")
    );
    assert!(summary.material_entities_3d.is_empty());
}

#[test]
fn playground_sidescroller_tilemap_bootstraps_without_ruleset() {
    let temp_mods = copied_mods_root(
        "sidescroller-no-ruleset",
        &["core", "playground-sidescroller"],
    );
    let scene_path = temp_mods
        .join("playground-sidescroller")
        .join("scenes")
        .join("vertical-slice")
        .join("scene.yml");
    let original_scene =
        fs::read_to_string(&scene_path).expect("sidescroller scene should be readable");
    let updated_scene = original_scene
        .lines()
        .filter(|line| !line.contains("ruleset: playground-sidescroller/tilesets/platformer-rules"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&scene_path, updated_scene).expect("scene without ruleset should be writable");

    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-sidescroller".to_owned(),
            ])
            .with_startup_mod("playground-sidescroller")
            .with_startup_scene("vertical-slice")
            .with_dev_mode(true),
    )
    .expect("sidescroller bootstrap without ruleset should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("vertical-slice"));
    assert!(summary.failed_assets.is_empty());

    let tilemap_command = runtime
        .resolve::<TileMap2dSceneService>()
        .expect("tilemap scene service should exist")
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "playground-sidescroller-tilemap")
        .expect("tilemap command should exist");
    assert!(tilemap_command.tilemap.ruleset.is_none());
    assert!(tilemap_command.tilemap.resolved.is_none());
}

#[test]
fn playground_sidescroller_vertical_slice_bootstraps() {
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
    .expect("sidescroller vertical slice bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("vertical-slice"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/vertical-slice/scene.yml")
    );
    let component_kinds = &summary
        .loaded_scene_document
        .as_ref()
        .expect("loaded scene document should exist")
        .component_kinds;
    assert!(component_kinds.iter().any(|kind| kind == "TileMap2D x1"));
    assert!(
        component_kinds
            .iter()
            .any(|kind| kind == "KinematicBody2D x1")
    );
    assert!(
        component_kinds
            .iter()
            .any(|kind| kind == "AabbCollider2D x1")
    );
    assert!(
        component_kinds
            .iter()
            .any(|kind| kind == "MotionController2D x1")
    );
    assert!(
        component_kinds
            .iter()
            .any(|kind| kind == "CameraFollow2D x1")
    );
    assert!(component_kinds.iter().any(|kind| kind == "Parallax2D x4"));
    assert!(
        component_kinds
            .iter()
            .any(|kind| kind == "TileMapMarker2D x27")
    );
    assert!(component_kinds.iter().any(|kind| kind == "Trigger2D x26"));
    assert!(component_kinds.iter().any(|kind| kind == "UiDocument x1"));

    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-background-layer-01")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-background-layer-02")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-background-layer-03")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-background-layer-04")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-player")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-coin-25")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-tilemap")
    );
    assert!(
        summary
            .scene_entities
            .iter()
            .any(|entity| entity == "playground-sidescroller-hud")
    );
    let player_transform = _runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-player")
        .expect("player transform should exist after tilemap marker anchoring");
    assert!(
        player_transform.translation.x > 0.0 && player_transform.translation.y > 0.0,
        "player should be anchored to a non-zero tilemap marker position"
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/backgrounds/layer-01 (image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/backgrounds/layer-02 (image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/backgrounds/layer-03 (image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/backgrounds/layer-04 (image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/textures/player (sprite-sheet-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/textures/coin (sprite-sheet-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/textures/finish (image-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/tilesets/platformer (tileset-2d)")
    );
    assert!(summary.prepared_assets.iter().any(|asset| {
        asset == "playground-sidescroller/tilesets/platformer-rules (tile-ruleset-2d)"
    }));
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/fonts/debug-ui (font-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| { asset == "playground-sidescroller/audio/jump (generated-audio)" })
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| { asset == "playground-sidescroller/audio/coin (generated-audio)" })
    );
    assert!(summary.prepared_assets.iter().any(|asset| {
        asset == "playground-sidescroller/audio/level-complete (generated-audio)"
    }));
    assert!(summary.prepared_assets.iter().any(|asset| {
        asset == "playground-sidescroller/audio/proximity-beep (generated-audio)"
    }));
    assert_eq!(summary.audio_master_volume, 1.0);
    assert!(summary.audio_sources.is_empty());
    assert!(
        summary
            .pending_audio_runtime_commands
            .iter()
            .any(|entry| entry == "audio.play(playground-sidescroller/audio/jump)")
    );
    assert!(!summary.audio_output_started);
    assert!(summary.failed_assets.is_empty());
}
