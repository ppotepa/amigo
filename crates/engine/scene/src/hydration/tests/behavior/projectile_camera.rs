    

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        BehaviorKindSceneCommand, SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_projectile_fire_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: fire-controller
    name: ship-fire-controller
    components:
      - type: Behavior
        enabled_when:
          state: game_mode
          equals: playing
        kind: projectile_fire_controller
        emitter: ship-gun
        source: ship
        action: ship.fire
        cooldown: 0.16
        cooldown_id: ship.fire.cooldown
        audio: shot
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "ship-fire-controller"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::ProjectileFireController {
                            emitter,
                            source,
                            action,
                            cooldown_seconds,
                            cooldown_id,
                            audio,
                        } if emitter == "ship-gun"
                            && source.as_deref() == Some("ship")
                            && action == "ship.fire"
                            && (*cooldown_seconds - 0.16).abs() < f32::EPSILON
                            && cooldown_id.as_deref() == Some("ship.fire.cooldown")
                            && audio.as_deref() == Some("shot")
                    )
        )));
    }

    #[test]
    fn hydrates_camera_follow_mode_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: camera-mode
    name: camera-mode
    components:
      - type: Behavior
        kind: camera_follow_mode_controller
        camera: camera
        action: camera.fast
        target: ship
        lerp: 0.12
        lookahead_velocity_scale: 0.35
        lookahead_max_distance: 180.0
        sway_amount: 18.0
        sway_frequency: 1.4
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "camera-mode"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::CameraFollowModeController {
                            camera,
                            action,
                            target,
                            lerp,
                            lookahead_velocity_scale,
                            lookahead_max_distance,
                            sway_amount,
                            sway_frequency,
                        } if camera == "camera"
                            && action == "camera.fast"
                            && target.as_deref() == Some("ship")
                            && lerp.is_some_and(|value| (value - 0.12).abs() < f32::EPSILON)
                            && lookahead_velocity_scale.is_some_and(|value| (value - 0.35).abs() < f32::EPSILON)
                            && lookahead_max_distance.is_some_and(|value| (value - 180.0).abs() < f32::EPSILON)
                            && sway_amount.is_some_and(|value| (value - 18.0).abs() < f32::EPSILON)
                            && sway_frequency.is_some_and(|value| (value - 1.4).abs() < f32::EPSILON)
                    )
        )));
    }

