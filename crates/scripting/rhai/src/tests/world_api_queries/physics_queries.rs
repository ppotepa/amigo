use super::*;

#[test]
fn exposes_world_physics_overlap_queries_to_scripts() {
    let scene = Arc::new(SceneService::default());
    scene.spawn("bullet");
    scene.spawn("target");
    assert!(scene.configure_entity_metadata(
        "target",
        SceneEntityLifecycle::default(),
        vec!["hazard".to_owned()],
        vec!["targets".to_owned()],
        BTreeMap::new(),
    ));
    assert!(scene.set_transform(
        "bullet",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(16.0, 24.0, 0.0),
            ..Default::default()
        },
    ));
    assert!(scene.set_transform(
        "target",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(24.0, 24.0, 0.0),
            ..Default::default()
        },
    ));

    let physics = Arc::new(Physics2dSceneService::default());
    physics.queue_circle_collider(CircleCollider2dCommand {
        entity_id: SceneEntityId::new(0),
        entity_name: "bullet".to_owned(),
        collider: CircleCollider2d {
            radius: 4.0,
            offset: Vec2::ZERO,
        },
    });
    physics.queue_circle_collider(CircleCollider2dCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "target".to_owned(),
        collider: CircleCollider2d {
            radius: 6.0,
            offset: Vec2::ZERO,
        },
    });

    let runtime = RhaiScriptRuntime::new_with_motion(
        Some(scene),
        None,
        None,
        Some(physics.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "world-physics-test",
            r#"
                fn update(dt) {
                    if !world.physics.overlaps("bullet", "target") {
                        throw("physics overlap should be true");
                    }

                    if world.physics.overlaps("bullet", "missing") {
                        throw("missing collider should return false");
                    }

                    let hit = world.physics.first_overlap("bullet", ["missing", "target"]);
                    if hit != "target" {
                        throw("first overlap should return target");
                    }

                    let hit_index = world.physics.first_overlap_index("bullet", ["missing", "target"]);
                    if hit_index != 1 {
                        throw("first overlap index should return the candidate index");
                    }

                    let no_hit = world.physics.first_overlap("bullet", ["missing", "ghost"]);
                    if no_hit != "" {
                        throw("first overlap should return empty string when nothing matches");
                    }

                    let no_hit_index = world.physics.first_overlap_index("bullet", ["missing", "ghost"]);
                    if no_hit_index != -1 {
                        throw("first overlap index should return -1 when nothing matches");
                    }

                    if world.physics.first_overlap_by_tag("bullet", "hazard") != "target" {
                        throw("tag selector overlap should return target");
                    }

                    if world.physics.first_overlap_by_group("bullet", "targets") != "target" {
                        throw("group selector overlap should return target");
                    }

                    if world.physics.first_overlap_by_selector("bullet", "tag", "hazard") != "target" {
                        throw("generic selector overlap should return target");
                    }

                    if !world.physics.overlaps_by_tag("bullet", "hazard") {
                        throw("overlaps_by_tag should be true");
                    }

                    if world.physics.selector_candidates("tag", "hazard").len() != 1 {
                        throw("selector candidates should include tagged collider");
                    }

                    if !world.physics.set_circle_radius("target", 16.0) {
                        throw("circle radius setter should succeed");
                    }
                }
            "#,
        )
        .expect("script execution should succeed");
    runtime
        .call_update("world-physics-test", 1.0 / 60.0)
        .expect("update should succeed");
    assert_eq!(
        physics
            .circle_collider("target")
            .expect("target circle should exist")
            .collider
            .radius,
        16.0
    );
}

#[test]
fn update_function_can_set_vector_polygon_points() {
    let vector_scene = Arc::new(VectorSceneService::default());
    vector_scene.queue(VectorShape2dDrawCommand {
        entity_id: SceneEntityId::new(9),
        entity_name: "test-polygon".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polygon {
                points: vec![
                    Vec2::new(-10.0, -10.0),
                    Vec2::new(0.0, 10.0),
                    Vec2::new(10.0, -10.0),
                ],
            },
            style: VectorStyle2d::default(),
        },
        z_index: 0.0,
        transform: Transform2::default(),
    });

    let runtime = RhaiScriptRuntime::new_with_motion_and_vector(
        None,
        None,
        Some(vector_scene.clone()),
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
    );

    runtime
        .execute(
            "world-vector-test",
            r#"
                fn update(dt) {
                    if !world.vector.set_polygon("test-polygon", [[-12.0, -4.0], [0.0, 14.0], [12.0, -6.0], [2.0, -15.0]]) {
                        throw("set_polygon should succeed");
                    }
                }
            "#,
        )
        .expect("script execution should succeed");
    runtime
        .call_update("world-vector-test", 1.0 / 60.0)
        .expect("update should succeed");

    let commands = vector_scene.commands();
    assert_eq!(commands.len(), 1);
    match &commands[0].shape.kind {
        VectorShapeKind2d::Polygon { points } => {
            assert_eq!(points.len(), 4);
            assert_eq!(points[0], Vec2::new(-12.0, -4.0));
        }
        other => panic!("expected polygon shape, got {other:?}"),
    }
}
