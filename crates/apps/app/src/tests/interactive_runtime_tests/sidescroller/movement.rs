use crate::tests::*;

#[test]
fn interactive_host_handler_advances_sidescroller_sprite_frames() {
    let (runtime, summary) = bootstrap_with_options(
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

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    let sprites = handler
        .runtime
        .resolve::<SpriteSceneService>()
        .expect("sprite scene service should exist");
    assert_eq!(sprites.frame_of("playground-sidescroller-coin-01"), Some(0));
    assert_eq!(sprites.frame_of("playground-sidescroller-player"), Some(0));

    for _ in 0..12 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");
    }

    let sprites = handler
        .runtime
        .resolve::<SpriteSceneService>()
        .expect("sprite scene service should exist");
    assert_ne!(
        sprites.frame_of("playground-sidescroller-coin-01"),
        Some(0),
        "coin should advance its spritesheet frame over time"
    );

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Right,
            pressed: true,
        })
        .expect("input event should be accepted");

    for _ in 0..2 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");
    }

    let sprites = handler
        .runtime
        .resolve::<SpriteSceneService>()
        .expect("sprite scene service should exist");
    assert!(
        matches!(
            sprites.frame_of("playground-sidescroller-player"),
            Some(1 | 2)
        ),
        "player should switch into run frames while moving right"
    );
}

#[test]
fn interactive_host_handler_applies_sidescroller_parallax() {
    let (runtime, summary) = bootstrap_with_options(
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

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let initial_camera = scene
        .transform_of("playground-sidescroller-camera")
        .expect("sidescroller camera should exist");
    let initial_layer_01 = scene
        .transform_of("playground-sidescroller-background-layer-01")
        .expect("background layer 01 should exist");
    let initial_layer_02 = scene
        .transform_of("playground-sidescroller-background-layer-02")
        .expect("background layer 02 should exist");
    let initial_layer_03 = scene
        .transform_of("playground-sidescroller-background-layer-03")
        .expect("background layer 03 should exist");
    let initial_layer_04 = scene
        .transform_of("playground-sidescroller-background-layer-04")
        .expect("background layer 04 should exist");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Right,
            pressed: true,
        })
        .expect("input event should be accepted");

    for _ in 0..12 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");
    }

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let updated_camera = scene
        .transform_of("playground-sidescroller-camera")
        .expect("sidescroller camera should exist after update");
    let updated_layer_01 = scene
        .transform_of("playground-sidescroller-background-layer-01")
        .expect("background layer 01 should exist after update");
    let updated_layer_02 = scene
        .transform_of("playground-sidescroller-background-layer-02")
        .expect("background layer 02 should exist after update");
    let updated_layer_03 = scene
        .transform_of("playground-sidescroller-background-layer-03")
        .expect("background layer 03 should exist after update");
    let updated_layer_04 = scene
        .transform_of("playground-sidescroller-background-layer-04")
        .expect("background layer 04 should exist after update");

    let layer_01_screen_delta = (updated_layer_01.translation.x - updated_camera.translation.x)
        - (initial_layer_01.translation.x - initial_camera.translation.x);
    let layer_02_screen_delta = (updated_layer_02.translation.x - updated_camera.translation.x)
        - (initial_layer_02.translation.x - initial_camera.translation.x);
    let layer_03_screen_delta = (updated_layer_03.translation.x - updated_camera.translation.x)
        - (initial_layer_03.translation.x - initial_camera.translation.x);
    let layer_04_screen_delta = (updated_layer_04.translation.x - updated_camera.translation.x)
        - (initial_layer_04.translation.x - initial_camera.translation.x);

    assert!(
        layer_01_screen_delta.abs() > 0.0,
        "background layer 01 should visibly shift on screen"
    );
    assert!(
        layer_02_screen_delta.abs() > layer_01_screen_delta.abs()
            && layer_03_screen_delta.abs() > layer_02_screen_delta.abs()
            && layer_04_screen_delta.abs() > layer_03_screen_delta.abs(),
        "closer background layers should move more on screen than farther ones"
    );
}

