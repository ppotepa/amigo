use super::super::*;

#[test]
fn interactive_host_handler_applies_arrow_input_to_playground_3d_cube() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
            .with_startup_mod("playground-3d")
            .with_startup_scene("hello-world-cube")
            .with_dev_mode(true),
    )
    .expect("3d main playground bootstrap should succeed");

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let initial = scene
        .transform_of("playground-3d-cube")
        .expect("playground 3d cube should exist");

    let mut handler = InteractiveRuntimeHostHandler::new(
        amigo_session::RuntimeSession::from_runtime(
            runtime,
            amigo_session::RuntimeSessionProfile::Game,
        ),
        summary,
    )
    .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Right,
            pressed: true,
        })
        .expect("input event should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("physics update should advance spawned cube");

    let updated = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-3d-cube")
        .expect("playground 3d cube should exist after update");

    assert!(
        updated.rotation_euler.y > initial.rotation_euler.y,
        "Right arrow should rotate the 3D cube around the Y axis"
    );
}

#[test]
fn interactive_host_handler_selects_playground_npr_model_with_digit_keys() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-npr".to_owned()])
            .with_startup_mod("playground-npr")
            .with_startup_scene("comic-lines")
            .with_dev_mode(true),
    )
    .expect("npr comic lines playground bootstrap should succeed");

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    assert!(scene.is_visible("playground-npr-model-1-soldier"));
    assert!(!scene.is_visible("playground-npr-model-2-khronos-male"));

    let mut handler = InteractiveRuntimeHostHandler::new(
        amigo_session::RuntimeSession::from_runtime(
            runtime,
            amigo_session::RuntimeSessionProfile::Game,
        ),
        summary,
    )
    .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Digit2,
            pressed: true,
        })
        .expect("digit input event should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr selection update should run");

    let scene = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after update");
    assert!(!scene.is_visible("playground-npr-model-1-soldier"));
    assert!(scene.is_visible("playground-npr-model-2-khronos-male"));
}

#[test]
fn interactive_host_handler_toggles_playground_npr_strategy_without_replacing_preset() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-npr".to_owned()])
            .with_startup_mod("playground-npr")
            .with_startup_scene("comic-lines")
            .with_dev_mode(true),
    )
    .expect("npr comic lines playground bootstrap should succeed");

    let initial = runtime
        .resolve::<amigo_3d_mesh::MeshSceneService>()
        .expect("mesh scene service should exist")
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "playground-npr-model-1-soldier")
        .expect("soldier mesh command should exist")
        .mesh
        .npr
        .expect("soldier should have npr");
    assert_eq!(
        initial.render_strategy,
        amigo_render_api::NprRenderStrategy3d::GpuRealtime
    );

    let mut handler = InteractiveRuntimeHostHandler::new(
        amigo_session::RuntimeSession::from_runtime(
            runtime,
            amigo_session::RuntimeSessionProfile::Game,
        ),
        summary,
    )
    .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::G,
            pressed: true,
        })
        .expect("strategy toggle input event should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr strategy toggle update should run");
    process_placeholder_bridges(handler.session.runtime())
        .expect("queued npr strategy command should be applied");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::G,
            pressed: false,
        })
        .expect("strategy toggle key release should be accepted");
    handler
        .session
        .runtime()
        .resolve::<amigo_input_api::InputState>()
        .expect("input state should exist")
        .clear_frame_transients();
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr strategy key release update should run");

    let updated = handler
        .session
        .runtime()
        .resolve::<amigo_3d_mesh::MeshSceneService>()
        .expect("mesh scene service should exist after toggle")
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "playground-npr-model-1-soldier")
        .expect("soldier mesh command should still exist")
        .mesh
        .npr
        .expect("soldier should still have npr");
    assert_eq!(
        updated.render_strategy,
        amigo_render_api::NprRenderStrategy3d::CpuReference
    );
    let mut normalized = updated;
    normalized.render_strategy = initial.render_strategy;
    assert_eq!(
        normalized, initial,
        "G should only switch NPR backend strategy, not replace the active preset style"
    );
}

#[test]
fn interactive_host_handler_toggles_playground_npr_model_autorotate_with_r() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-npr".to_owned()])
            .with_startup_mod("playground-npr")
            .with_startup_scene("comic-lines")
            .with_dev_mode(true),
    )
    .expect("npr comic lines playground bootstrap should succeed");

    let initial = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-npr-model-1-soldier")
        .expect("soldier should exist");

    let mut handler = InteractiveRuntimeHostHandler::new(
        amigo_session::RuntimeSession::from_runtime(
            runtime,
            amigo_session::RuntimeSessionProfile::Game,
        ),
        summary,
    )
    .expect("interactive host handler should initialize");

    handler
        .session
        .runtime()
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist")
        .force_single_simulation_tick(0.25);
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr update without autorotate should run");
    let without_autorotate = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after update")
        .transform_of("playground-npr-model-1-soldier")
        .expect("soldier should exist after update");
    assert_eq!(
        without_autorotate.rotation_euler, initial.rotation_euler,
        "model should not rotate before R enables autorotate"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::R,
            pressed: true,
        })
        .expect("autorotate toggle input should be accepted");
    handler
        .session
        .runtime()
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist")
        .force_single_simulation_tick(0.25);
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr autorotate update should run");

    let rotated = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after autorotate")
        .transform_of("playground-npr-model-1-soldier")
        .expect("soldier should exist after autorotate");
    assert!(
        rotated.rotation_euler.y > without_autorotate.rotation_euler.y,
        "R should enable automatic model rotation"
    );
}

