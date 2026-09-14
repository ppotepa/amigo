use super::super::*;

use amigo_npr_playground_plugin::playground::{NprPlaygroundIntent, NprPlaygroundService};
use std::{
    fs, thread,
    time::{Duration, Instant},
};

fn npr_edit(service: &NprPlaygroundService, intent: NprPlaygroundIntent) {
    service
        .dispatch_intent(
            1,
            service.domain_snapshot().revision,
            "render.fixture".into(),
            intent,
        )
        .unwrap();
}
fn pencil_fixture(service: &NprPlaygroundService) {
    let snapshot = service.domain_snapshot();
    let mut style = amigo_npr_playground_plugin::state::style_preset("Pencil Study").unwrap();
    style.paper = snapshot.settings.global.paper;
    style.light_direction = snapshot.settings.global.light_direction;
    npr_edit(
        service,
        NprPlaygroundIntent::SetLook {
            style,
            layers: snapshot.settings.style_layers,
        },
    );
    npr_edit(
        service,
        NprPlaygroundIntent::SetMotion {
            paused: true,
            speed: 1.,
            sketch_paused: false,
        },
    );
}

/// Runtime scenes publish camera-independent NPR meshes while Drawing Studio
/// fixtures continue to publish prepared drawing packets.
fn capture_ready_npr_frame(preview: &mut crate::ScenePreviewHost) -> crate::ScenePreviewFrame {
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut ready_before_frame = false;
    while Instant::now() < deadline {
        let frame = preview
            .capture_next_frame()
            .expect("NPR preview frame should render offscreen");
        let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
            preview.runtime().unwrap(),
        )
        .extract_all(preview.runtime().unwrap());
        let loading_ready = preview
            .runtime()
            .unwrap()
            .required::<amigo_session::RuntimeLoadingService>()
            .unwrap()
            .snapshot()
            .state
            == amigo_session::RuntimeLoadingState::Ready;
        if ready_before_frame && (!packet.npr().is_empty() || !packet.npr_meshes().is_empty()) {
            return frame;
        }
        ready_before_frame = loading_ready;
        thread::sleep(Duration::from_millis(10));
    }
    let loading = preview
        .runtime()
        .unwrap()
        .required::<amigo_session::RuntimeLoadingService>()
        .unwrap()
        .snapshot();
    let assets = preview
        .runtime()
        .unwrap()
        .required::<amigo_assets::AssetCatalog>()
        .unwrap();
    let mesh_service = preview
        .runtime()
        .unwrap()
        .required::<amigo_3d_mesh::MeshSceneService>()
        .unwrap();
    let meshes = mesh_service.commands();
    let geometries = meshes
        .iter()
        .filter(|command| {
            mesh_service
                .geometry_for(&command.mesh.mesh_asset)
                .is_some()
        })
        .count();
    let geometry_sizes = meshes
        .iter()
        .filter_map(|command| {
            mesh_service
                .geometry_for(&command.mesh.mesh_asset)
                .map(|geometry| (geometry.positions.len(), geometry.indices.len()))
        })
        .collect::<std::collections::BTreeSet<_>>();
    panic!(
        "NPR preparation did not publish a render contribution: state={:?}, stage={}, item={:?}, error={:?}, assets={:?}, failures={:?}, meshes={}, source_geometries={}, geometry_sizes={:?}",
        loading.state,
        loading.stage,
        loading.current_item,
        loading.error,
        assets.loading_summary(),
        assets.failed_assets(),
        meshes.len(),
        geometries,
        geometry_sizes,
    );
}

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
fn npr_city_resolves_loading_presentation_before_hydration() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "npr-city".to_owned()])
            .with_startup_mod("npr-city")
            .with_startup_scene("city")
            .with_dev_mode(true),
    )
    .expect("NPR city bootstrap should succeed");

    assert!(summary.failed_assets.is_empty());
    let presentation = runtime
        .required::<amigo_session::RuntimeLoadingService>()
        .unwrap()
        .snapshot()
        .presentation;
    assert_eq!(presentation.title, "Preparing city blocks");
    assert_eq!(presentation.accent.as_deref(), Some("#F6B44C"));
    assert_eq!(presentation.background.as_deref(), Some("#101827E8"));
}

