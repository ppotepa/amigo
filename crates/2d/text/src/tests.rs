use crate::{
    Text2d, Text2dDrawCommand, Text2dSceneService, TextSceneCommandContext,
    can_handle_text_scene_command, handle_text_scene_command, queue_text2d_scene_command,
};
use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, Text2dSceneCommand};

#[test]
fn stores_text_draw_commands() {
    let service = Text2dSceneService::default();

    service.queue(Text2dDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(9),
        entity_name: "playground-2d-label".to_owned(),
        render_layer: "default".to_owned(),
        text: Text2d {
            content: "AMIGO 2D".to_owned(),
            font: AssetKey::new("playground-2d/fonts/debug-ui"),
            bounds: Vec2::new(320.0, 64.0),
            transform: Transform2::default(),
        },
        z_index: 0.0,
    });

    assert_eq!(service.commands().len(), 1);
    assert_eq!(
        service.entity_names(),
        vec!["playground-2d-label".to_owned()]
    );

    service.clear();
    assert!(service.commands().is_empty());
}

#[test]
fn queues_text2d_scene_command() {
    let scene = SceneService::default();
    let service = Text2dSceneService::default();

    let entity = queue_text2d_scene_command(
        &scene,
        &service,
        &Text2dSceneCommand::new(
            "playground-2d",
            "playground-2d-label",
            "AMIGO 2D",
            AssetKey::new("playground-2d/fonts/debug-ui"),
            Vec2::new(320.0, 64.0),
        ),
    );

    assert_eq!(entity.raw(), 0);
    assert_eq!(service.commands().len(), 1);
    assert_eq!(scene.entity_names(), vec!["playground-2d-label".to_owned()]);
}

#[test]
fn can_handle_text_scene_command_returns_true_for_text_command() {
    let command = SceneCommand::QueueText2d {
        command: Text2dSceneCommand::new(
            "playground-2d",
            "playground-2d-label",
            "AMIGO 2D",
            AssetKey::new("playground-2d/fonts/debug-ui"),
            Vec2::new(320.0, 64.0),
        ),
    };

    assert!(can_handle_text_scene_command(&command));
}

#[test]
fn handle_text_scene_command_queues_text_and_publishes_event() {
    let scene_service = SceneService::default();
    let text_scene_service = Text2dSceneService::default();
    let scene_event_queue = SceneEventQueue::default();
    let command = SceneCommand::QueueText2d {
        command: Text2dSceneCommand::new(
            "playground-2d",
            "playground-2d-label",
            "AMIGO 2D",
            AssetKey::new("playground-2d/fonts/debug-ui"),
            Vec2::new(320.0, 64.0),
        ),
    };

    let outcome = handle_text_scene_command(
        TextSceneCommandContext {
            scene_service: &scene_service,
            text_scene_service: &text_scene_service,
            scene_event_queue: &scene_event_queue,
        },
        command,
    )
    .expect("text scene command should be handled");

    assert_eq!(outcome.entity_name, "playground-2d-label");
    assert_eq!(outcome.source_mod, "playground-2d");
    assert_eq!(outcome.font.as_str(), "playground-2d/fonts/debug-ui");
    assert_eq!(
        scene_service.entity_names(),
        vec!["playground-2d-label".to_owned()]
    );
    assert_eq!(text_scene_service.commands().len(), 1);

    let events = scene_event_queue.drain();
    assert_eq!(events.len(), 1);
    match &events[0] {
        SceneEvent::TextQueued {
            entity_id,
            entity_name,
            font,
        } => {
            assert_eq!(*entity_id, 0);
            assert_eq!(entity_name, "playground-2d-label");
            assert_eq!(font.as_str(), "playground-2d/fonts/debug-ui");
        }
        other => panic!("expected text queued event, got {other:?}"),
    }
}
