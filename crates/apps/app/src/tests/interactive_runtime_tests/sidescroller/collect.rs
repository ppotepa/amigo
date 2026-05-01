use crate::tests::*;

#[test]
fn interactive_host_handler_collects_sidescroller_coin_and_updates_hud() {
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
    let coin = scene
        .transform_of("playground-sidescroller-coin-01")
        .expect("coin should exist");
    assert!(
        scene.set_transform("playground-sidescroller-player", coin),
        "player transform should be repositioned onto the coin"
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
            .text_override("playground-sidescroller-hud.root.coins")
            .as_deref(),
        Some("Coins: 1 / 25")
    );
    assert_eq!(
        ui_state
            .text_override("playground-sidescroller-hud.root.message")
            .as_deref(),
        Some("COIN COLLECTED")
    );
    let moved_coin = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist")
        .transform_of("playground-sidescroller-coin-01")
        .expect("coin should still exist after collection");
    assert!(
        moved_coin.translation.x <= -10_000.0 && moved_coin.translation.y <= -10_000.0,
        "collected coin should be moved out of the playable space"
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
                    if clip.as_str() == "playground-sidescroller/audio/coin"
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
            .any(|source| source == "playground-sidescroller/audio/coin")
    }));

    let scene = handler
        .runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let mut reset_transform = coin;
    reset_transform.translation.x = 0.0;
    reset_transform.translation.y = 0.0;
    assert!(
        scene.set_transform("playground-sidescroller-player", reset_transform),
        "player should be moved away from the collected coin"
    );
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime tick after moving away should succeed");
    assert!(
        scene.set_transform("playground-sidescroller-player", coin),
        "player should be moved back to the original coin position"
    );
    handler
        .on_lifecycle(HostLifecycleEvent::AboutToWait)
        .expect("runtime tick after returning should succeed");

    assert_eq!(
        ui_state
            .text_override("playground-sidescroller-hud.root.coins")
            .as_deref(),
        Some("Coins: 1 / 25")
    );
    let coin_play_count = audio_state
        .processed_commands()
        .iter()
        .filter(|command| {
            matches!(
                command,
                AudioCommand::PlayOnce { clip }
                    if clip.as_str() == "playground-sidescroller/audio/coin"
            )
        })
        .count();
    assert_eq!(
        coin_play_count, 1,
        "collected coin should not replay when revisiting the original location"
    );
}
