    

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn builds_hydration_plan_for_sidescroller_components() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: vertical-slice
  label: Vertical Slice
entities:
  - id: camera
    name: playground-sidescroller-camera
    components:
      - type: Camera2D
      - type: CameraFollow2D
        target: playground-sidescroller-player
  - id: tilemap
    name: playground-sidescroller-tilemap
    components:
      - type: TileMap2D
        tileset: playground-sidescroller/tilesets/platformer
        ruleset: playground-sidescroller/tilesets/platformer-rules
        tile_size: { x: 16.0, y: 16.0 }
        grid:
          - "...."
          - ".P.."
          - "####"
  - id: player
    name: playground-sidescroller-player
    components:
      - type: TileMapMarker2D
        tilemap_entity: playground-sidescroller-tilemap
        symbol: "P"
        offset: { x: 0.0, y: 8.0 }
      - type: KinematicBody2D
        velocity: { x: 0.0, y: 0.0 }
        gravity_scale: 1.0
        terminal_velocity: 720.0
      - type: AabbCollider2D
        size: { x: 20.0, y: 30.0 }
        offset: { x: 0.0, y: 1.0 }
        layer: player
        mask: [world, trigger]
      - type: MotionController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
  - id: coin
    name: playground-sidescroller-coin
    components:
      - type: Sprite2D
        texture: playground-sidescroller/textures/coin
        size: { x: 16.0, y: 16.0 }
        animation:
          fps: 10.0
          looping: true
      - type: Trigger2D
        size: { x: 16.0, y: 16.0 }
        layer: trigger
        mask: [player]
        event: coin.collected
"#####,
        )
        .expect("sidescroller scene should parse");

        let plan = build_scene_hydration_plan("playground-sidescroller", &document)
            .expect("sidescroller hydration plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTileMap2d { command }
                if command.entity_name == "playground-sidescroller-tilemap"
                    && command.ruleset.as_ref().map(|ruleset| ruleset.as_str())
                        == Some("playground-sidescroller/tilesets/platformer-rules")
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueKinematicBody2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueAabbCollider2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMotionController2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTileMapMarker2d { command }
                if command.entity_name == "playground-sidescroller-player"
                    && command.symbol == "P"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueCameraFollow2d { command }
                if command.entity_name == "playground-sidescroller-camera"
                    && command.target == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTrigger2d { command }
                if command.entity_name == "playground-sidescroller-coin"
                    && command.event.as_deref() == Some("coin.collected")
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-sidescroller-coin"
                    && command.animation.as_ref().and_then(|animation| animation.fps) == Some(10.0)
                    && command.animation.as_ref().and_then(|animation| animation.looping) == Some(true)
        )));
    }

