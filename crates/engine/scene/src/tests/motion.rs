    
    

    use crate::{
        MotionController2dSceneCommand,
        SceneCommand, SceneEvent, format_scene_command,
    };
    
    

    #[test]
    fn motion_controller_commands_use_canonical_surface() {
        let command = MotionController2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-player",
            180.0,
            900.0,
            1200.0,
            500.0,
            900.0,
            -360.0,
            720.0,
        );

        let motion_command = SceneCommand::queue_motion_controller(command.clone());

        assert!(motion_command.is_motion_controller_command());
        assert_eq!(
            motion_command
                .motion_controller_command()
                .expect("motion command should be available")
                .entity_name,
            "playground-sidescroller-player"
        );
        assert_eq!(
            format_scene_command(&motion_command),
            "scene.2d.motion(playground-sidescroller-player, max_speed=180, jump_velocity=-360)"
        );
    }

    #[test]
    fn motion_controller_events_use_canonical_lookup() {
        let motion_event = SceneEvent::motion_controller_queued(7, "player");

        assert_eq!(motion_event.motion_controller_entity_name(), Some("player"));
    }
