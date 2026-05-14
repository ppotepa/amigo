use crate::{Bounds2d, BoundsBehavior2d, FreeflightMotionProfile2d};
use amigo_math::{Curve1d, Vec2};

pub fn test_bounds(behavior: BoundsBehavior2d) -> Bounds2d {
    Bounds2d {
        min: Vec2::new(-10.0, -5.0),
        max: Vec2::new(10.0, 5.0),
        behavior,
    }
}

pub fn test_freeflight_profile() -> FreeflightMotionProfile2d {
    FreeflightMotionProfile2d {
        thrust_acceleration: 20.0,
        reverse_acceleration: 10.0,
        strafe_acceleration: 12.0,
        turn_acceleration: 8.0,
        linear_damping: 2.0,
        turn_damping: 4.0,
        max_speed: 30.0,
        max_angular_speed: 6.0,
        thrust_response_curve: Curve1d::Linear,
        reverse_response_curve: Curve1d::Linear,
        strafe_response_curve: Curve1d::Linear,
        turn_response_curve: Curve1d::Linear,
    }
}