#[test]
fn interactive_host_handler_toggles_playground_npr_camera_freelook_with_f() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-npr".to_owned()])
            .with_startup_mod("playground-npr")
            .with_startup_scene("comic-lines")
            .with_dev_mode(true),
    )
    .expect("npr comic lines playground bootstrap should succeed");

    let initial_camera = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-npr-camera")
        .expect("npr camera should exist");

    let mut handler = InteractiveRuntimeHostHandler::new(
        amigo_session::RuntimeSession::from_runtime(
            runtime,
            amigo_session::RuntimeSessionProfile::Game,
        ),
        summary,
    )
    .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::MouseButton {
            button: amigo_input_api::MouseButton::Left,
            pressed: true,
        })
        .expect("orbit mouse press should be accepted");
    handler
        .on_input_event(InputEvent::CursorMoved { x: 120.0, y: 100.0 })
        .expect("orbit cursor start should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr orbit camera should record initial cursor");
    handler
        .on_input_event(InputEvent::CursorMoved { x: 184.0, y: 100.0 })
        .expect("orbit cursor drag should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr orbit camera update should run");
    handler
        .on_input_event(InputEvent::MouseButton {
            button: amigo_input_api::MouseButton::Left,
            pressed: false,
        })
        .expect("orbit mouse release should be accepted");

    let orbit_camera = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after orbit update")
        .transform_of("playground-npr-camera")
        .expect("npr camera should exist after orbit update");
    assert!(
        orbit_camera.rotation_euler.y < initial_camera.rotation_euler.y,
        "left mouse drag should orbit the NPR camera around the active model"
    );

    handler
        .on_input_event(InputEvent::ModifiersChanged(
            amigo_input_api::InputModifiers {
                shift: true,
                ..Default::default()
            },
        ))
        .expect("shift modifier should be accepted");
    handler
        .on_input_event(InputEvent::MouseButton {
            button: amigo_input_api::MouseButton::Left,
            pressed: true,
        })
        .expect("orbit pan mouse press should be accepted");
    handler
        .on_input_event(InputEvent::CursorMoved { x: 184.0, y: 100.0 })
        .expect("orbit pan cursor start should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr orbit pan should record initial cursor");
    handler
        .on_input_event(InputEvent::CursorMoved { x: 184.0, y: 142.0 })
        .expect("orbit pan cursor drag should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr orbit pan camera update should run");
    handler
        .on_input_event(InputEvent::MouseButton {
            button: amigo_input_api::MouseButton::Left,
            pressed: false,
        })
        .expect("orbit pan mouse release should be accepted");
    handler
        .on_input_event(InputEvent::ModifiersChanged(Default::default()))
        .expect("shift modifier release should be accepted");

    let panned_orbit_camera = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after orbit pan")
        .transform_of("playground-npr-camera")
        .expect("npr camera should exist after orbit pan");
    assert_eq!(
        panned_orbit_camera.rotation_euler, orbit_camera.rotation_euler,
        "Shift+LMB should pan the orbit target without rotating the camera"
    );
    assert_ne!(
        panned_orbit_camera.translation, orbit_camera.translation,
        "Shift+LMB should move the orbit camera framing"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::F,
            pressed: true,
        })
        .expect("camera toggle input event should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr camera update should run");

    let controller_service = handler
        .session
        .runtime()
        .resolve::<amigo_camera_core_plugin::CameraController3dSceneService>()
        .expect("camera controller service should exist after update");
    assert_eq!(
        controller_service.controllers()[0].mode,
        amigo_scene::CameraController3dModeSceneCommand::Freelook
    );
    handler
        .session
        .runtime()
        .resolve::<amigo_input_api::InputState>()
        .expect("input state should exist after camera toggle")
        .clear_frame_transients();
    handler
        .on_input_event(InputEvent::MouseWheel { delta_y: 120.0 })
        .expect("freelook wheel input should be accepted");
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr freelook wheel update should run");
    let speed_multiplier = handler
        .session
        .runtime()
        .resolve::<amigo_camera_core_plugin::CameraController3dSceneService>()
        .expect("camera controller service should exist after wheel")
        .controllers()[0]
        .freelook_speed_multiplier;
    assert!(
        speed_multiplier > 1.0 && speed_multiplier <= 1.2,
        "large wheel deltas should adjust freelook speed without a huge jump"
    );
    handler
        .session
        .runtime()
        .resolve::<amigo_input_api::InputState>()
        .expect("input state should exist after wheel")
        .clear_frame_transients();

    let before_fly = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist before freelook fly")
        .transform_of("playground-npr-camera")
        .expect("npr camera should exist before freelook fly");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::F,
            pressed: false,
        })
        .expect("camera toggle release should be accepted");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::W,
            pressed: true,
        })
        .expect("freelook forward input should be accepted");
    assert!(
        handler
            .session
            .runtime()
            .resolve::<amigo_input_api::InputState>()
            .expect("input state should exist before freelook fly")
            .is_down(KeyCode::W),
        "W should be down before freelook update"
    );
    handler
        .session
        .runtime()
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist")
        .force_single_simulation_tick(0.25);
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr freelook camera update should run");
    let state = handler
        .session
        .runtime()
        .resolve::<amigo_state::SceneStateService>()
        .expect("scene state service should exist after freelook fly");
    assert_eq!(
        state.get_string("npr_last_active_input_map").as_deref(),
        Some("playground-npr")
    );

    let after_fly = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after freelook fly")
        .transform_of("playground-npr-camera")
        .expect("npr camera should exist after freelook fly");
    assert_ne!(
        after_fly.translation, before_fly.translation,
        "W should move the NPR camera in freelook mode"
    );
    let pitch = -before_fly.rotation_euler.x;
    let yaw = before_fly.rotation_euler.y;
    let expected_forward = amigo_math::Vec3::new(
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        -yaw.cos() * pitch.cos(),
    );
    let forward_delta = amigo_math::Vec3::new(
        after_fly.translation.x - before_fly.translation.x,
        after_fly.translation.y - before_fly.translation.y,
        after_fly.translation.z - before_fly.translation.z,
    );
    assert!(
        dot3(forward_delta, expected_forward) > 0.0,
        "W should move the NPR camera along the direction it is looking"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::W,
            pressed: false,
        })
        .expect("freelook forward release should be accepted");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::S,
            pressed: true,
        })
        .expect("freelook backward input should be accepted");
    handler
        .session
        .runtime()
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist before backward fly")
        .force_single_simulation_tick(0.25);
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("npr freelook backward camera update should run");
    let after_back = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after backward fly")
        .transform_of("playground-npr-camera")
        .expect("npr camera should exist after backward fly");
    let backward_delta = amigo_math::Vec3::new(
        after_back.translation.x - after_fly.translation.x,
        after_back.translation.y - after_fly.translation.y,
        after_back.translation.z - after_fly.translation.z,
    );
    assert!(
        dot3(backward_delta, expected_forward) < 0.0,
        "S should move the NPR camera backward from the look direction"
    );
}