#[test]
fn npr_city_publishes_a_complete_npr_packet_after_loading() {
    let options = crate::ScenePreviewOptions::new(mods_root(), "npr-city", "city", 512, 512)
        .with_active_mods(vec!["core".to_owned(), "npr-city".to_owned()])
        .with_warmup_frames(0)
        .with_playback_delta_seconds(1.0 / 30.0);
    let mut preview = crate::ScenePreviewHost::new(options);
    preview.warmup(1).unwrap();

    let first_frame = preview
        .capture_next_frame()
        .expect("NPR city should render its loading frame");
    let first_packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
        preview.runtime().unwrap(),
    )
    .extract_all(preview.runtime().unwrap());
    assert!(
        first_packet.world_3d_meshes().is_empty(),
        "NPR city must not expose standard Mesh3D content during preparation"
    );
    assert!(
        first_packet.world_3d_materials().is_empty(),
        "NPR city must not expose standard material content during preparation"
    );
    assert!(
        first_packet.world_3d_text().is_empty(),
        "NPR city must not expose standard text content during preparation"
    );
    assert!(
        !first_packet.game_ui_overlay().is_empty(),
        "NPR city must keep the engine loading overlay visible during preparation"
    );
    assert!(
        !first_frame.pixels_rgba8.is_empty(),
        "loading frame should be valid"
    );
    let loading_black_pixels = first_frame
        .pixels_rgba8
        .chunks_exact(4)
        .filter(|pixel| pixel[0] < 16 && pixel[1] < 16 && pixel[2] < 16)
        .count();
    assert!(
        loading_black_pixels > 512 * 512 * 3 / 4,
        "loading presentation must cover scene rendering with a black screen"
    );

    let frame = capture_ready_npr_frame(&mut preview);
    capture_npr_candidate("npr-city-ink", &frame.pixels_rgba8);
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
        preview.runtime().unwrap(),
    )
    .extract_all(preview.runtime().unwrap());
    assert!(
        packet.npr_meshes().len() >= 80,
        "NPR city should publish every model-space mesh every frame"
    );
    let style_for = |entity: &str| {
        packet
            .npr_meshes()
            .iter()
            .find(|command| command.mesh.entity_name == entity)
            .map(|command| command.style)
            .expect("authored NPR runtime entity should publish its style")
    };
    let hero_style = style_for("hero-officer");
    let support_style = style_for("support-officer");
    let civilian_style = style_for("civilian-01");
    assert!(hero_style.hatching_enabled && support_style.hatching_enabled);
    assert!(hero_style.join_strokes);
    assert!(!support_style.join_strokes);
    assert_ne!(hero_style.wobble_pixels, support_style.wobble_pixels);
    assert_ne!(hero_style.hatching_angle_degrees, civilian_style.hatching_angle_degrees);
    assert!(
        packet.npr().is_empty(),
        "animated NPR scenes must not retain camera-dependent drawing packets"
    );
    assert!(
        packet.world_3d_meshes().is_empty(),
        "NPR city must not also publish the standard Mesh3D presentation"
    );
    assert!(
        packet.world_3d_materials().is_empty(),
        "NPR city must not also publish the standard material presentation"
    );
    assert!(
        packet.world_3d_text().is_empty(),
        "NPR city must not also publish the standard text presentation"
    );
    assert_eq!(
        preview
            .runtime()
            .unwrap()
            .required::<amigo_session::RuntimeLoadingService>()
            .unwrap()
            .snapshot()
            .state,
        amigo_session::RuntimeLoadingState::Ready
    );
    // Pencil coverage is deliberately translucent. The previous near-black
    // threshold counted stacked, incorrectly unoccluded contours as success.
    // Require clearly contrasting graphite (at least 60 levels below paper),
    // while keeping the independent light-ground and animation checks below.
    let graphite_pixels = frame
        .pixels_rgba8
        .chunks_exact(4)
        .filter(|pixel| pixel[0] < 180 && pixel[1] < 180 && pixel[2] < 180)
        .count();
    let white_pixels = frame
        .pixels_rgba8
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 235 && pixel[1] > 235 && pixel[2] > 235)
        .count();
    assert!(
        graphite_pixels > 200,
        "Graphite contours must contrast with the paper in the completed frame"
    );
    assert!(
        white_pixels > 512 * 512 / 2,
        "Ink scene must have a light paper ground"
    );

    let next = preview
        .capture_next_frame()
        .expect("animated NPR city frame should render");
    let mut advanced = next;
    for _ in 0..4 {
        advanced = preview
            .capture_next_frame()
            .expect("animated NPR city frame should render");
    }
    assert_ne!(
        frame.pixels_rgba8, advanced.pixels_rgba8,
        "camera, sampled pose and line boiling must advance at the authored 8 FPS cadence"
    );

    let hero_positions = |preview: &crate::ScenePreviewHost| {
        let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
            preview.runtime().unwrap(),
        )
        .extract_all(preview.runtime().unwrap());
        packet
            .npr_meshes()
            .iter()
            .find(|command| command.mesh.entity_name == "hero-officer")
            .and_then(|command| command.mesh.mesh.geometry.as_ref())
            .map(|geometry| geometry.positions.clone())
            .expect("hero NPR mesh should expose sampled geometry")
    };
    let before_animation = hero_positions(&preview);
    for frame_index in 0..180 {
        let animation_frame = preview
            .capture_next_frame()
            .expect("animated NPR city frame should render");
        if [30, 90, 179].contains(&frame_index) {
            capture_npr_candidate(&format!("npr-city-animation-{frame_index:03}"), &animation_frame.pixels_rgba8);
        }
    }
    let after_animation = hero_positions(&preview);
    assert_ne!(before_animation, after_animation, "GLB animation must deform hero vertices over time");
}

