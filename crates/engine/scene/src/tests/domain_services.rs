    
    

    use crate::{
        SceneService,
    };
    
    use amigo_math::{Transform3, Vec2, Vec3};    #[test]
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



