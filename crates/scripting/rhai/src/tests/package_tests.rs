use super::*;

#[test]
fn script_can_use_input_actions() {
    let input = Arc::new(InputState::default());
    input.set_key(KeyCode::W, true);
    input.set_key(KeyCode::Space, true);

    let actions = Arc::new(InputActionService::default());
    actions.register_map(
        InputActionMap {
            id: "gameplay".to_owned(),
            actions: BTreeMap::from([
                (
                    InputActionId::new("actor.thrust"),
                    InputActionBinding::Axis {
                        positive: vec![KeyCode::W],
                        negative: vec![KeyCode::S],
                    },
                ),
                (
                    InputActionId::new("actor.fire"),
                    InputActionBinding::Button {
                        pressed: vec![KeyCode::Space],
                    },
                ),
            ]),
        },
        true,
    );

    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(input),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(actions),
        None,
    );

    runtime
        .execute(
            "actions-test",
            r#"
                    if world.actions.active_map() != "gameplay" {
                        throw("wrong active action map");
                    }
                    if world.actions.axis("actor.thrust") != 1.0 {
                        throw("wrong action axis");
                    }
                    if !world.actions.down("actor.fire") {
                        throw("fire should be down");
                    }
                    if !world.actions.pressed("actor.fire") {
                        throw("fire should be pressed");
                    }
                    if world.actions.axis("missing") != 0.0 {
                        throw("missing axis should be neutral");
                    }
                "#,
        )
        .expect("script should read input actions");
}

#[test]
fn script_can_drive_freeflight_with_arcade_actions() {
    let input = Arc::new(InputState::default());
    input.set_key(KeyCode::W, true);
    input.set_key(KeyCode::D, true);

    let actions = Arc::new(InputActionService::default());
    actions.register_map(
        InputActionMap {
            id: "gameplay".to_owned(),
            actions: BTreeMap::from([
                (
                    InputActionId::new("actor.thrust"),
                    InputActionBinding::Axis {
                        positive: vec![KeyCode::W],
                        negative: vec![KeyCode::S],
                    },
                ),
                (
                    InputActionId::new("actor.turn"),
                    InputActionBinding::Axis {
                        positive: vec![KeyCode::A],
                        negative: vec![KeyCode::D],
                    },
                ),
            ]),
        },
        true,
    );

    let motion = Arc::new(Motion2dSceneService::default());
    motion.queue_freeflight(FreeflightMotion2dCommand {
        entity_id: SceneEntityId::new(7),
        entity_name: "actor".to_owned(),
        profile: test_freeflight_profile(),
        initial_state: FreeflightMotionState2d::default(),
    });

    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
        None,
        None,
        None,
        Some(motion.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(input),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(actions),
        None,
    );

    runtime
        .execute(
            "arcade-actions-test",
            r#"
                    if !world.arcade.drive_freeflight("actor", "actor.thrust", "actor.turn") {
                        throw("arcade drive should succeed");
                    }
                "#,
        )
        .expect("script should drive freeflight through arcade API");

    let intent = motion
        .freeflight_intent("actor")
        .expect("actor should have freeflight intent");
    assert_eq!(intent.thrust, 1.0);
    assert_eq!(intent.turn, -1.0);
}

#[test]
fn script_can_drive_freeflight_with_arcade_emitter() {
    let input = Arc::new(InputState::default());
    input.set_key(KeyCode::W, true);

    let actions = Arc::new(InputActionService::default());
    actions.register_map(
        InputActionMap {
            id: "gameplay".to_owned(),
            actions: BTreeMap::from([
                (
                    InputActionId::new("actor.thrust"),
                    InputActionBinding::Axis {
                        positive: vec![KeyCode::W],
                        negative: vec![KeyCode::S],
                    },
                ),
                (
                    InputActionId::new("actor.turn"),
                    InputActionBinding::Axis {
                        positive: vec![KeyCode::A],
                        negative: vec![KeyCode::D],
                    },
                ),
            ]),
        },
        true,
    );

    let motion = Arc::new(Motion2dSceneService::default());
    motion.queue_freeflight(FreeflightMotion2dCommand {
        entity_id: SceneEntityId::new(8),
        entity_name: "actor".to_owned(),
        profile: test_freeflight_profile(),
        initial_state: FreeflightMotionState2d::default(),
    });

    let particles = Arc::new(Particle2dSceneService::default());
    particles.queue_emitter(ParticleEmitter2dCommand {
        entity_id: SceneEntityId::new(45),
        entity_name: "emitter".to_owned(),
        emitter: test_particle_emitter(),
    });

    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
        None,
        None,
        None,
        Some(motion.clone()),
        Some(particles.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(input),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(actions),
        None,
    );

    runtime
            .execute(
                "arcade-emitter-test",
                r#"
                    import "pkg:amigo.arcade_2d/freeflight" as freeflight;

                    if !freeflight::drive_freeflight_with_emitter(world, "actor", "emitter", "actor.thrust", "actor.turn") {
                        throw("arcade drive with emitter should succeed");
                    }
                "#,
            )
            .expect("script should drive freeflight and emitter through arcade package");

    let intent = motion
        .freeflight_intent("actor")
        .expect("actor should have freeflight intent");
    assert_eq!(intent.thrust, 1.0);
    assert!(particles.is_active("emitter"));
    assert_eq!(particles.intensity("emitter"), 1.0);
}
