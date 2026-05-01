    use std::path::PathBuf;

    use amigo_math::{Curve1d, Vec2};

    use super::super::build_scene_hydration_plan;
    use crate::{
        EventPipelineStepSceneCommand,
        ParticleAlignMode2dSceneCommand, ParticleBlendMode2dSceneCommand,
        ParticleSpawnArea2dSceneCommand, SceneCommand, UiModelBindingKindSceneCommand, load_scene_document_from_str,
    };

    #[test]
    fn hydrates_event_pipeline_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: event-pipeline-scene
entities:
  - id: pipeline
    name: collision-pipeline
    components:
      - type: EventPipeline
        id: asteroid-hit
        topic: collision.asteroid_hit
        steps:
          - kind: play_audio
            clip: explosion
          - kind: increment_state
            key: score
            by: 100.0
          - kind: show_ui
            path: hud.root.toast
          - kind: transition_scene
            scene: game-over
          - kind: emit_event
            topic: asteroid.custom
            payload: [hit]
          - kind: script
            function: on_asteroid_hit_pipeline
"#####,
        )
        .expect("event pipeline scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("event pipeline scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueEventPipeline { command }
                if command.id == "asteroid-hit"
                    && command.topic == "collision.asteroid_hit"
                    && command.steps.len() == 6
                    && command.steps.iter().any(|step| matches!(
                        step,
                        EventPipelineStepSceneCommand::Script { function }
                            if function == "on_asteroid_hit_pipeline"
                    ))
        )));
    }

    #[test]
    fn hydrates_ui_model_bindings_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-model-scene
entities:
  - id: bindings
    name: ui-model-bindings
    components:
      - type: UiModelBindings
        bindings:
          - path: editor.root.spawn-rate.value
            state: editor.spawn_rate
            kind: text
            format: "spawn={value}"
          - path: editor.root.spawn-rate.slider
            state: editor.spawn_rate_normalized
            kind: value
          - path: editor.root.preset.dropdown
            state: editor.selected_preset
            kind: selected
          - path: editor.root.preset.dropdown
            state: editor.preset_options
            kind: options
          - path: editor.root.swatch
            state: editor.color
            kind: background
          - path: editor.root
            state: editor.active_theme
            kind: theme
"#####,
        )
        .expect("ui model bindings scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("ui model bindings hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUiModelBindings { command }
                if command.entity_name == "ui-model-bindings"
                    && command.bindings.len() == 6
                    && command.bindings[0].format.as_deref() == Some("spawn={value}")
                    && matches!(command.bindings[2].kind, UiModelBindingKindSceneCommand::Selected)
                    && matches!(command.bindings[3].kind, UiModelBindingKindSceneCommand::Options)
                    && matches!(command.bindings[4].kind, UiModelBindingKindSceneCommand::Background)
                    && matches!(command.bindings[5].kind, UiModelBindingKindSceneCommand::Theme)
        )));
    }

    #[test]
    fn hydrates_script_component_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: script-component-scene
entities:
  - id: actor
    name: script-actor
    components:
      - type: ScriptComponent
        script: scripts/components/bob_motion.rhai
        params:
          amplitude: 12.0
          speed: 2
          enabled: true
          label: bob
"#####,
        )
        .expect("script component scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("script component hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueScriptComponent { command }
                if command.entity_name == "script-actor"
                    && command.script == PathBuf::from("scripts/components/bob_motion.rhai")
                    && command.params.len() == 4
        )));
    }

    #[test]
    fn hydrates_particle_emitter_2d_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: particle-scene
entities:
  - id: emitter
    name: test-emitter
    components:
      - type: ParticleEmitter2D
        attached_to: test-source
        local_offset: { x: -12.0, y: 1.0 }
        local_direction_degrees: 180.0
        spawn_area:
          kind: rect
          size: { x: 120.0, y: 20.0 }
        active: false
        spawn_rate: 90.0
        max_particles: 64
        particle_lifetime: 0.5
        initial_speed: 120.0
        velocity_mode: source_inertial
        simulation_space: source
        initial_size: 2.0
        final_size: 8.0
        color: "#FFFFFFFF"
        shape:
          kind: circle
          segments: 8
        line_anchor: start
        shape_choices:
          - weight: 2.0
            shape: { kind: circle, segments: 8 }
          - weight: 1.0
            shape: { kind: line, length: 14.0 }
        shape_over_lifetime:
          - t: 0.0
            shape: { kind: quad }
          - t: 0.75
            shape: { kind: circle, segments: 12 }
        align: emitter
        blend_mode: additive
        motion_stretch:
          enabled: true
          velocity_scale: 2.2
          max_length: 96.0
        material:
          receives_light: true
          light_response: 0.6
        light:
          radius: 24.0
          intensity: 0.35
          mode: source
          glow: false
        emission_rate_curve:
          kind: ease_out
        forces:
          - kind: gravity
            acceleration: { x: 0.0, y: -480.0 }
          - kind: drag
            coefficient: 1.8
"#####,
        )
        .expect("particle scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("particle scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueParticleEmitter2d { command }
                if command.entity_name == "test-emitter"
                    && command.attached_to.as_deref() == Some("test-source")
                    && command.spawn_rate == 90.0
                    && command.max_particles == 64
                    && command.emission_rate_curve == Curve1d::EaseOut
                    && command.velocity_mode == crate::ParticleVelocityMode2dSceneCommand::SourceInertial
                    && command.simulation_space == crate::ParticleSimulationSpace2dSceneCommand::Source
                    && command.shape_choices.len() == 2
                    && command.shape_over_lifetime.len() == 2
                    && command.line_anchor == crate::ParticleLineAnchor2dSceneCommand::Start
                    && command.align == ParticleAlignMode2dSceneCommand::Emitter
                    && command.blend_mode == ParticleBlendMode2dSceneCommand::Additive
                    && command.motion_stretch.is_some_and(|stretch| stretch.enabled && stretch.velocity_scale == 2.2 && stretch.max_length == 96.0)
                    && command.material.receives_light
                    && (command.material.light_response - 0.6).abs() < f32::EPSILON
                    && command.light.is_some_and(|light| (light.radius - 24.0).abs() < f32::EPSILON && (light.intensity - 0.35).abs() < f32::EPSILON && light.mode == crate::ParticleLightMode2dSceneCommand::Source && !light.glow)
                    && matches!(command.spawn_area, ParticleSpawnArea2dSceneCommand::Rect { size } if size == Vec2::new(120.0, 20.0))
                    && command.forces.len() == 2
        )));
    }
