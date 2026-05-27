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
