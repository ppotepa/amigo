use super::common::test_freeflight_profile;
use crate::{FreeflightMotionIntent2d, FreeflightMotionState2d, step_freeflight_motion_2d};
use amigo_math::{Curve1d, Vec2};

#[test]
fn freeflight_thrust_accelerates_along_facing() {
    let step = step_freeflight_motion_2d(
        &test_freeflight_profile(),
        &FreeflightMotionIntent2d {
            thrust: 1.0,
            ..Default::default()
        },
        FreeflightMotionState2d::default(),
        0.5,
    );

    assert_eq!(step.state.velocity, Vec2::new(10.0, 0.0));
    assert_eq!(step.translation_delta, Vec2::new(5.0, 0.0));
}

#[test]
fn freeflight_thrust_uses_response_curve() {
    let mut profile = test_freeflight_profile();
    profile.thrust_response_curve = Curve1d::Constant(0.5);

    let step = step_freeflight_motion_2d(
        &profile,
        &FreeflightMotionIntent2d {
            thrust: 1.0,
            ..Default::default()
        },
        FreeflightMotionState2d::default(),
        1.0,
    );

    assert_eq!(step.state.velocity, Vec2::new(10.0, 0.0));
}

#[test]
fn freeflight_reverse_uses_reverse_response_curve() {
    let mut profile = test_freeflight_profile();
    profile.reverse_response_curve = Curve1d::Constant(0.5);

    let step = step_freeflight_motion_2d(
        &profile,
        &FreeflightMotionIntent2d {
            thrust: -1.0,
            ..Default::default()
        },
        FreeflightMotionState2d::default(),
        1.0,
    );

    assert_eq!(step.state.velocity, Vec2::new(-5.0, 0.0));
}

#[test]
fn freeflight_turn_uses_turn_response_curve() {
    let mut profile = test_freeflight_profile();
    profile.turn_response_curve = Curve1d::Constant(0.25);

    let step = step_freeflight_motion_2d(
        &profile,
        &FreeflightMotionIntent2d {
            turn: 1.0,
            ..Default::default()
        },
        FreeflightMotionState2d::default(),
        1.0,
    );

    assert_eq!(step.state.angular_velocity, 2.0);
}

#[test]
fn freeflight_turn_damping_reduces_angular_velocity_without_turn_intent() {
    let step = step_freeflight_motion_2d(
        &test_freeflight_profile(),
        &FreeflightMotionIntent2d::default(),
        FreeflightMotionState2d {
            angular_velocity: 3.0,
            ..Default::default()
        },
        0.25,
    );

    assert_eq!(step.state.angular_velocity, 2.0);
    assert_eq!(step.rotation_delta, 0.5);
}

#[test]
fn freeflight_speed_is_clamped_to_profile_max_speed() {
    let step = step_freeflight_motion_2d(
        &test_freeflight_profile(),
        &FreeflightMotionIntent2d {
            thrust: 1.0,
            ..Default::default()
        },
        FreeflightMotionState2d {
            velocity: Vec2::new(25.0, 0.0),
            ..Default::default()
        },
        1.0,
    );

    assert_eq!(step.state.velocity, Vec2::new(30.0, 0.0));
}
