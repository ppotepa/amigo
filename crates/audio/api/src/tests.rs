use crate::{
    AudioBus, AudioClip, AudioClipKey, AudioCommand, AudioCommandQueue, AudioCue, AudioPlaybackMode,
    AudioSceneService, AudioSourceId, AudioStateService,
};

#[test]
fn stores_audio_commands_and_state() {
    let queue = AudioCommandQueue::default();
    let scene = AudioSceneService::default();
    let state = AudioStateService::default();

    assert_eq!(state.master_volume(), 1.0);

    scene.register_clip(AudioClip {
        key: AudioClipKey::new("playground-sidescroller/audio/jump"),
        mode: AudioPlaybackMode::OneShot,
    });
    assert!(scene.register_cue(AudioCue::new(
        "jump",
        AudioClipKey::new("playground-sidescroller/audio/jump"),
        Some(0.1),
    )));
    queue.push(AudioCommand::PlayOnce {
        clip: AudioClipKey::new("playground-sidescroller/audio/jump"),
    });
    state.start_source(
        AudioSourceId::new("proximity-beep"),
        AudioClipKey::new("playground-sidescroller/audio/proximity-beep"),
    );
    state.set_param("proximity-beep", "distance", 128.0);
    state.set_volume("music", 0.5);
    state.set_master_volume(0.75);
    state.record_processed_command(AudioCommand::PlayOnce {
        clip: AudioClipKey::new("playground-sidescroller/audio/jump"),
    });
    state.queue_runtime_command(AudioCommand::PlayOnce {
        clip: AudioClipKey::new("playground-sidescroller/audio/jump"),
    });

    assert_eq!(scene.clips().len(), 1);
    assert_eq!(
        scene.cue("jump").map(|cue| cue.clip.as_str().to_owned()),
        Some("playground-sidescroller/audio/jump".to_owned())
    );
    assert!(scene.mark_cue_played_if_ready(&scene.cue("jump").expect("cue exists")));
    assert!(!scene.mark_cue_played_if_ready(&scene.cue("jump").expect("cue exists")));
    assert_eq!(queue.snapshot().len(), 1);
    assert!(state.playing_sources().contains_key("proximity-beep"));
    assert_eq!(
        state
            .source_params()
            .get("proximity-beep")
            .and_then(|params| params.get("distance"))
            .copied(),
        Some(128.0)
    );
    assert_eq!(state.bus_volumes().get("music").copied(), Some(0.5));
    assert_eq!(state.master_volume(), 0.75);
    assert_eq!(state.processed_commands().len(), 1);
    assert_eq!(state.drain_runtime_commands().len(), 1);
    assert!(state.stop_source("proximity-beep"));

    assert_eq!(AudioBus::new("music").id, "music");
}

