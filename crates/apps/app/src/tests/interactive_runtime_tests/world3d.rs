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

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Right,
            pressed: true,
        })
        .expect("input event should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime tick should succeed");

    let updated = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-3d-cube")
        .expect("playground 3d cube should exist after update");

    assert!(
        updated.rotation_euler.y > initial.rotation_euler.y,
        "Right arrow should rotate the 3D cube around the Y axis"
    );
}
