use amigo_2d_physics::PhysicsBodyState2d;
use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

use crate::math::move_towards;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing2d {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionAnimationState {
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionProfile2d {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub air_acceleration: f32,
    pub gravity: f32,
    pub jump_velocity: f32,
    pub terminal_velocity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionState2d {
    pub grounded: bool,
    pub facing: Facing2d,
    pub animation: MotionAnimationState,
    pub velocity: Vec2,
}

impl Default for MotionState2d {
    fn default() -> Self {
        Self {
            grounded: false,
            facing: Facing2d::Right,
            animation: MotionAnimationState::Idle,
            velocity: Vec2::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionIntent2d {
    pub move_x: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionController2d {
    pub params: MotionProfile2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionController2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub controller: MotionController2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionDriveResult {
    pub velocity: Vec2,
    pub facing: Facing2d,
    pub jumped: bool,
}

pub fn drive_motion_2d(
    params: &MotionProfile2d,
    body_state: &PhysicsBodyState2d,
    motor: &MotionIntent2d,
    facing: Facing2d,
    delta_seconds: f32,
) -> MotionDriveResult {
    let mut velocity = body_state.velocity;
    let grounded = body_state.grounded.grounded;
    let move_x = motor.move_x.clamp(-1.0, 1.0);
    let mut next_facing = facing;

    if move_x.abs() > 0.01 {
        let target = move_x * params.max_speed;
        let acceleration = if grounded {
            params.acceleration
        } else {
            params.air_acceleration
        };
        velocity.x = move_towards(velocity.x, target, acceleration * delta_seconds);
        next_facing = if move_x < 0.0 {
            Facing2d::Left
        } else {
            Facing2d::Right
        };
    } else {
        let deceleration = if grounded {
            params.deceleration
        } else {
            params.air_acceleration
        };
        velocity.x = move_towards(velocity.x, 0.0, deceleration * delta_seconds);
    }

    velocity.y += -params.gravity.abs() * delta_seconds;
    velocity.y = velocity.y.max(-params.terminal_velocity.abs());

    let mut jumped = false;
    if motor.jump_pressed && grounded {
        velocity.y = params.jump_velocity.abs();
        jumped = true;
    }

    MotionDriveResult {
        velocity,
        facing: next_facing,
        jumped,
    }
}

pub fn motion_animation_state_for(velocity: Vec2, grounded: bool) -> MotionAnimationState {
    if !grounded {
        if velocity.y >= 0.0 {
            MotionAnimationState::Jump
        } else {
            MotionAnimationState::Fall
        }
    } else if velocity.x.abs() > 1.0 {
        MotionAnimationState::Run
    } else {
        MotionAnimationState::Idle
    }
}

pub fn motion_facing_to_str(facing: Facing2d) -> &'static str {
    match facing {
        Facing2d::Left => "left",
        Facing2d::Right => "right",
    }
}

pub fn drive_controller(
    params: &MotionProfile2d,
    body_state: &PhysicsBodyState2d,
    motor: &MotionIntent2d,
    facing: Facing2d,
    delta_seconds: f32,
) -> MotionDriveResult {
    drive_motion_2d(params, body_state, motor, facing, delta_seconds)
}

pub fn animation_state_for(velocity: Vec2, grounded: bool) -> MotionAnimationState {
    motion_animation_state_for(velocity, grounded)
}