#[test]
fn npr_playground_offscreen_matches_packet_contract() {
    let options = crate::ScenePreviewOptions::new(mods_root(), "npr-playground", "cube", 512, 512)
        .with_active_mods(vec!["core".to_owned(), "npr-playground".to_owned()])
        .with_warmup_frames(0)
        .with_playback_delta_seconds(1.0 / 60.0);
    let mut preview = crate::ScenePreviewHost::new(options);
    preview.warmup(1).unwrap();
    let service = preview
        .runtime()
        .unwrap()
        .required::<NprPlaygroundService>()
        .unwrap();
    npr_edit(
        &service,
        NprPlaygroundIntent::SetMotion {
            paused: true,
            speed: 1.,
            sketch_paused: false,
        },
    );
    let mut object = service.domain_snapshot().settings.objects["cube"].clone();
    object.rotation = [0.36_f32.to_degrees(), 0.71_f32.to_degrees(), 0.].into();
    npr_edit(
        &service,
        NprPlaygroundIntent::SetObjectPose {
            object: "cube".into(),
            position: object.position,
            rotation: object.rotation,
            scale: object.scale,
        },
    );
    npr_edit(&service, NprPlaygroundIntent::SetSeed { seed: 42 });
    let first = capture_ready_npr_frame(&mut preview);
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
        12_131_211_759_204_130_156
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
    let service = preview
        .runtime()
        .unwrap()
        .required::<NprPlaygroundService>()
        .unwrap();
    pencil_fixture(&service);
    let image = capture_ready_npr_frame(&mut preview);
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
    assert_eq!(command.packet.fingerprint().hash, 5_823_521_104_577_903_901);
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
    let service = preview
        .runtime()
        .unwrap()
        .required::<NprPlaygroundService>()
        .unwrap();
    let pose = preview
        .runtime()
        .unwrap()
        .required::<amigo_npr_playground_plugin::NprPlaygroundState>()
        .unwrap()
        .snapshot()
        .objects["cube"]
        .rotation;
    npr_edit(
        &service,
        NprPlaygroundIntent::SelectModel {
            model: "cylinder".into(),
        },
    );
    let mut object = service.domain_snapshot().settings.objects["cylinder"].clone();
    object.rotation = pose;
    npr_edit(
        &service,
        NprPlaygroundIntent::SetObject {
            object: "cylinder".into(),
            settings: object,
        },
    );
    npr_edit(
        &service,
        NprPlaygroundIntent::Navigate {
            mode: "focus".into(),
            dx: 0.,
            dy: 0.,
            wheel: 0.,
            x: 0.,
            y: 0.,
            width: 512,
            height: 512,
            focused: true,
        },
    );
    pencil_fixture(&service);
    let image = capture_ready_npr_frame(&mut preview);
    capture_npr_candidate("pencil-cylinder-512-candidate", &image.pixels_rgba8);
    verify_reviewed_npr_image("pencil-cylinder-512", &image.pixels_rgba8);
    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(
        preview.runtime().unwrap(),
    )
    .extract_all(preview.runtime().unwrap());
    let command = &packet.npr()[0];
    assert_eq!(service.domain_snapshot().settings.objects.len(), 1);
    assert_eq!(service.domain_snapshot().settings.selected, "cylinder");
    assert!(!command.packet.occluders.is_empty());
    assert!(command.packet.fills.is_empty());
    assert!(command.packet.stats.hatching_strokes > 0);
    assert_eq!(command.packet.fingerprint().hash, 8_103_697_224_930_990_954);
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
