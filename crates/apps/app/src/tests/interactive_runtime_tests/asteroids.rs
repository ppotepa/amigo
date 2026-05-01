use super::super::*;

#[test]
fn interactive_asteroids_options_low_mode_persists_into_game_scene() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-asteroids".to_owned(),
            ])
            .with_startup_mod("playground-2d-asteroids")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("asteroids bootstrap should succeed");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("initial runtime tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Down,
            pressed: true,
        })
        .expect("menu down input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("menu navigation tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Down,
            pressed: false,
        })
        .expect("menu down release should be accepted");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: true,
        })
        .expect("options select input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("options select tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: false,
        })
        .expect("options select release should be accepted");
    for _ in 0..3 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("options transition tick should succeed");
    }

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    assert_eq!(
        scene.selected_scene().map(|id| id.as_str().to_owned()),
        Some("options".to_owned())
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: true,
        })
        .expect("low toggle input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("low toggle tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: false,
        })
        .expect("low toggle release should be accepted");

    let session = handler
        .runtime
        .resolve::<amigo_state::SessionStateService>()
        .expect("session state service should exist");
    assert_eq!(session.get_bool("asteroids.low_mode"), Some(true));

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Escape,
            pressed: true,
        })
        .expect("options back input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("options back tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Escape,
            pressed: false,
        })
        .expect("options back release should be accepted");
    for _ in 0..3 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("main menu transition tick should succeed");
    }

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: true,
        })
        .expect("start input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("start tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: false,
        })
        .expect("start release should be accepted");
    for _ in 0..3 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("game transition tick should succeed");
    }

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    assert_eq!(
        scene.selected_scene().map(|id| id.as_str().to_owned()),
        Some("game".to_owned())
    );
    let pools = handler
        .runtime
        .resolve::<EntityPoolSceneService>()
        .expect("entity pool service should exist");
    assert_eq!(pools.active_count("asteroids"), 3);
}

#[test]
fn interactive_asteroids_sustained_thrust_moves_camera() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-asteroids".to_owned(),
            ])
            .with_startup_mod("playground-2d-asteroids")
            .with_startup_scene("game")
            .with_dev_mode(true),
    )
    .expect("asteroids game bootstrap should succeed");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("initial runtime tick should succeed");

    let camera_follow = handler
        .runtime
        .resolve::<amigo_scene::CameraFollow2dSceneService>()
        .expect("camera follow scene service should exist");
    assert!(
        camera_follow
            .follow("playground-2d-asteroids-arena-void")
            .is_some(),
        "endless Asteroids background should follow the camera"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Up,
            pressed: true,
        })
        .expect("thrust input should be accepted");

    for _ in 0..120 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime sustained thrust tick should succeed");
    }

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let camera = scene
        .transform_of("playground-2d-asteroids-camera")
        .expect("Asteroids camera should exist");
    assert!(
        camera.translation.y > 0.0,
        "endless Asteroids camera should follow the accelerating ship"
    );
}

#[test]
fn interactive_host_handler_updates_asteroids_ship_and_bullet_loop() {
    let (runtime, summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec![
                "core".to_owned(),
                "playground-2d-asteroids".to_owned(),
            ])
            .with_startup_mod("playground-2d-asteroids")
            .with_startup_scene("main-menu")
            .with_dev_mode(true),
    )
    .expect("asteroids bootstrap should succeed");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("initial runtime tick should succeed");

    {
        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        assert!(scene.is_visible("playground-2d-asteroids-main-menu"));
        let ui_state = handler
            .runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        assert!(ui_state.is_visible("playground-2d-asteroids-main-menu.root"));
    }

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: true,
        })
        .expect("menu start input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("start game tick should succeed");
    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: false,
        })
        .expect("menu start release should be accepted");
    for _ in 0..3 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("scene transition tick should succeed");
    }

    let initial_ship = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-2d-asteroids-ship")
        .expect("asteroids ship should exist");

    {
        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        assert_eq!(
            scene.selected_scene().map(|id| id.as_str().to_owned()),
            Some("game".to_owned())
        );
        assert!(scene.is_visible("playground-2d-asteroids-hud"));
        assert!(scene.is_visible("playground-2d-asteroids-ship"));
        assert!(scene.is_visible("playground-2d-asteroids-ship-shield"));
        assert!(scene.is_simulation_enabled("playground-2d-asteroids-ship"));
        let ui_state = handler
            .runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        assert!(ui_state.is_visible("playground-2d-asteroids-hud.root"));
    }

    let asteroid_to_hit = {
        let pools = handler
            .runtime
            .resolve::<EntityPoolSceneService>()
            .expect("entity pool scene service should exist");
        let active_asteroids = pools.active_members("asteroids");
        assert_eq!(active_asteroids.len(), 4);
        active_asteroids
            .first()
            .cloned()
            .expect("wave should spawn an asteroid")
    };
    handler
        .runtime
        .resolve::<ScriptEventQueue>()
        .expect("script event queue should exist")
        .publish(ScriptEvent::new(
            "asteroids.bullet_hit_asteroid",
            vec![
                "playground-2d-asteroids-bullet-01".to_owned(),
                asteroid_to_hit,
            ],
        ));
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("bullet hit event tick should succeed");
    let pools = handler
        .runtime
        .resolve::<EntityPoolSceneService>()
        .expect("entity pool scene service should exist");
    assert!(
        pools.active_count("asteroids") > 4,
        "hitting a wave asteroid should split it into smaller active fragments"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Up,
            pressed: true,
        })
        .expect("thrust input should be accepted");

    for _ in 0..6 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime thrust tick should succeed");
    }

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let updated_ship = scene
        .transform_of("playground-2d-asteroids-ship")
        .expect("asteroids ship should exist after thrust");
    assert!(
        updated_ship.translation.y > initial_ship.translation.y,
        "holding thrust should move the Asteroids ship forward"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: true,
        })
        .expect("fire input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime fire tick should succeed");

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let active_bullet = (1..=6)
        .map(|index| format!("playground-2d-asteroids-bullet-{index:02}"))
        .any(|entity| scene.is_visible(&entity) && scene.is_simulation_enabled(&entity));
    assert!(
        active_bullet,
        "firing should activate the first Asteroids bullet"
    );

    let audio_state = handler
        .runtime
        .resolve::<AudioStateService>()
        .expect("audio state service should exist");
    assert!(
        audio_state
            .processed_commands()
            .iter()
            .any(|command| matches!(
                command,
                AudioCommand::PlayOnce { clip }
                    if clip.as_str() == "playground-2d-asteroids/audio/shot"
            )),
        "firing should queue the Asteroids shot audio clip"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::R,
            pressed: true,
        })
        .expect("reload input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime reload tick should succeed");
    for _ in 0..4 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime post-reload tick should succeed");
    }

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    assert_eq!(
        scene.selected_scene().map(|id| id.as_str().to_owned()),
        Some("game".to_owned())
    );
}
