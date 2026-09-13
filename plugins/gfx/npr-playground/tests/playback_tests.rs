use amigo_npr_playground_plugin::{
    NprPlaygroundRenderService, NprPlaygroundState,
    playback::{PlaybackCommand as Command, PlaybackSource as Source},
    playground::{NprPlaygroundIntent as Intent, NprPlaygroundService},
};
use amigo_playground_api::{PlaygroundEvent, PlaygroundHostService, PlaygroundProvider};
use serde_json::json;
use std::{fs, sync::Arc};

fn fixture() -> (
    tempfile::TempDir,
    Arc<NprPlaygroundState>,
    Arc<NprPlaygroundService>,
    Arc<NprPlaygroundRenderService>,
) {
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = Vec::new();
    for value in [
        -1.0f32, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    fs::write(dir.path().join("mesh.bin"), &bytes).unwrap();
    let doc = json!({"asset":{"version":"2.0"},"buffers":[{"uri":"mesh.bin","byteLength":bytes.len()}],
      "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},{"buffer":0,"byteOffset":36,"byteLength":8},{"buffer":0,"byteOffset":44,"byteLength":24}],
      "accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1,-1,0],"max":[1,1,0]},
        {"bufferView":1,"componentType":5126,"count":2,"type":"SCALAR","min":[0],"max":[1]},
        {"bufferView":2,"componentType":5126,"count":2,"type":"VEC3"}],
      "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],"nodes":[{"name":"Moving triangle","mesh":0}],"scenes":[{"nodes":[0]}],"scene":0,
      "animations":[{"name":"Slide","samplers":[{"input":1,"output":2,"interpolation":"LINEAR"}],"channels":[{"sampler":0,"target":{"node":0,"path":"translation"}}]}]});
    let path = dir.path().join("model.gltf");
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
    let state = Arc::new(NprPlaygroundState::default());
    state.settings.lock().unwrap().sketch_paused = true;
    state
        .settings
        .lock()
        .unwrap()
        .objects
        .get_mut("cube")
        .unwrap()
        .model = "animated".into();
    let render = Arc::new(NprPlaygroundRenderService::default());
    render.load_model("animated", &path).unwrap();
    let service = Arc::new(NprPlaygroundService::new(state.clone()));
    service.attach_runtime(
        Arc::new(amigo_assets::AssetCatalog::default()),
        render.clone(),
    );
    (dir, state, service, render)
}
fn send(service: &NprPlaygroundService, intent: Intent) {
    service
        .dispatch_intent(1, service.revision(), "test".into(), intent)
        .unwrap();
}
fn playback(service: &NprPlaygroundService, command: Command) {
    send(service, Intent::Playback { command });
}

#[test]
fn playback_is_transient_preserves_live_preview_and_publishes_same_revision_time() {
    let (_dir, state, service, _render) = fixture();
    let authored = service.domain_snapshot().settings;
    let mut layer = authored.style_layers.layer("contours").unwrap().clone();
    layer.opacity = 0.3;
    send(
        &service,
        Intent::PreviewLayer {
            layer: layer.id.clone(),
            value: layer.clone(),
        },
    );
    playback(
        &service,
        Command::Source {
            source: Source::Clip { index: 0 },
        },
    );
    playback(&service, Command::Playing { playing: true });
    assert!(state.snapshot().needs_temporal_frames());
    let revision = service.revision();
    let host = PlaygroundHostService::default();
    host.register(service.clone()).unwrap();
    let id = service.descriptor().id;
    host.connect(&id).unwrap();
    let revision = service.revision().max(revision); // Catalog discovery may complete at handshake.
    state.tick(0.25);
    assert_eq!(service.revision(), revision);
    let events = host.poll(&id).unwrap();
    assert!(events.iter().any(|e|matches!(e,PlaygroundEvent::Delta {base_revision,revision:current,changed,..} if *base_revision==revision && *current==revision && changed.get("playback").is_some_and(|v|v["time_seconds"]==json!(0.25)))));
    assert_eq!(service.domain_snapshot().settings, authored);
    assert!(!service.domain_snapshot().dirty);
    assert!(!service.domain_snapshot().can_undo);
    assert_eq!(
        state
            .snapshot()
            .style_layers
            .layer("contours")
            .unwrap()
            .opacity,
        0.3
    );
    assert_eq!(
        service.domain_snapshot().preview_layer.as_deref(),
        Some("contours")
    );
    assert!(
        !serde_json::to_string(&state.snapshot())
            .unwrap()
            .contains("playback")
    );
    let snapshot = service.snapshot();
    assert_eq!(
        snapshot.values["animation_clips"][0]["tracks"][0]["node_path"],
        "Moving triangle"
    );
    assert!(
        snapshot.values["animation_clips"][0]["tracks"][0]
            .get("times")
            .is_none()
    );
}