fn dot3(left: amigo_math::Vec3, right: amigo_math::Vec3) -> f32 {
    left.x * right.x + left.y * right.y + left.z * right.z
}

#[test]
fn interactive_host_handler_spawns_and_advances_physics_cubes_scene() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
            .with_startup_mod("playground-3d")
            .with_startup_scene("physics-cubes")
            .with_dev_mode(true),
    )
    .expect("3d physics playground bootstrap should succeed");

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    assert!(
        scene.transform_of("playground-3d-cube-spawner").is_some(),
        "authored physics spawner should exist at bootstrap"
    );

    let mut handler = InteractiveRuntimeHostHandler::new(
        amigo_session::RuntimeSession::from_runtime(
            runtime,
            amigo_session::RuntimeSessionProfile::Game,
        ),
        summary,
    )
    .expect("interactive host handler should initialize");

    handler
        .session
        .runtime()
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist")
        .force_single_simulation_tick(1.1);
    handler
        .session
        .run_phase(amigo_runtime::SystemPhase::Update)
        .expect("physics update should run");
    process_placeholder_bridges(handler.session.runtime())
        .expect("physics scene placeholder commands should dispatch");

    let spawned_initial = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after spawn")
        .transform_of("playground-3d-cube-0")
        .expect("physics spawner should create the first cube");

    handler
        .session
        .runtime()
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist")
        .force_single_simulation_tick(0.25);
    handler
        .on_redraw_requested()
        .expect("runtime tick should succeed");

    let scene = handler
        .session
        .runtime()
        .resolve::<SceneService>()
        .expect("scene service should exist after updates");
    let spawned_updated = scene
        .transform_of("playground-3d-cube-0")
        .expect("spawned cube should still exist after updates");

    assert!(
        spawned_updated.translation.y < spawned_initial.translation.y,
        "spawned cube should fall under gravity"
    );
    assert!(
        spawned_updated.rotation_euler != spawned_initial.rotation_euler,
        "spawned cube should rotate with authored angular velocity"
    );
}
