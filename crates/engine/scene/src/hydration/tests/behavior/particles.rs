    use amigo_math::Curve1d;

    use super::super::super::build_scene_hydration_plan;
    use crate::{
        BehaviorKindSceneCommand, SceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_particle_profile_controller_behavior_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: particle-profile-scene
entities:
  - id: profile
    name: intensity-profile
    components:
      - type: Behavior
        kind: particle_profile_controller
        emitter: main-emitter
        action: actor.accelerate
        max_hold_seconds: 5.0
        phases:
          - id: ignition
            start_seconds: 0.0
            end_seconds: 0.5
            velocity_mode: source_inertial
            clear_forces: true
            color_ramp:
              stops:
                - { t: 0.0, color: "#FFFFFFFF" }
                - { t: 1.0, color: "#0033FF00" }
            spawn_rate:
              from: 20.0
              to: 80.0
              curve: { kind: ease_out }
              intensity_scale: 10.0
              noise_scale: 5.0
            shape_line:
              from: 4.0
              to: 48.0
            alpha_curve:
              v0: { from: 1.0, to: 1.0 }
              v1: { from: 0.8, to: 0.8 }
              v2: { from: 0.4, to: 0.4 }
              v3: { from: 0.0, to: 0.0 }
"#####,
        )
        .expect("particle profile scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("particle profile scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueBehavior { command }
                if command.entity_name == "intensity-profile"
                    && matches!(
                        &command.behavior,
                        BehaviorKindSceneCommand::ParticleProfileController {
                            emitter,
                            action,
                            max_hold_seconds,
                            phases,
                        } if emitter == "main-emitter"
                            && action == "actor.accelerate"
                            && (*max_hold_seconds - 5.0).abs() < f32::EPSILON
                            && phases.len() == 1
                            && phases[0].velocity_mode
                                == Some(crate::ParticleProfileVelocityModeSceneCommand::SourceInertial)
                            && phases[0].color_ramp.is_some()
                            && phases[0]
                                .spawn_rate
                                .as_ref()
                                .is_some_and(|scalar| scalar.curve == Curve1d::EaseOut
                                    && (scalar.noise_scale - 5.0).abs() < f32::EPSILON)
                            && phases[0].alpha_curve.is_some()
                    )
        )));
    }