#[test]
fn clip_pause_seek_loop_end_restart_and_edits_preserve_transport() {
    let (_dir, state, service, render) = fixture();
    playback(
        &service,
        Command::Source {
            source: Source::Clip { index: 0 },
        },
    );
    render
        .rebuild(&state.render_snapshot(), [256, 256])
        .unwrap();
    let original = render.commands()[0].packet.fingerprint();
    assert!(
        render
            .pick_surface(&state.snapshot(), [256, 256], glam::Vec2::splat(128.0))
            .is_some()
    );
    let builds = render.stats()["packet_builds"];
    playback(&service, Command::Playing { playing: true });
    state.tick(0.5);
    render
        .rebuild(&state.render_snapshot(), [256, 256])
        .unwrap();
    assert_ne!(render.commands()[0].packet.fingerprint(), original);
    assert_eq!(render.stats()["packet_builds"], builds + 1);
    assert!(
        render
            .pick_surface(&state.snapshot(), [256, 256], glam::Vec2::splat(128.0))
            .is_none()
    );
    playback(&service, Command::Playing { playing: false });
    state.tick(0.3);
    assert_eq!(state.snapshot().playback.unwrap().time_seconds, 0.5);
    let camera = state.snapshot();
    send(
        &service,
        Intent::SetCamera {
            target: camera.camera_target,
            yaw: 20.0,
            pitch: 10.0,
            distance: camera.camera_distance,
            fov: camera.camera_fov,
        },
    );
    assert_eq!(state.snapshot().playback.unwrap().time_seconds, 0.5);
    send(&service, Intent::Undo);
    assert_eq!(state.snapshot().playback.unwrap().time_seconds, 0.5);
    playback(&service, Command::Seek { seconds: 0.9 });
    playback(&service, Command::Playing { playing: true });
    state.tick(0.3);
    assert!((state.snapshot().playback.unwrap().time_seconds - 0.2).abs() < 1e-5);
    playback(
        &service,
        Command::Options {
            looping: false,
            speed: 2.0,
        },
    );
    state.tick(1.0);
    let p = state.snapshot().playback.unwrap();
    assert_eq!(p.time_seconds, 1.0);
    assert!(!p.playing);
    playback(&service, Command::Playing { playing: true });
    assert_eq!(state.snapshot().playback.unwrap().time_seconds, 0.0);
    let before = state.snapshot();
    for command in [
        Command::Seek { seconds: 2.0 },
        Command::Options {
            looping: true,
            speed: 0.0,
        },
        Command::Source {
            source: Source::Clip { index: 99 },
        },
    ] {
        assert!(
            service
                .dispatch_intent(
                    1,
                    service.revision(),
                    "invalid".into(),
                    Intent::Playback { command }
                )
                .is_err()
        );
        assert_eq!(state.snapshot(), before);
    }
}

#[test]
fn top_play_turntable_does_not_dirty_document_and_reset_is_immediate() {
    let (_dir, state, service, _) = fixture();
    let original = state.snapshot().objects["cube"].rotation;
    playback(&service, Command::Playing { playing: true });
    state.tick(0.5);
    assert_ne!(state.snapshot().objects["cube"].rotation, original);
    assert!(!service.domain_snapshot().dirty);
    assert!(!service.domain_snapshot().can_undo);
    playback(&service, Command::Playing { playing: false });
    let rotation = state.snapshot().objects["cube"].rotation;
    state.tick(0.5);
    assert_eq!(state.snapshot().objects["cube"].rotation, rotation);
    playback(&service, Command::Seek { seconds: 0.0 });
    assert_eq!(state.snapshot().objects["cube"].rotation, original);
    assert!(!state.snapshot().needs_temporal_frames());
}

#[test]
fn model_switch_discards_transport_and_keeps_the_preset() {
    let state = Arc::new(NprPlaygroundState::default());
    let service = NprPlaygroundService::new(state.clone());
    let layers = service.domain_snapshot().settings.style_layers;
    playback(&service, Command::Playing { playing: true });
    state.tick(0.5);
    send(
        &service,
        Intent::SelectModel {
            model: "sphere".into(),
        },
    );
    assert!(state.snapshot().playback.is_none());
    assert_eq!(state.snapshot().objects.len(), 1);
    assert_eq!(state.snapshot().selected, "sphere");
    assert_eq!(state.snapshot().style_layers, layers);
    assert!(!state.snapshot().objects["sphere"].rotating);
}
