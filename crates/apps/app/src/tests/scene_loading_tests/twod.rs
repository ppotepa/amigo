use super::super::*;
use amigo_camera_core_plugin::{CameraId, CameraService};
use amigo_layered_image_2d_plugin::LayeredImageSceneService;
use amigo_particles_2d_plugin::Particle2dSceneService;
use amigo_render_api::VisualSourceKind2d;
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
        .find(|command| command.entity_name == "rotten-club-background")
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
    assert_eq!(background.image.size, amigo_math::Vec2::new(1672.0, 941.0));
    assert_eq!(background.image.base_opacity, 1.0);
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
fn rotten_club_main_menu_script_keeps_authored_intro_state() {
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
        "authored intro state should start with hidden rain"
    );
    assert!(particles.is_active("rotten-club-rain-near"));
    assert!(particles.is_active("rotten-club-rain-far"));

    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    let commands = layered_images.commands();
    let command = commands
        .iter()
        .find(|command| command.entity_name == "rotten-club-background")
        .expect("background layered image command should exist");
    assert_eq!(command.image.base_opacity, 1.0);
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
            .any(|override_| { override_.id == "club_sign" && override_.opacity == Some(0.95) })
    );

    script_runtime
        .call_update("scene:rotten-club:main-menu", 6.0)
        .expect("main menu script update should run");
    process_placeholder_bridges(&runtime).expect("menu reveal commands should dispatch");

    let command = layered_images
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "rotten-club-background")
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
            .is_some_and(|rain| rain <= f64::EPSILON),
        "the scene script update should not advance the timeline clock by itself"
    );
    assert!(particles.is_active("rotten-club-rain-near"));
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
    let scene_state = runtime
        .resolve::<SceneStateService>()
        .expect("scene state service should be registered");
    let particles = runtime
        .resolve::<Particle2dSceneService>()
        .expect("particle scene service should be registered");
    let layered_images = runtime
        .resolve::<LayeredImageSceneService>()
        .expect("layered image scene service should be registered");
    let render_layers = runtime
        .resolve::<amigo_2d_composition::RenderLayer2dSceneService>()
        .expect("render layer scene service should be registered");
    let light_groups = runtime
        .resolve::<amigo_light_2d_plugin::LightGroup2dSceneService>()
        .expect("light group scene service should be registered");
    let cameras = runtime
        .resolve::<CameraService>()
        .expect("camera service should be registered");

    process_placeholder_bridges(&runtime).expect("scene commands should dispatch");
    amigo_runtime_bundles::tick_timeline_2d_world(&runtime, 1.0)
        .expect("timeline should tick through the first second");
    assert!(
        scene_state
            .get_float("intro.rain_intensity")
            .is_some_and(|rain| rain <= f64::EPSILON),
        "Rotten Club rain should stay hidden during the first second"
    );
    assert_eq!(particles.intensity("rotten-club-rain-near"), 0.0);
    assert!(
        render_layers
            .commands()
            .iter()
            .any(|layer| layer.id == "title.depth2d" && layer.opacity <= f32::EPSILON),
        "title render layer should start fully transparent"
    );

    amigo_runtime_bundles::tick_timeline_2d_world(&runtime, 11.0)
        .expect("timeline should tick through the weather reveal");
    assert!(
        scene_state
            .get_float("intro.rain_intensity")
            .is_some_and(|rain| rain > 0.0),
        "rain timing state should advance after the weather pass begins"
    );
    assert!(
        particles.intensity("rotten-club-rain-near") > 0.0,
        "near rain emitter should be driven by the timeline"
    );
    assert!(particles.is_active("rotten-club-rain-near"));
    assert!(
        scene_state
            .get_float("intro.camera_focus_distance_m")
            .is_some_and(|focus| focus <= 9.1),
        "camera focus timing state should settle onto the club sign after the club focus beat"
    );
    assert!(
        cameras
            .get_2d(&CameraId::new("main"))
            .is_some_and(|camera| camera.aperture.focus_distance_m <= 9.1),
        "camera focus beat should drive CameraService, not only scene state"
    );

    let command = layered_images
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "rotten-club-background")
        .expect("Rotten Club background layered image command should exist");
    assert!(
        command
            .image
            .layer_overrides
            .iter()
            .any(|override_| override_.id == "club_sign" && override_.opacity.unwrap_or(0.0) > 0.0),
        "club sign layer should be visible once the club lighting pass has started"
    );

    amigo_runtime_bundles::tick_timeline_2d_world(&runtime, 2.25)
        .expect("timeline should tick into the lightning beat");
    assert!(
        scene_state
            .get_float("intro.lightning_intensity")
            .is_some_and(|flash| flash > 0.0),
        "lightning beat should produce a visible flash during the strike window"
    );
    assert!(
        light_groups
            .commands()
            .iter()
            .any(|group| group.id == "lightning" && group.intensity > 0.0),
        "lightning beat should drive the explicit light group intensity"
    );

    amigo_runtime_bundles::tick_timeline_2d_world(&runtime, 3.25)
        .expect("timeline should tick into the title reveal");
    assert!(
        scene_state
            .get_float("intro.title_alpha")
            .is_some_and(|alpha| alpha > 0.0),
        "title alpha should ramp after the title reveal starts"
    );
    assert!(
        render_layers
            .commands()
            .iter()
            .any(|layer| layer.id == "title.depth2d" && layer.opacity > 0.0),
        "title render layer opacity should ramp after the title reveal starts"
    );

    let session = amigo_session::RuntimeSession::from_runtime(
        runtime,
        amigo_session::RuntimeSessionProfile::Game,
    );
    let render_packet = amigo_runtime_bundles::extract_game_frame_packet(&session, false)
        .expect("Rotten Club render packet extraction should succeed");
    assert!(
        render_packet
            .render_light_groups_2d()
            .any(|group| group.id == "neon.mid"),
        "declared light groups should be extracted into the render packet"
    );
    assert!(
        render_packet
            .world_2d_light_sources()
            .iter()
            .any(|source| source.owner == "neon.mid"),
        "light group extraction should expose neon.mid as a camera-optical light source"
    );
    assert!(
        particles
            .emitter("rotten-club-rain-near")
            .is_some_and(|emitter| {
                emitter.emitter.material.lighting_mode
                    == amigo_light_2d_plugin::Material2dLightingMode::LightGroupSampled
                    && emitter
                        .emitter
                        .material
                        .light_receiver
                        .is_some_and(|receiver| {
                            receiver.groups.iter().any(|group| group == "neon.mid")
                        })
            }),
        "Rotten Club rain should receive light through light-group sampled particle material"
    );

    let camera_candidates = amigo_runtime_bundles::render_extractor_bridges::
        collect_camera_optical_candidates_from_light_sources_2d(
            render_packet.world_2d_light_sources(),
        );
    assert!(
        camera_candidates.iter().any(|candidate| {
            candidate.owner == "neon.mid"
                && candidate.is_active()
                && candidate.targets_scene_highlight()
                && candidate.targets_scene_emissive()
        }),
        "neon.mid should become an active camera optical candidate for highlight and emissive buffers"
    );
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

    assert_eq!(yaml_str(&timeline, "kind"), Some("timeline-2d"));
    assert_eq!(yaml_str(&timeline, "id"), Some("main-menu-intro"));
    assert_eq!(
        yaml_child(&timeline, "clock").and_then(|clock| yaml_f64(clock, "complete_at_s")),
        Some(19.0)
    );

    for beat_id in [
        "skyline",
        "club",
        "club-focus",
        "rain",
        "lightning",
        "title-focus",
        "title",
    ] {
        assert!(
            yaml_sequence(&timeline, "beats")
                .iter()
                .any(|beat| yaml_str(beat, "id") == Some(beat_id)),
            "timeline should declare beat `{beat_id}`"
        );
    }

    for track in yaml_sequence(&timeline, "tracks") {
        assert!(
            yaml_str(track, "control_path").is_some() || yaml_str(track, "state_key").is_some(),
            "track `{}` should declare control_path or state_key",
            yaml_str(track, "id").unwrap_or("<missing-id>")
        );
        assert!(
            !yaml_sequence(track, "curve").is_empty() || yaml_child(track, "pulse").is_some(),
            "track `{}` should declare curve or pulse",
            yaml_str(track, "id").unwrap_or("<missing-id>")
        );
    }

    for track_id in [
        "background.club_sign.opacity",
        "rain.near.intensity",
        "camera.focus_distance_m",
        "title.opacity",
    ] {
        assert!(
            yaml_sequence(&timeline, "tracks")
                .iter()
                .any(|track| yaml_str(track, "id") == Some(track_id)),
            "timeline should declare track `{track_id}`"
        );
    }

    let camera_track = yaml_sequence(&timeline, "tracks")
        .into_iter()
        .find(|track| yaml_str(track, "id") == Some("camera.focus_distance_m"))
        .expect("camera focus track should exist");
    assert_eq!(
        yaml_str(camera_track, "control_path"),
        Some("world.camera.main.Camera2D.aperture.focus_distance_m")
    );
    assert!(
        yaml_sequence(camera_track, "curve")
            .iter()
            .any(|keyframe| yaml_f64(keyframe, "t") == Some(15.0)
                && yaml_f64(keyframe, "value") == Some(9.0)),
        "camera focus track should hold club focus before title focus"
    );

    let title_opacity_track = yaml_sequence(&timeline, "tracks")
        .into_iter()
        .find(|track| yaml_str(track, "id") == Some("title.opacity"))
        .expect("title opacity track should exist");
    assert_eq!(
        yaml_str(title_opacity_track, "control_path"),
        Some("world.title.depth2d.RenderLayer2D.opacity")
    );
}

