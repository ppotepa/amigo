    
    

    use crate::{
        EntityPoolSceneCommand, EntityPoolSceneService, LifetimeExpirationOutcome, LifetimeSceneCommand,
        LifetimeSceneService, SceneService,
    };
    
    use amigo_math::{Transform3, Vec3};

    #[test]
    fn entity_pool_acquires_first_free_slot_and_reports_no_slot() {
        let scene = SceneService::default();
        scene.spawn("projectile-a");
        scene.spawn("projectile-b");
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-a".to_owned())
        );
        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-b".to_owned())
        );
        assert_eq!(pools.acquire(&scene, "projectiles"), None);
        assert_eq!(
            pools.active_members("projectiles"),
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()]
        );
        assert_eq!(pools.active_count("projectiles"), 2);
    }

    #[test]
    fn entity_pool_release_deactivates_and_reuses_member_without_moving_it() {
        let scene = SceneService::default();
        let transform = Transform3 {
            translation: Vec3::new(4.0, 5.0, 0.0),
            ..Transform3::default()
        };
        scene.spawn_with_transform("projectile-a", transform);
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned()],
        ));

        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-a".to_owned())
        );
        assert!(pools.release(&scene, "projectiles", "projectile-a"));

        assert!(!scene.is_visible("projectile-a"));
        assert!(!scene.is_simulation_enabled("projectile-a"));
        assert!(!scene.is_collision_enabled("projectile-a"));
        assert_eq!(scene.transform_of("projectile-a"), Some(transform));
        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-a".to_owned())
        );
    }

    #[test]
    fn entity_pool_release_all_deactivates_every_active_member() {
        let scene = SceneService::default();
        scene.spawn("projectile-a");
        scene.spawn("projectile-b");
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert!(pools.acquire(&scene, "projectiles").is_some());
        assert!(pools.acquire(&scene, "projectiles").is_some());
        assert_eq!(pools.release_all(&scene, "projectiles"), 2);

        assert_eq!(pools.active_count("projectiles"), 0);
        assert!(!scene.is_visible("projectile-a"));
        assert!(!scene.is_visible("projectile-b"));
        assert!(!scene.is_simulation_enabled("projectile-a"));
        assert!(!scene.is_collision_enabled("projectile-b"));
    }

    #[test]
    fn lifetime_service_expires_after_tick() {
        let lifetimes = LifetimeSceneService::default();
        lifetimes.queue(LifetimeSceneCommand::new(
            "test",
            "projectile-a",
            0.25,
            LifetimeExpirationOutcome::Hide,
        ));

        assert!(lifetimes.tick(0.1).is_empty());
        let expired = lifetimes.tick(0.2);

        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].entity_name, "projectile-a");
        assert!(lifetimes.lifetime("projectile-a").is_none());
    }

    #[test]
    fn entity_pool_exposes_members_for_selector_use() {
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert_eq!(
            pools.members("projectiles"),
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()]
        );
    }

