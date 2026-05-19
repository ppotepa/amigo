use super::common::test_freeflight_profile;
use crate::{
    FreeflightMotion2dCommand, FreeflightMotionIntent2d, FreeflightMotionState2d,
    Motion2dSceneService,
};
use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

#[test]
fn service_reset_freeflight_clears_motion_state_and_intent() {
    let service = Motion2dSceneService::default();
    service.queue_freeflight(FreeflightMotion2dCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "ship".to_owned(),
        profile: test_freeflight_profile(),
        initial_state: FreeflightMotionState2d {
            velocity: Vec2::new(12.0, 3.0),
            angular_velocity: 2.0,
            rotation_radians: 1.5,
        },
    });
    assert!(service.drive_freeflight(
        "ship",
        FreeflightMotionIntent2d {
            thrust: 1.0,
            turn: -1.0,
            ..Default::default()
        }
    ));

    assert!(service.reset_freeflight("ship", 0.25));

    assert_eq!(
        service.freeflight_state("ship"),
        Some(FreeflightMotionState2d {
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            rotation_radians: 0.25,
        })
    );
    assert_eq!(
        service.freeflight_intent("ship"),
        Some(FreeflightMotionIntent2d::default())
    );
}
