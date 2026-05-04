    
    

    use crate::{
        AabbCollider2dSceneCommand,
        KinematicBody2dSceneCommand, Material3dSceneCommand, Mesh3dSceneCommand,
        MotionController2dSceneCommand,
        SceneCommand, SceneCommandQueue, SceneEvent, SceneEventQueue,
        SceneKey, SceneTransitionService, Sprite2dSceneCommand,
        Text2dSceneCommand, TileMap2dSceneCommand, TileMapMarker2dSceneCommand,
        Trigger2dSceneCommand,
    };
    use amigo_assets::AssetKey;
    use amigo_math::{Transform3, Vec2};

    #[test]
    fn queues_placeholder_scene_commands_and_events() {
        let commands = SceneCommandQueue::default();
        let events = SceneEventQueue::default();

        commands.submit(SceneCommand::SelectScene {
            scene: SceneKey::new("mesh-lab"),
        });
        commands.submit(SceneCommand::ReloadActiveScene);
        events.publish(SceneEvent::SceneSelected {
            scene: SceneKey::new("mesh-lab"),
        });
        events.publish(SceneEvent::SceneReloadRequested {
            scene: SceneKey::new("mesh-lab"),
        });

        assert_eq!(commands.pending().len(), 2);
        assert_eq!(events.pending().len(), 2);
        assert_eq!(commands.drain().len(), 2);
        assert_eq!(events.drain().len(), 2);
    }

    #[test]
    fn scene_plugin_transition_service_defaults_to_empty() {
        let transitions = SceneTransitionService::default();
        assert_eq!(transitions.snapshot().transition_ids.len(), 0);
    }

    #[test]
    fn queues_domain_scene_commands() {
        let commands = SceneCommandQueue::default();

        commands.submit(SceneCommand::SpawnNamedEntity {
            name: "playground-2d-sprite".to_owned(),
            transform: Some(Transform3::default()),
        });

        commands.submit(SceneCommand::QueueSprite2d {
            command: Sprite2dSceneCommand::new(
                "playground-2d",
                "playground-2d-sprite",
                AssetKey::new("playground-2d/images/sprite-lab"),
                Vec2::new(128.0, 128.0),
            ),
        });
        commands.submit(SceneCommand::QueueText2d {
            command: Text2dSceneCommand::new(
                "playground-2d",
                "playground-2d-label",
                "AMIGO 2D",
                AssetKey::new("playground-2d/fonts/debug-ui"),
                Vec2::new(320.0, 64.0),
            ),
        });
        commands.submit(SceneCommand::QueueTileMap2d {
            command: TileMap2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-tilemap",
                AssetKey::new("playground-sidescroller/tilesets/platformer"),
                Vec2::new(16.0, 16.0),
                vec!["....".to_owned(), "####".to_owned()],
            ),
        });
        commands.submit(SceneCommand::QueueKinematicBody2d {
            command: KinematicBody2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                Vec2::ZERO,
                1.0,
                720.0,
            ),
        });
        commands.submit(SceneCommand::QueueAabbCollider2d {
            command: AabbCollider2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                Vec2::new(20.0, 30.0),
                Vec2::new(0.0, 1.0),
                "player",
                vec!["world".to_owned(), "trigger".to_owned()],
            ),
        });
        commands.submit(SceneCommand::QueueTrigger2d {
            command: Trigger2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-coin",
                Vec2::new(16.0, 16.0),
                Vec2::ZERO,
                "trigger",
                vec!["player".to_owned()],
                Some("coin.collected".to_owned()),
            ),
        });
        commands.submit(SceneCommand::queue_motion_controller(
            MotionController2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                180.0,
                900.0,
                1200.0,
                500.0,
                900.0,
                -360.0,
                720.0,
            ),
        ));
        commands.submit(SceneCommand::QueueTileMapMarker2d {
            command: TileMapMarker2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                Some("playground-sidescroller-tilemap".to_owned()),
                "P",
                0,
                Vec2::new(0.0, 8.0),
            ),
        });
        commands.submit(SceneCommand::QueueMesh3d {
            command: Mesh3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                AssetKey::new("playground-3d/meshes/probe"),
            ),
        });
        commands.submit(SceneCommand::QueueMaterial3d {
            command: Material3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                "debug-surface",
                Some(AssetKey::new("playground-3d/materials/debug-surface")),
            ),
        });

        assert_eq!(commands.pending().len(), 11);
        assert_eq!(commands.drain().len(), 11);
    }

