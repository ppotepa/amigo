use amigo_playground_api::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Default)]
struct Provider(Mutex<(u64, bool)>);
impl PlaygroundProvider for Provider {
    fn descriptor(&self) -> PlaygroundDescriptor {
        PlaygroundDescriptor {
            id: PlaygroundId("test".into()),
            label: "Test".into(),
            client: "test".into(),
        }
    }
    fn snapshot(&self) -> PlaygroundSnapshot {
        let state = self.0.lock().unwrap();
        PlaygroundSnapshot {
            revision: state.0,
            values: BTreeMap::from([("paused".into(), json!(state.1))]),
            metadata: Value::Null,
        }
    }
    fn dispatch(&self, action: &PlaygroundActionEnvelope) -> Result<(), PlaygroundActionError> {
        let Some(value) = action.intent.as_bool() else {
            return Err(PlaygroundActionError {
                control: action.control.clone(),
                code: "invalid_value".into(),
                message: "Expected boolean".into(),
            });
        };
        let mut state = self.0.lock().unwrap();
        state.0 += 1;
        state.1 = value;
        Ok(())
    }
}

#[test]
fn revisions_reject_stale_edits_and_deltas_only_include_changes() {
    let host = PlaygroundHostService::default();
    host.register(Arc::new(Provider::default())).unwrap();
    assert!(host.register(Arc::new(Provider::default())).is_err());
    let id = PlaygroundId("test".into());
    assert!(matches!(
        host.connect(&id).unwrap(),
        PlaygroundEvent::Snapshot { .. }
    ));
    let action = PlaygroundActionEnvelope {
        request_id: 1,
        base_revision: 0,
        control: "pause".into(),
        intent: json!(true),
    };
    assert!(matches!(
        host.dispatch(&id, action.clone()),
        PlaygroundEvent::Accepted { revision: 1, .. }
    ));
    assert!(matches!(
        host.dispatch(&id, action.clone()),
        PlaygroundEvent::Rejected { .. }
    ));
    assert!(
        matches!(host.dispatch(&id, PlaygroundActionEnvelope { request_id: 2, ..action }), PlaygroundEvent::Rejected { error, .. } if error.code == "revision_conflict")
    );
    let changes = host.poll(&id).unwrap();
    assert!(
        matches!(&changes[0], PlaygroundEvent::Delta { base_revision: 0, revision: 1, changed, .. } if changed.get("paused") == Some(&json!(true)))
    );
    assert!(host.poll(&id).unwrap().is_empty());
    assert!(
        matches!(host.connect(&id).unwrap(), PlaygroundEvent::Snapshot { snapshot } if snapshot.revision == 1)
    );
}

#[test]
fn handshake_checks_every_boundary_and_consumes_token_once() {
    let mut guard = PlaygroundHandshakeGuard::new(
        PlaygroundId("test".into()),
        "secret".into(),
        "http://tauri.localhost".into(),
    );
    let hello = PlaygroundHandshake {
        version: PLAYGROUND_PROTOCOL_VERSION,
        playground: PlaygroundId("test".into()),
        token: "secret".into(),
    };
    assert!(guard.accept(&hello, "https://untrusted.example").is_err());
    assert!(
        guard
            .accept(
                &PlaygroundHandshake {
                    token: "wrong".into(),
                    ..hello.clone()
                },
                "http://tauri.localhost"
            )
            .is_err()
    );
    assert!(
        guard
            .accept(
                &PlaygroundHandshake {
                    version: 99,
                    ..hello.clone()
                },
                "http://tauri.localhost"
            )
            .is_err()
    );
    assert!(
        guard
            .accept(
                &PlaygroundHandshake {
                    playground: PlaygroundId("other".into()),
                    ..hello.clone()
                },
                "http://tauri.localhost"
            )
            .is_err()
    );
    guard.accept(&hello, "http://tauri.localhost").unwrap();
    assert!(guard.accept(&hello, "http://tauri.localhost").is_err());
}

