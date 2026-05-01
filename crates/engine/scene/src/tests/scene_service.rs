    use std::collections::BTreeMap;
    

    use crate::{
        SceneEntityLifecycle,
        SceneKey, ScenePropertyValue, SceneService,
    };
    
    use amigo_math::{Transform3, Vec3};

    #[test]
    fn scene_service_spawns_entities_in_order() {
        let scene = SceneService::default();

        let first = scene.spawn("core-root");
        let second = scene.spawn("playground-2d-camera");

        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        assert_eq!(scene.entity_count(), 2);
        assert_eq!(
            scene.entity_names(),
            vec!["core-root".to_owned(), "playground-2d-camera".to_owned()]
        );
    }

    #[test]
    fn scene_service_tracks_selected_scene() {
        let scene = SceneService::default();

        scene.select_scene(SceneKey::new("dev-shell"));

        assert_eq!(
            scene
                .selected_scene()
                .expect("selected scene should exist")
                .as_str(),
            "dev-shell"
        );
    }

    #[test]
    fn scene_service_can_find_entity_by_name() {
        let scene = SceneService::default();

        scene.spawn("playground-3d-probe");

        assert_eq!(
            scene
                .entity_by_name("playground-3d-probe")
                .expect("entity should exist")
                .name,
            "playground-3d-probe"
        );
    }

    #[test]
    fn scene_service_can_spawn_entity_with_transform() {
        let scene = SceneService::default();

        scene.spawn_with_transform(
            "playground-2d-sprite",
            Transform3 {
                translation: Vec3::new(8.0, -2.0, 0.0),
                rotation_euler: Vec3::new(0.0, 0.0, 0.25),
                scale: Vec3::new(2.0, 2.0, 1.0),
            },
        );

        assert_eq!(
            scene
                .entity_by_name("playground-2d-sprite")
                .expect("entity should exist")
                .transform
                .translation,
            Vec3::new(8.0, -2.0, 0.0)
        );
    }

    #[test]
    fn scene_service_tracks_lifecycle_without_mutating_transform() {
        let scene = SceneService::default();
        let transform = Transform3 {
            translation: Vec3::new(8.0, -2.0, 0.0),
            rotation_euler: Vec3::new(0.0, 0.0, 0.25),
            scale: Vec3::new(2.0, 2.0, 1.0),
        };
        scene.spawn_with_transform("actor", transform);

        assert!(scene.set_visible("actor", false));
        assert!(scene.set_simulation_enabled("actor", false));
        assert!(scene.set_collision_enabled("actor", false));

        assert!(!scene.is_visible("actor"));
        assert!(!scene.is_simulation_enabled("actor"));
        assert!(!scene.is_collision_enabled("actor"));
        assert_eq!(scene.transform_of("actor"), Some(transform));
    }

    #[test]
    fn scene_service_tracks_entity_metadata_queries() {
        let scene = SceneService::default();
        scene.spawn("actor");

        assert!(scene.configure_entity_metadata(
            "actor",
            SceneEntityLifecycle::default(),
            vec!["enemy".to_owned(), "flying".to_owned()],
            vec!["wave-1".to_owned()],
            BTreeMap::from([
                ("score_value".to_owned(), ScenePropertyValue::Int(100)),
                (
                    "label".to_owned(),
                    ScenePropertyValue::String("scout".to_owned())
                ),
            ]),
        ));

        assert!(scene.has_tag("actor", "enemy"));
        assert!(scene.has_group("actor", "wave-1"));
        assert_eq!(scene.entities_by_tag("enemy"), vec!["actor".to_owned()]);
        assert_eq!(scene.entities_by_group("wave-1"), vec!["actor".to_owned()]);
        assert_eq!(
            scene.property_of("actor", "score_value"),
            Some(ScenePropertyValue::Int(100))
        );

        assert!(scene.set_property("actor", "score_value", ScenePropertyValue::Int(250)));
        assert_eq!(
            scene.property_of("actor", "score_value"),
            Some(ScenePropertyValue::Int(250))
        );
    }

    #[test]
    fn scene_service_can_remove_entities_by_name() {
        let scene = SceneService::default();
        scene.spawn("core-root");
        scene.spawn("playground-2d-camera");
        scene.spawn("playground-2d-sprite");

        let removed = scene.remove_entities_by_name(&[
            "playground-2d-camera".to_owned(),
            "playground-2d-sprite".to_owned(),
        ]);

        assert_eq!(removed, 2);
        assert_eq!(scene.entity_names(), vec!["core-root".to_owned()]);
    }

