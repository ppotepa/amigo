    
    

    use crate::{
        CameraFollow2dSceneCommand, CameraFollow2dSceneService, Parallax2dSceneCommand, Parallax2dSceneService, SceneService,
    };
    
    use amigo_math::{Transform3, Vec2, Vec3};

    #[test]
    fn camera_follow_scene_service_replaces_existing_entity_binding() {
        let service = CameraFollow2dSceneService::default();
        service.queue(CameraFollow2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-camera",
            "playground-sidescroller-player",
            Vec2::new(0.0, 0.0),
            1.0,
        ));
        service.queue(CameraFollow2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-camera",
            "playground-sidescroller-finish",
            Vec2::new(16.0, 0.0),
            0.5,
        ));

        let binding = service
            .follow("playground-sidescroller-camera")
            .expect("camera follow binding should exist");

        assert_eq!(service.commands().len(), 1);
        assert_eq!(binding.target, "playground-sidescroller-finish");
        assert_eq!(binding.offset, Vec2::new(16.0, 0.0));
        assert_eq!(binding.lerp, 0.5);
    }

    #[test]
    fn parallax_scene_service_tracks_camera_origin() {
        let service = Parallax2dSceneService::default();
        service.queue(Parallax2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-background-far",
            "playground-sidescroller-camera",
            Vec2::new(0.12, 0.04),
            Vec2::new(512.0, 256.0),
        ));

        assert!(service.set_camera_origin(
            "playground-sidescroller-background-far",
            Vec2::new(64.0, 64.0),
        ));

        let binding = service.commands();
        assert_eq!(binding.len(), 1);
        assert_eq!(binding[0].camera_origin, Some(Vec2::new(64.0, 64.0)));
        assert_eq!(
            service.entity_names(),
            vec!["playground-sidescroller-background-far".to_owned()]
        );
    }

    #[test]
    fn scene_service_can_rotate_2d_entity_by_name() {
        let scene = SceneService::default();
        scene.spawn_with_transform("square", Transform3::default());

        assert!(scene.rotate_entity_2d("square", 1.0));

        let transform = scene.transform_of("square").expect("entity should exist");
        assert_eq!(transform.rotation_euler.z, 1.0);
    }

    #[test]
    fn scene_service_can_rotate_3d_entity_by_name() {
        let scene = SceneService::default();
        scene.spawn_with_transform("cube", Transform3::default());

        assert!(scene.rotate_entity_3d("cube", Vec3::new(1.0, 2.0, 0.0)));

        let transform = scene.transform_of("cube").expect("entity should exist");
        assert_eq!(transform.rotation_euler.x, 1.0);
        assert_eq!(transform.rotation_euler.y, 2.0);
    }

