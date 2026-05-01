use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

use crate::math::{move_towards, signed_curve_response};

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightMotionProfile2d {
    pub thrust_acceleration: f32,
    pub reverse_acceleration: f32,
    pub strafe_acceleration: f32,
    pub turn_acceleration: f32,
    pub linear_damping: f32,
    pub turn_damping: f32,
    pub max_speed: f32,
    pub max_angular_speed: f32,
    pub thrust_response_curve: amigo_math::Curve1d,
    pub reverse_response_curve: amigo_math::Curve1d,
    pub strafe_response_curve: amigo_math::Curve1d,
    pub turn_response_curve: amigo_math::Curve1d,
}

impl FreeflightMotionProfile2d {
    pub fn response_curves_linear() -> (
        amigo_math::Curve1d,
        amigo_math::Curve1d,
        amigo_math::Curve1d,
        amigo_math::Curve1d,
    ) {
        (
            amigo_math::Curve1d::Linear,
            amigo_math::Curve1d::Linear,
            amigo_math::Curve1d::Linear,
            amigo_math::Curve1d::Linear,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FreeflightMotionIntent2d {
    pub thrust: f32,
    pub strafe: f32,
    pub turn: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreeflightMotionState2d {
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub rotation_radians: f32,
}

impl Default for FreeflightMotionState2d {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            rotation_radians: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreeflightMotionStep2d {
    pub state: FreeflightMotionState2d,
    pub translation_delta: Vec2,
    pub rotation_delta: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightMotion2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub profile: FreeflightMotionProfile2d,
    pub initial_state: FreeflightMotionState2d,
}

pub fn step_freeflight_motion_2d(
    profile: &FreeflightMotionProfile2d,
    intent: &FreeflightMotionIntent2d,
    state: FreeflightMotionState2d,
    delta_seconds: f32,
) -> FreeflightMotionStep2d {
    let thrust_input = intent.thrust.clamp(-1.0, 1.0);
    let strafe_input = intent.strafe.clamp(-1.0, 1.0);
    let turn_input = intent.turn.clamp(-1.0, 1.0);
    let thrust = signed_curve_response(
        thrust_input,
        if thrust_input >= 0.0 {
            &profile.thrust_response_curve
        } else {
            &profile.reverse_response_curve
        },
    );
    let strafe = signed_curve_response(strafe_input, &profile.strafe_response_curve);
    let turn = signed_curve_response(turn_input, &profile.turn_response_curve);
    let forward = Vec2::new(state.rotation_radians.cos(), state.rotation_radians.sin());
    let right = Vec2::new(-state.rotation_radians.sin(), state.rotation_radians.cos());
    let thrust_acceleration = if thrust_input >= 0.0 {
        profile.thrust_acceleration
    } else {
        profile.reverse_acceleration
    };

    let mut velocity = Vec2::new(
        state.velocity.x
            + (forward.x * thrust * thrust_acceleration
                + right.x * strafe * profile.strafe_acceleration)
                * delta_seconds,
        state.velocity.y
            + (forward.y * thrust * thrust_acceleration
                + right.y * strafe * profile.strafe_acceleration)
                * delta_seconds,
    );

    if thrust.abs() <= 0.01 && strafe.abs() <= 0.01 {
        velocity.x = move_towards(velocity.x, 0.0, profile.linear_damping * delta_seconds);
        velocity.y = move_towards(velocity.y, 0.0, profile.linear_damping * delta_seconds);
    }
    velocity = clamp_vec2_length(velocity, profile.max_speed);

    let mut angular_velocity =
        state.angular_velocity + turn * profile.turn_acceleration * delta_seconds;
    if turn.abs() <= 0.01 {
        angular_velocity =
            move_towards(angular_velocity, 0.0, profile.turn_damping * delta_seconds);
    }
    angular_velocity = angular_velocity.clamp(
        -profile.max_angular_speed.abs(),
        profile.max_angular_speed.abs(),
    );

    let rotation_delta = angular_velocity * delta_seconds;
    let translation_delta = Vec2::new(velocity.x * delta_seconds, velocity.y * delta_seconds);
    FreeflightMotionStep2d {
        state: FreeflightMotionState2d {
            velocity,
            angular_velocity,
            rotation_radians: state.rotation_radians + rotation_delta,
        },
        translation_delta,
        rotation_delta,
    }
}

fn clamp_vec2_length(value: Vec2, max_length: f32) -> Vec2 {
    let max_length = max_length.max(0.0);
    let length_squared = value.x * value.x + value.y * value.y;
    if max_length <= 0.0 || length_squared <= max_length * max_length {
        return value;
    }
    let length = length_squared.sqrt();
    Vec2::new(value.x / length * max_length, value.y / length * max_length)
}
