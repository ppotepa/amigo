use super::super::*;

use std::fs;

fn capture_npr_candidate(name: &str, pixels_rgba8: &[u8]) {
    let Some(root) = std::env::var_os("AMIGO_CAPTURE_NPR_GOLDEN_DIR") else {
        return;
    };
    let path = std::path::PathBuf::from(root).join(format!("{name}.png"));
    fs::create_dir_all(path.parent().expect("candidate path has a parent")).unwrap();
    image::save_buffer(&path, pixels_rgba8, 512, 512, image::ColorType::Rgba8)
        .expect("candidate capture should be writable");
}

/// Pixel images are an art-review artifact, not the primary renderer contract.
/// They are checked only in an explicit review run because GPU rasterisation is
/// device-dependent. Regular tests lock the deterministic backend packet.
fn verify_reviewed_npr_image(name: &str, pixels_rgba8: &[u8]) {
    if std::env::var_os("AMIGO_VERIFY_NPR_GOLDEN").is_none() {
        return;
    }
    let golden_path = mods_root().join(format!("npr-playground/tests/golden/{name}.png"));
    let expected = image::open(&golden_path)
        .expect("reviewed NPR golden must exist")
        .to_rgba8();
    let diff = amigo_render_api::compare_golden_rgba8(512, 512, expected.as_raw(), pixels_rgba8)
        .expect("NPR preview buffers should have golden dimensions");
    assert!(
        diff.passes(amigo_render_api::GoldenImageTolerance {
            max_channel_delta: 255,
            max_mismatched_pixels: 512,
        }),
        "{name}: {diff:?}"
    );
}

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
fn npr_playground_offscreen_matches_packet_contract() {
    let options = crate::ScenePreviewOptions::new(mods_root(), "npr-playground", "cube", 512, 512)
        .with_active_mods(vec!["core".to_owned(), "npr-playground".to_owned()])
        .with_warmup_frames(0)
        .with_playback_delta_seconds(1.0 / 60.0);
    let mut preview = crate::ScenePreviewHost::new(options);
    preview.warmup(1).unwrap();
    let controls = preview
        .runtime()
        .unwrap()
        .required::<amigo_runtime_control::RuntimeControlService>()
        .unwrap();
    let prefix = "world.npr.settings.NprSettings.";
    controls
        .set(
            &format!("{prefix}paused"),
            amigo_runtime_control::ControlValue::Bool(true),
        )
        .unwrap();
    controls
        .set(
            &format!("{prefix}object.rotation"),
            amigo_runtime_control::ControlValue::Vec3([
                0.36_f32.to_degrees(),
                0.71_f32.to_degrees(),
                0.0,
            ]),
        )
        .unwrap();
    controls
        .set(
            &format!("{prefix}seed"),
            amigo_runtime_control::ControlValue::U64(42),
        )
        .unwrap();
    let first = preview
        .capture_rgba8()
        .expect("NPR preview should render offscreen");
    capture_npr_candidate("cube-512-candidate", &first.pixels_rgba8);
    verify_reviewed_npr_image("cube-512", &first.pixels_rgba8);
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
        preview.runtime().unwrap(),
    )
    .extract_all(preview.runtime().unwrap());
    let stats = &packet.npr()[0].packet.stats;
    assert_eq!(stats.geometry, 1);
    assert_eq!(stats.topology_edges, 18);
    assert_eq!(stats.feature_segments, 12);
    assert_eq!(stats.viewport, [512, 512]);
    assert_eq!(
        packet.npr()[0].packet.fingerprint().hash,
        6_968_191_395_311_846_154
    );
    assert!(
        first
            .pixels_rgba8
            .chunks_exact(4)
            .any(|pixel| pixel[0] != 0)
    );
}

#[test]
fn npr_pencil_profile_uses_depth_occluders_without_color_bands() {
    let options = crate::ScenePreviewOptions::new(mods_root(), "npr-playground", "cube", 512, 512)
        .with_active_mods(vec!["core".to_owned(), "npr-playground".to_owned()])
        .with_warmup_frames(0)
        .with_playback_delta_seconds(1.0 / 60.0);
    let mut preview = crate::ScenePreviewHost::new(options);
    preview.warmup(1).unwrap();
    let controls = preview
        .runtime()
        .unwrap()
        .required::<amigo_runtime_control::RuntimeControlService>()
        .unwrap();
    let prefix = "world.npr.settings.NprSettings.";
    controls
        .set(
            &format!("{prefix}style_preset"),
            amigo_runtime_control::ControlValue::String("Pencil Study".into()),
        )
        .unwrap();
    controls
        .set(
            &format!("{prefix}paused"),
            amigo_runtime_control::ControlValue::Bool(true),
        )
        .unwrap();
    let image = preview
        .capture_rgba8()
        .expect("pencil profile should render offscreen");
    capture_npr_candidate("pencil-cube-512-candidate", &image.pixels_rgba8);
    verify_reviewed_npr_image("pencil-cube-512", &image.pixels_rgba8);
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
        preview.runtime().unwrap(),
    )
    .extract_all(preview.runtime().unwrap());
    let command = &packet.npr()[0];
    assert!(!command.packet.occluders.is_empty());
    assert!(command.packet.fills.is_empty());
    assert!(command.packet.stats.hatching_strokes > 0);
    assert_eq!(command.packet.fingerprint().hash, 16_406_345_649_915_924_628);
    let darkest = image
        .pixels_rgba8
        .chunks_exact(4)
        .map(|pixel| pixel[0].min(pixel[1]).min(pixel[2]))
        .min()
        .unwrap();
    assert!(
        darkest < 200,
        "pencil output lacks contrast against paper: {darkest}"
    );
}

