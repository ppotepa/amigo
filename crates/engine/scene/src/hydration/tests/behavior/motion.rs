    use amigo_math::Curve1d;

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        BehaviorKindSceneCommand, SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_freeflight_motion_response_curves() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: curve-motion
entities:
  - id: ship
    name: curve-ship
    components:
      - type: FreeflightMotion2D
        thrust_acceleration: 100.0
        reverse_acceleration: 50.0
        strafe_acceleration: 20.0
        turn_acceleration: 8.0
        linear_damping: 2.0
        turn_damping: 3.0
        max_speed: 300.0
        max_angular_speed: 4.0
        thrust_response_curve:
          kind: ease_out
        reverse_response_curve:
          kind: ease_in
        strafe_response_curve:
          kind: constant
          value: 0.5
        turn_response_curve:
          kind: smooth_step
"#####,
        )
        .expect("freeflight curve scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("freeflight curve hydration plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueFreeflightMotion2d { command }
                if command.entity_name == "curve-ship"
                    && command.thrust_response_curve == Curve1d::EaseOut
                    && command.reverse_response_curve == Curve1d::EaseIn
                    && command.strafe_response_curve == Curve1d::Constant(0.5)
                    && command.turn_response_curve == Curve1d::SmoothStep
        )));
    }

    #[test]
    fn hydrates_input_action_map_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: input-actions-scene
entities:
  - id: controls
    name: gameplay-controls
    components:
      - type: InputActionMap
        id: gameplay
        active: true
        actions:
          actor.accelerate:
            kind: axis
            positive: [ArrowUp, KeyW]
            negative: [ArrowDown, KeyS]
          actor.primary:
            kind: button
            pressed: [Space]
"#####,
        )
        .expect("input action scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("input action scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueInputActionMap { command }
                if command.entity_name == "gameplay-controls"
                    && command.id == "gameplay"
                    && command.active
                    && command.actions.len() == 2
        )));
    }

    #[test]
    fn hydrates_freeflight_input_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: behavior-scene
entities:
  - id: controller
    name: ship-controller
    components:
      - type: Behavior
        enabled_when:
          state: game_mode
          equals: playing
        kind: freeflight_input_controller
        target: ship
        input:
          thrust: ship.accelerate
          turn: ship.turn
"#####,
        )
        .expect("behavior scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("behavior scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "ship-controller"
                    && command
                        .condition
                        .as_ref()
                        .is_some_and(|condition| condition.state_key == "game_mode"
                            && condition.equals.as_deref() == Some("playing"))
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::FreeflightInputController {
                            target_entity,
                            thrust_action,
                            turn_action,
                            ..
                        } if target_entity == "ship"
                            && thrust_action == "ship.accelerate"
                            && turn_action == "ship.turn"
            )
        )));
    }