#[test]
fn viewport_bounds_preserve_aspect_and_reject_invalid_sizes() {
    let mut request = PlaygroundViewportRequest {
        mode: PlaygroundViewportMode::Jpeg,
        generation: 1,
        adaptive_resolution: false,
        css_width: 1920.0,
        css_height: 1080.0,
        dpr: 2.0,
        interacting: false,
        high_quality_capture: false,
    };
    assert_eq!(request.effective_size([1280, 720]), Ok([1280, 720]));
    request.css_width = 1080.0;
    request.css_height = 1920.0;
    assert_eq!(request.effective_size([1280, 720]), Ok([405, 720]));
    request.dpr = f64::NAN;
    assert!(request.effective_size([1280, 720]).is_err());
}

#[test]
fn slow_consumers_get_latest_frame_and_idle_does_not_render() {
    let queue = Arc::new(PlaygroundFrameQueue::default());
    for _ in 0..2 {
        let permit = queue.acquire().unwrap();
        let frame = PlaygroundViewportFrame {
            header: PlaygroundFrameHeader {
                sequence: permit.sequence, generation: permit.generation, revision: 0,
                size: [1, 1], format: PlaygroundPixelFormat::Rgba8Srgb,
                mode: PlaygroundViewportMode::Jpeg, input_id: 0, stages: Default::default(),
            },
            payload: vec![],
        };
        permit.publish(frame);
    }
    assert!(queue.acquire().is_none());
    let frame = queue.take_latest().unwrap();
    assert_eq!(frame.header.sequence, 2);
    assert!(queue.take_latest().is_none());
    let reserved = queue.acquire().unwrap();
    assert!(queue.acquire().is_none());
    let mut ack = PlaygroundFrameAck { sequence: 2, generation: 1, presented: true, decode_ms: 0.0, present_ms: 0.0 };
    assert!(!queue.acknowledge(&ack));
    ack.generation = 0;
    queue.set_generation(1);
    assert!(queue.acknowledge(&ack));
    assert!(!queue.acknowledge(&ack));
    drop(reserved);
    assert!(queue.acquire().is_some());
    let now = Instant::now();
    let mut policy = PlaygroundFramePolicy::default();
    assert!(!policy.due(now, false, false, false));
    assert!(policy.due(now, true, false, false));
    policy.committed(now);
    assert!(!policy.due(now + Duration::from_millis(17), true, false, false));
    assert!(policy.due(now + Duration::from_millis(17), false, false, true));
    assert!(policy.due(now + Duration::from_millis(34), false, true, false));
}

#[test]
fn delayed_host_wakeups_preserve_cadence_without_catchup_bursts() {
    let mut policy = PlaygroundFramePolicy::default();
    let start = Instant::now();
    let mut produced = 0;
    for tick in 0..1000 {
        let now = start + Duration::from_millis(tick * 2);
        if policy.due(now, false, true, true) {
            policy.committed(now);
            produced += 1;
            assert!(!policy.due(now, false, true, true));
        }
    }
    assert_eq!(produced, 120);
    let delayed = start + Duration::from_secs(5);
    assert!(policy.due(delayed, false, true, true));
    policy.committed(delayed);
    assert!(!policy.due(delayed, false, true, true));
    assert!(!policy.due(delayed + Duration::from_secs(1), false, false, false));
}

#[test]
fn obsolete_production_and_unpublished_ack_cannot_consume_new_generation_credits() {
    let queue = Arc::new(PlaygroundFrameQueue::default());
    let old = queue.acquire().unwrap();
    let ack = PlaygroundFrameAck { sequence: old.sequence, generation: old.generation, presented: true, decode_ms: 0.0, present_ms: 0.0 };
    assert!(!queue.acknowledge(&ack));
    queue.set_generation(7);
    old.publish(PlaygroundViewportFrame { header: PlaygroundFrameHeader {
        sequence: ack.sequence, generation: ack.generation, revision: 0, size: [1, 1],
        format: PlaygroundPixelFormat::Rgba8Srgb, mode: PlaygroundViewportMode::Jpeg,
        input_id: 0, stages: Default::default(),
    }, payload: vec![0] });
    assert!(!queue.produced());
    assert!(queue.take_latest().is_none());
    let first = queue.acquire().unwrap();
    let second = queue.acquire().unwrap();
    assert_eq!(first.generation, 7);
    assert!(queue.acquire().is_none());
    drop((first, second));
    assert!(queue.acquire().is_some());
}