#[test]
fn interactive_host_handler_moves_sidescroller_camera_with_player() {
    let (runtime, summary) = bootstrap_with_options(
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

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let initial = scene
        .transform_of("playground-sidescroller-camera")
        .expect("sidescroller camera should exist");

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Right,
            pressed: true,
        })
        .expect("input event should be accepted");

    for _ in 0..8 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");
    }

    let updated = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-camera")
        .expect("sidescroller camera should exist after update");

    assert!(
        updated.translation.x > initial.translation.x,
        "camera follow should move the sidescroller camera to the right with the player"
    );
}

#[test]
fn interactive_host_handler_moves_sidescroller_player_right() {
    let (runtime, summary) = bootstrap_with_options(
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

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("initial runtime tick should succeed");

    let initial = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-player")
        .expect("sidescroller player should exist");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Right,
            pressed: true,
        })
        .expect("input event should be accepted");

    for _ in 0..8 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");
    }

    let updated = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-player")
        .expect("sidescroller player should exist after update");

    assert!(
        updated.translation.x > initial.translation.x,
        "Right arrow should move the sidescroller player to the right"
    );
}

#[test]
fn interactive_host_handler_player_jump_updates_hud_and_audio() {
    let (runtime, summary) = bootstrap_with_options(
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

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");

    for _ in 0..24 {
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime settle tick should succeed");
    }

    let before = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-player")
        .expect("player should exist");

    handler
        .on_input_event(InputEvent::Key {
            key: KeyCode::Space,
            pressed: true,
        })
        .expect("jump input should be accepted");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime jump tick should succeed");

    let after = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-player")
        .expect("player should exist after jump");
    assert!(
        after.translation.y > before.translation.y,
        "jump should move the player upward"
    );

    let ui_state = handler
        .runtime
        .resolve::<UiStateService>()
        .expect("ui state service should exist");
    assert_eq!(
        ui_state
            .text_override("playground-sidescroller-hud.root.message")
            .as_deref(),
        Some("JUMP")
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
                    if clip.as_str() == "playground-sidescroller/audio/jump"
            ))
    );
    let audio_mixer = handler
        .runtime
        .resolve::<AudioMixerService>()
        .expect("audio mixer service should exist");
    assert!(audio_mixer.frames().iter().any(|frame| {
        frame
            .sources
            .iter()
            .any(|source| source == "playground-sidescroller/audio/jump")
    }));
}

#[test]
fn interactive_host_handler_reaching_finish_updates_message_and_audio_state() {
    let (runtime, summary) = bootstrap_with_options(
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

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let finish = scene
        .transform_of("playground-sidescroller-finish")
        .expect("finish should exist");
    assert!(
        scene.set_transform("playground-sidescroller-player", finish),
        "player transform should be repositioned onto the finish trigger"
    );

    let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
        .expect("interactive host handler should initialize");
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime tick should succeed");

    let ui_state = handler
        .runtime
        .resolve::<UiStateService>()
        .expect("ui state service should exist");
    assert_eq!(
        ui_state
            .text_override("playground-sidescroller-hud.root.message")
            .as_deref(),
        Some("LEVEL COMPLETE")
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
                    if clip.as_str() == "playground-sidescroller/audio/level-complete"
            ))
    );
    assert!(
        audio_state
            .processed_commands()
            .iter()
            .any(|command| matches!(
                command,
                AudioCommand::StopSource { source } if source.as_str() == "proximity-beep"
            ))
    );
    assert!(
        audio_state
            .playing_sources()
            .iter()
            .all(|(source_id, _)| source_id != "proximity-beep"),
        "finish event should stop the realtime proximity source"
    );
    let audio_mixer = handler
        .runtime
        .resolve::<AudioMixerService>()
        .expect("audio mixer service should exist");
    assert!(audio_mixer.frames().iter().any(|frame| {
        frame
            .sources
            .iter()
            .any(|source| source == "playground-sidescroller/audio/level-complete")
    }));
    assert!(
        audio_mixer
            .active_realtime_sources()
            .iter()
            .all(|source| source != "proximity-beep")
    );
}