fn yaml_child<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
}

fn yaml_sequence<'a>(value: &'a serde_yaml::Value, key: &str) -> Vec<&'a serde_yaml::Value> {
    yaml_child(value, key)
        .and_then(serde_yaml::Value::as_sequence)
        .map(|sequence| sequence.iter().collect())
        .unwrap_or_default()
}

fn yaml_str<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
    yaml_child(value, key).and_then(serde_yaml::Value::as_str)
}

fn yaml_f64(value: &serde_yaml::Value, key: &str) -> Option<f64> {
    yaml_child(value, key).and_then(serde_yaml::Value::as_f64)
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
        emitter_names
            .iter()
            .any(|name| name == "rotten-club-rain-near"),
        "rain emitters should be hydrated, got {emitter_names:?}"
    );
    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should be registered");
    assert!(scene.is_visible("rotten-club-rain-near"));
    assert!(scene.is_simulation_enabled("rotten-club-rain-near"));
    assert!(
        particles.intensity("rotten-club-rain-near") > 0.0,
        "intro rain should drive the nearest world-space emitter"
    );
    assert!(particles.is_active("rotten-club-rain-near"));
    assert!(
        particles
            .effective_max_particles("rotten-club-rain-near")
            .unwrap_or(0)
            > 0,
        "rain emitter should have a positive particle budget"
    );
    assert!(
        particles
            .effective_spawn_rate("rotten-club-rain-near")
            .unwrap_or(0.0)
            > 0.0,
        "rain emitter should have a positive spawn rate"
    );
    assert!(
        particles.particle_count("rotten-club-rain-near") > 0,
        "active rain should spawn near-camera particles before extraction"
    );
    assert!(
        particles.particle_count("rotten-club-rain-far") > 0,
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
    assert_eq!(focus_blur.depth_map.as_deref(), Some("main-depth"));
    assert!(focus_blur.max_blur_px >= 14.0);
    assert!(focus_blur.background_blur_boost >= 1.0);
    assert!(focus_blur.highlight_threshold <= 0.58);
    assert!(focus_blur.highlight_gain >= 1.2);
    assert!(
        packet.render_depth_maps_2d().any(|depth_map| {
            depth_map.id == "main-depth"
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
            .any(|layer| layer.layer_id == "weather.rain.far")
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
            item.owner_entity() == "rotten-club-rain-far"
                && item.component_kind() == "ParticleEmitter2D"
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
    assert!(bridge.processed_scene_commands.iter().any(|command| {
        command.starts_with("scene.plugin(amigo.gfx.text-2d.scene-command.Text2D)")
    }));
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
    assert!(summary.processed_scene_commands.iter().any(|command| {
        command.starts_with("scene.plugin(amigo.gfx.sprite-2d.scene-command.Sprite2D)")
    }));
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
    assert!(summary.processed_scene_commands.iter().any(|command| {
        command.starts_with("scene.plugin(amigo.gfx.text-2d.scene-command.Text2D)")
    }));
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
