use crate::tests::*;

#[test]
fn interactive_host_handler_can_return_from_spritesheet_through_yaml_transition() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("hello-world-spritesheet")
            .with_dev_mode(true),
    )
    .expect("2d spritesheet playground bootstrap should succeed");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Down,
            pressed: true,
        })
        .expect("input event should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime tick should succeed");

    let updated_scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .selected_scene()
        .map(|scene| scene.as_str().to_owned());

    assert_eq!(
        updated_scene.as_deref(),
        Some("hello-world-square"),
        "ArrowDown on the spritesheet scene should emit a script event that triggers the YAML transition"
    );
}

#[test]
fn interactive_host_handler_can_switch_playground_2d_scenes_through_script_input() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("hello-world-square")
            .with_dev_mode(true),
    )
    .expect("2d square playground bootstrap should succeed");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Up,
            pressed: true,
        })
        .expect("input event should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime tick should succeed");

    let updated_scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .selected_scene()
        .map(|scene| scene.as_str().to_owned());

    assert_eq!(
        updated_scene.as_deref(),
        Some("hello-world-spritesheet"),
        "ArrowUp on the square scene should switch to the spritesheet scene through Rhai"
    );
}