#[test]
fn npr_pencil_cylinder_streamlines_match_reviewed_golden() {
    let options = crate::ScenePreviewOptions::new(mods_root(), "npr-playground", "cube", 512, 512)
        .with_active_mods(vec!["core".to_owned(), "npr-playground".to_owned()])
        .with_warmup_frames(0)
        .with_playback_delta_seconds(1.0 / 60.0);
    let mut preview = crate::ScenePreviewHost::new(options);
    preview.warmup(1).unwrap();
    let controls = preview
        .runtime()
        .unwrap()
        .required::<amigo_runtime_control::RuntimeControlService>()
        .unwrap();
    let prefix = "world.npr.settings.NprSettings.";
    controls
        .set(
            &format!("{prefix}selected"),
            amigo_runtime_control::ControlValue::String("cylinder".into()),
        )
        .unwrap();
    controls
        .set(
            &format!("{prefix}style_preset"),
            amigo_runtime_control::ControlValue::String("Pencil Study".into()),
        )
        .unwrap();
    controls
        .set(
            &format!("{prefix}paused"),
            amigo_runtime_control::ControlValue::Bool(true),
        )
        .unwrap();
    let image = preview
        .capture_rgba8()
        .expect("pencil cylinder should render offscreen");
    capture_npr_candidate("pencil-cylinder-512-candidate", &image.pixels_rgba8);
    verify_reviewed_npr_image("pencil-cylinder-512", &image.pixels_rgba8);
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
        preview.runtime().unwrap(),
    )
    .extract_all(preview.runtime().unwrap());
    let command = &packet.npr()[0];
    assert!(!command.packet.occluders.is_empty());
    assert!(command.packet.fills.is_empty());
    assert!(command.packet.stats.hatching_strokes > 0);
    assert_eq!(command.packet.fingerprint().hash, 13_897_957_754_279_461_727);
    assert!(
        command
            .packet
            .strokes
            .iter()
            .any(|stroke| stroke.vertices.len() > 8),
        "cylinder needs multi-sample tonal streamlines rather than isolated segments"
    );
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
    assert!(summary.processed_scene_commands.iter().any(|command| {
        command.starts_with("scene.plugin(amigo.rendering.3d.scene-command.Material3d)")
    }));
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
fn panel_playground_uses_layer_metadata_and_rhai_without_npr() {
    let mut preview = crate::ScenePreviewHost::new(
        crate::ScenePreviewOptions::new(mods_root(), "panel-playground", "layer", 320, 240)
            .with_active_mods(vec![
                "core".into(),
                "playground-2d".into(),
                "panel-playground".into(),
            ])
            .with_warmup_frames(0),
    );
    preview.warmup(1).unwrap();
    let runtime = preview.runtime().unwrap();
    let controls = runtime
        .required::<amigo_runtime_control::RuntimeControlService>()
        .unwrap();
    controls
        .set(
            "world.demo.RenderLayer2D.opacity",
            amigo_runtime_control::ControlValue::F64(0.25),
        )
        .unwrap();
    runtime
        .required::<amigo_scripting_api::ScriptEventQueue>()
        .unwrap()
        .publish(amigo_scripting_api::ScriptEvent::new("layer.reset", vec![]));
    preview.warmup(1).unwrap();
    assert_eq!(
        controls.get("world.demo.RenderLayer2D.opacity").unwrap(),
        amigo_runtime_control::ControlValue::F64(1.0)
    );
    let runtime = preview.runtime().unwrap();
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(runtime)
        .extract_all(runtime);
    assert!(packet.npr().is_empty());
}

#[test]
fn playground_3d_physics_scene_bootstraps() {
    let (_runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
            .with_startup_mod("playground-3d")
            .with_startup_scene("physics-cubes")
            .with_dev_mode(true),
    )
    .expect("3d physics playground bootstrap should succeed");

    assert_eq!(summary.active_scene.as_deref(), Some("physics-cubes"));
    assert_eq!(
        summary
            .loaded_scene_document
            .as_ref()
            .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
            .as_deref(),
        Some("scenes/physics-cubes/scene.yml")
    );
    assert!(
        summary
            .mesh_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-ground")
    );
    assert!(
        summary
            .text_entities_3d
            .iter()
            .any(|entity| entity == "playground-3d-physics-label")
    );
    assert!(summary.failed_assets.is_empty());
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
    assert!(summary.processed_scene_commands.iter().any(|command| {
        command.starts_with("scene.plugin(amigo.rendering.3d.scene-command.Mesh3d)")
    }));
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
        .filter(|line| {
            !line.contains(
                "ruleset: playground-sidescroller/spritesheets/platformer/rulesets/platform/rules",
            )
        })
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
    assert!(
        component_kinds
            .iter()
            .any(|kind| kind == "amigo.gfx.tilemap-2d.TileMap2D x1")
    );
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
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/background-layer-01 (sprite-sheet-2d)"));
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/background-layer-02 (sprite-sheet-2d)"));
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/background-layer-03 (sprite-sheet-2d)"));
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/background-layer-04 (sprite-sheet-2d)"));
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
            .any(|asset| asset == "playground-sidescroller/spritesheets/coin (sprite-sheet-2d)")
    );
    assert!(
        summary
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-sidescroller/spritesheets/finish (sprite-sheet-2d)")
    );
    assert!(summary.prepared_assets.iter().any(|asset| asset
        == "playground-sidescroller/spritesheets/platformer/tilesets/platform/base (tileset-2d)"));
    assert!(summary.prepared_assets.iter().any(|asset| {
        asset == "playground-sidescroller/spritesheets/platformer/rulesets/platform/rules (tile-ruleset-2d)"
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
