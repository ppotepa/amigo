use super::*;

    #[test]
    fn update_function_can_rotate_scene_entities() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("playground-2d-square");
        scene.spawn("playground-3d-cube");
        let input = Arc::new(InputState::default());
        input.set_key(KeyCode::Left, true);
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            Some(input),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "rotate-test",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");
                        let cube = world.entities.named("playground-3d-cube");

                        if world.input.down("ArrowLeft") {
                            let square_rotated = square.rotate_2d(dt);
                            let cube_rotated = cube.rotate_3d(dt, dt * 2.0, 0.0);
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("rotate-test", 1.0)
            .expect("update function should succeed");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("2d entity should exist")
                .rotation_euler
                .z,
            1.0
        );
        let cube = scene
            .transform_of("playground-3d-cube")
            .expect("3d entity should exist");
        assert_eq!(cube.rotation_euler.x, 1.0);
        assert_eq!(cube.rotation_euler.y, 2.0);
    }

    #[test]
    fn update_function_can_use_world_input_and_entity_refs() {
        let scene = Arc::new(SceneService::default());
        scene.spawn("playground-2d-square");
        let input = Arc::new(InputState::default());
        input.set_key(KeyCode::Right, true);
        let runtime = RhaiScriptRuntime::new(
            Some(scene.clone()),
            None,
            None,
            Some(input),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        runtime
            .execute(
                "world-update-test",
                r#"
                    fn update(dt) {
                        let square = world.entities.named("playground-2d-square");

                        if world.input.down("ArrowRight") {
                            let applied = square.rotate_2d(dt);
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-update-test", 0.5)
            .expect("update function should succeed");

        assert_eq!(
            scene
                .transform_of("playground-2d-square")
                .expect("2d entity should exist")
                .rotation_euler
                .z,
            0.5
        );
    }

    #[test]
    fn update_function_can_drive_motion_controller_and_read_state() {
        let motion_scene = Arc::new(Motion2dSceneService::default());
        motion_scene.queue_motion_controller(MotionController2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-player".to_owned(),
            controller: MotionController2d {
                params: MotionProfile2d {
                    max_speed: 180.0,
                    acceleration: 900.0,
                    deceleration: 1200.0,
                    air_acceleration: 500.0,
                    gravity: 900.0,
                    jump_velocity: -360.0,
                    terminal_velocity: 720.0,
                },
            },
        });
        assert!(motion_scene.sync_motion_state(
            "playground-sidescroller-player",
            MotionState2d {
                grounded: true,
                facing: Facing2d::Right,
                animation: MotionAnimationState::Run,
                velocity: Vec2::new(12.0, -4.0),
            }
        ));

        let runtime = RhaiScriptRuntime::new_with_motion(
            None,
            None,
            Some(motion_scene.clone()),
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
                "world-motion-controller-test",
                r#"
                    fn update(dt) {
                        let state = world.motion.state("playground-sidescroller-player");
                        if !state.grounded { throw("state should expose grounded"); }
                        if state.facing != "right" { throw("state should expose facing"); }
                        if state.animation != "run" { throw("state should expose animation"); }
                        if state.velocity_x < 10.0 { throw("state should expose velocity_x"); }
                        if !world.motion.drive("playground-sidescroller-player", -1.0, true, false, dt) {
                            throw("drive should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-motion-controller-test", 1.0 / 60.0)
            .expect("update should succeed");

        assert_eq!(
            motion_scene.motion_intent("playground-sidescroller-player"),
            Some(MotionIntent2d {
                move_x: -1.0,
                jump_pressed: true,
                jump_held: false,
            })
        );
    }

    #[test]
    fn update_function_can_drive_motion_alias_and_read_state() {
        let motion_scene = Arc::new(Motion2dSceneService::default());
        motion_scene.queue_motion_controller(MotionController2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-player".to_owned(),
            controller: MotionController2d {
                params: MotionProfile2d {
                    max_speed: 180.0,
                    acceleration: 900.0,
                    deceleration: 1200.0,
                    air_acceleration: 500.0,
                    gravity: 900.0,
                    jump_velocity: -360.0,
                    terminal_velocity: 720.0,
                },
            },
        });
        assert!(motion_scene.sync_motion_state(
            "playground-sidescroller-player",
            MotionState2d {
                grounded: true,
                facing: Facing2d::Right,
                animation: MotionAnimationState::Run,
                velocity: Vec2::new(12.0, -4.0),
            }
        ));

        let runtime = RhaiScriptRuntime::new_with_motion(
            None,
            None,
            Some(motion_scene.clone()),
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
                "world-motion-test",
                r#"
                    fn update(dt) {
                        let state = world.motion.state("playground-sidescroller-player");
                        if !state.grounded { throw("state should expose grounded"); }
                        if state.facing != "right" { throw("state should expose facing"); }
                        if state.animation != "run" { throw("state should expose animation"); }
                        if state.velocity_x < 10.0 { throw("state should expose velocity_x"); }
                        if !world.motion.drive("playground-sidescroller-player", 1.0, false, true, dt) {
                            throw("drive should succeed");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");
        runtime
            .call_update("world-motion-test", 1.0 / 60.0)
            .expect("update should succeed");

        assert_eq!(
            motion_scene.motion_intent("playground-sidescroller-player"),
            Some(MotionIntent2d {
                move_x: 1.0,
                jump_pressed: false,
                jump_held: true,
            })
        );
    }

    #[test]
    fn projectiles_fire_from_activates_pooled_projectile() {
        let scene = Arc::new(SceneService::default());
        let motion_scene = Arc::new(Motion2dSceneService::default());
        let pool_scene = Arc::new(EntityPoolSceneService::default());

        scene.spawn_with_transform(
            "player",
            Transform3 {
                translation: Vec3::new(10.0, 20.0, 0.0),
                rotation_euler: Vec3::new(0.0, 0.0, 0.0),
                ..Transform3::default()
            },
        );
        scene.spawn_with_transform(
            "bullet-a",
            Transform3 {
                translation: Vec3::new(-100.0, -100.0, 0.0),
                ..Transform3::default()
            },
        );
        let _ = scene.set_visible("bullet-a", false);
        let _ = scene.set_simulation_enabled("bullet-a", false);
        let _ = scene.set_collision_enabled("bullet-a", false);

        pool_scene.queue(EntityPoolSceneCommand::new(
            "test",
            "bullets",
            vec!["bullet-a".to_owned()],
        ));
        motion_scene.queue_projectile_emitter(ProjectileEmitter2dCommand {
            entity_id: SceneEntityId::new(3),
            entity_name: "player-gun".to_owned(),
            emitter: ProjectileEmitter2d {
                pool: "bullets".to_owned(),
                speed: 100.0,
                spawn_offset: Vec2::new(5.0, 0.0),
                inherit_velocity_scale: 0.0,
            },
        });

        let runtime = RhaiScriptRuntime::new_with_services(
            Some(scene.clone()),
            None,
            None,
            Some(motion_scene),
            None,
            None,
            Some(pool_scene.clone()),
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
        );

        runtime
            .execute(
                "projectile-fire-test",
                r#"
                    if !world.projectiles.fire_from("player-gun", "player") {
                        throw("fire_from should activate projectile");
                    }
                    if world.pools.active_members("bullets").len != 1 {
                        throw("pool should report active projectile");
                    }
                    if world.pools.active_count("bullets") != 1 {
                        throw("pool should report active count");
                    }
                "#,
            )
            .expect("script execution should succeed");

        assert!(scene.is_visible("bullet-a"));
        assert!(scene.is_simulation_enabled("bullet-a"));
        assert!(scene.is_collision_enabled("bullet-a"));
        assert_eq!(
            pool_scene.active_members("bullets"),
            vec!["bullet-a".to_owned()]
        );
        let bullet_transform = scene
            .transform_of("bullet-a")
            .expect("projectile should have a transform");
        assert_eq!(bullet_transform.translation, Vec3::new(15.0, 20.0, 0.0));
    }

    #[test]
    fn projectiles_release_returns_pooled_projectile_without_teleporting() {
        let scene = Arc::new(SceneService::default());
        let pool_scene = Arc::new(EntityPoolSceneService::default());
        let projectile_transform = Transform3 {
            translation: Vec3::new(42.0, -7.0, 0.0),
            ..Transform3::default()
        };
        scene.spawn_with_transform("bullet-a", projectile_transform);
        pool_scene.queue(EntityPoolSceneCommand::new(
            "test",
            "bullets",
            vec!["bullet-a".to_owned()],
        ));
        assert_eq!(
            pool_scene.acquire(&scene, "bullets"),
            Some("bullet-a".to_owned())
        );

        let runtime = RhaiScriptRuntime::new_with_services(
            Some(scene.clone()),
            None,
            None,
            None,
            None,
            None,
            Some(pool_scene.clone()),
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
        );

        runtime
            .execute(
                "projectile-release-test",
                r#"
                    if !world.projectiles.release("bullets", "bullet-a") {
                        throw("release should return active projectile");
                    }
                    if world.pools.active_members("bullets").len != 0 {
                        throw("pool should no longer report active projectile");
                    }
                    if world.pools.active_count("bullets") != 0 {
                        throw("pool active count should be zero after release");
                    }
                    if world.pools.acquire("bullets") != "bullet-a" {
                        throw("pool acquire should reuse released projectile");
                    }
                    if world.pools.release_all("bullets") != 1 {
                        throw("release_all should release one projectile");
                    }
                "#,
            )
            .expect("script execution should succeed");

        assert!(!scene.is_visible("bullet-a"));
        assert!(!scene.is_simulation_enabled("bullet-a"));
        assert!(!scene.is_collision_enabled("bullet-a"));
        assert_eq!(scene.transform_of("bullet-a"), Some(projectile_transform));
        assert!(pool_scene.active_members("bullets").is_empty());
    }

