use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_2d_physics::PhysicsBodyState2d;
use amigo_math::Vec2;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformerFacing {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformerAnimationState {
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerControllerParams {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub air_acceleration: f32,
    pub gravity: f32,
    pub jump_velocity: f32,
    pub terminal_velocity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerControllerState {
    pub grounded: bool,
    pub facing: PlatformerFacing,
    pub animation: PlatformerAnimationState,
    pub velocity: Vec2,
}

impl Default for PlatformerControllerState {
    fn default() -> Self {
        Self {
            grounded: false,
            facing: PlatformerFacing::Right,
            animation: PlatformerAnimationState::Idle,
            velocity: Vec2::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlatformerMotor2d {
    pub move_x: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerController2d {
    pub params: PlatformerControllerParams,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerController2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub controller: PlatformerController2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerDriveResult {
    pub velocity: Vec2,
    pub facing: PlatformerFacing,
    pub jumped: bool,
}

#[derive(Debug, Default)]
struct PlatformerState {
    commands: BTreeMap<String, PlatformerController2dCommand>,
    states: BTreeMap<String, PlatformerControllerState>,
    motors: BTreeMap<String, PlatformerMotor2d>,
}

#[derive(Debug, Default)]
pub struct PlatformerSceneService {
    state: Mutex<PlatformerState>,
}

impl PlatformerSceneService {
    pub fn queue(&self, command: PlatformerController2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("platformer scene service mutex should not be poisoned");
        state
            .states
            .entry(command.entity_name.clone())
            .or_insert_with(PlatformerControllerState::default);
        state
            .motors
            .entry(command.entity_name.clone())
            .or_insert_with(PlatformerMotor2d::default);
        state.commands.insert(command.entity_name.clone(), command);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("platformer scene service mutex should not be poisoned");
        state.commands.clear();
        state.states.clear();
        state.motors.clear();
    }

    pub fn commands(&self) -> Vec<PlatformerController2dCommand> {
        self.state
            .lock()
            .expect("platformer scene service mutex should not be poisoned")
            .commands
            .values()
            .cloned()
            .collect()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }

    pub fn controller(&self, entity_name: &str) -> Option<PlatformerController2dCommand> {
        self.state
            .lock()
            .expect("platformer scene service mutex should not be poisoned")
            .commands
            .get(entity_name)
            .cloned()
    }

    pub fn drive(
        &self,
        entity_name: &str,
        move_x: f32,
        jump_pressed: bool,
        jump_held: bool,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("platformer scene service mutex should not be poisoned");
        if !state.commands.contains_key(entity_name) {
            return false;
        }
        state.motors.insert(
            entity_name.to_owned(),
            PlatformerMotor2d {
                move_x: move_x.clamp(-1.0, 1.0),
                jump_pressed,
                jump_held,
            },
        );
        true
    }

    pub fn motor(&self, entity_name: &str) -> Option<PlatformerMotor2d> {
        self.state
            .lock()
            .expect("platformer scene service mutex should not be poisoned")
            .motors
            .get(entity_name)
            .cloned()
    }

    pub fn clear_motor(&self, entity_name: &str) {
        self.state
            .lock()
            .expect("platformer scene service mutex should not be poisoned")
            .motors
            .insert(entity_name.to_owned(), PlatformerMotor2d::default());
    }

    pub fn state(&self, entity_name: &str) -> Option<PlatformerControllerState> {
        self.state
            .lock()
            .expect("platformer scene service mutex should not be poisoned")
            .states
            .get(entity_name)
            .cloned()
    }

    pub fn sync_state(&self, entity_name: &str, state_value: PlatformerControllerState) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("platformer scene service mutex should not be poisoned");
        if !state.commands.contains_key(entity_name) {
            return false;
        }
        state.states.insert(entity_name.to_owned(), state_value);
        true
    }
}

#[derive(Debug, Clone)]
pub struct PlatformerDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

#[derive(Debug, Clone)]
pub struct Motion2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct PlatformerPlugin;

pub type Facing2d = PlatformerFacing;
pub type MotionAnimationState = PlatformerAnimationState;
pub type MotionProfile2d = PlatformerControllerParams;
pub type MotionState2d = PlatformerControllerState;
pub type MotionIntent2d = PlatformerMotor2d;
pub type MotionController2d = PlatformerController2d;
pub type MotionController2dCommand = PlatformerController2dCommand;
pub type MotionDriveResult = PlatformerDriveResult;
pub type Motion2dSceneService = PlatformerSceneService;
pub type Motion2dPlugin = PlatformerPlugin;

impl RuntimePlugin for PlatformerPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-platformer"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(PlatformerSceneService::default())?;
        registry.register(PlatformerDomainInfo {
            crate_name: "amigo-2d-platformer",
            capability: "platformer_2d",
        })?;
        registry.register(Motion2dDomainInfo {
            crate_name: "amigo-2d-platformer",
            capability: "motion_2d",
        })
    }
}

pub fn drive_motion_2d(
    params: &MotionProfile2d,
    body_state: &PhysicsBodyState2d,
    motor: &MotionIntent2d,
    facing: Facing2d,
    delta_seconds: f32,
) -> MotionDriveResult {
    drive_controller(params, body_state, motor, facing, delta_seconds)
}

pub fn motion_animation_state_for(
    velocity: Vec2,
    grounded: bool,
) -> MotionAnimationState {
    animation_state_for(velocity, grounded)
}

pub fn drive_controller(
    params: &PlatformerControllerParams,
    body_state: &PhysicsBodyState2d,
    motor: &PlatformerMotor2d,
    facing: PlatformerFacing,
    delta_seconds: f32,
) -> PlatformerDriveResult {
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
            PlatformerFacing::Left
        } else {
            PlatformerFacing::Right
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

    PlatformerDriveResult {
        velocity,
        facing: next_facing,
        jumped,
    }
}

pub fn animation_state_for(velocity: Vec2, grounded: bool) -> PlatformerAnimationState {
    if !grounded {
        if velocity.y >= 0.0 {
            PlatformerAnimationState::Jump
        } else {
            PlatformerAnimationState::Fall
        }
    } else if velocity.x.abs() > 1.0 {
        PlatformerAnimationState::Run
    } else {
        PlatformerAnimationState::Idle
    }
}

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlatformerAnimationState, PlatformerController2d, PlatformerController2dCommand,
        PlatformerControllerParams, PlatformerFacing, PlatformerMotor2d, PlatformerSceneService,
        animation_state_for, drive_controller,
    };
    use amigo_2d_physics::{GroundedState, PhysicsBodyState2d};
    use amigo_math::Vec2;
    use amigo_scene::SceneEntityId;

    fn params() -> PlatformerControllerParams {
        PlatformerControllerParams {
            max_speed: 180.0,
            acceleration: 900.0,
            deceleration: 1200.0,
            air_acceleration: 500.0,
            gravity: 900.0,
            jump_velocity: -360.0,
            terminal_velocity: 720.0,
        }
    }

    #[test]
    fn stores_platformer_controller_commands() {
        let service = PlatformerSceneService::default();

        service.queue(PlatformerController2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-player".to_owned(),
            controller: PlatformerController2d { params: params() },
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-sidescroller-player".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }

    #[test]
    fn right_input_accelerates_towards_max_speed() {
        let result = drive_controller(
            &params(),
            &PhysicsBodyState2d {
                velocity: Vec2::ZERO,
                grounded: GroundedState {
                    grounded: true,
                    ..GroundedState::default()
                },
            },
            &PlatformerMotor2d {
                move_x: 1.0,
                jump_pressed: false,
                jump_held: false,
            },
            PlatformerFacing::Right,
            1.0 / 60.0,
        );

        assert!(result.velocity.x > 0.0);
        assert_eq!(result.facing, PlatformerFacing::Right);
    }

    #[test]
    fn grounded_jump_sets_upward_velocity() {
        let result = drive_controller(
            &params(),
            &PhysicsBodyState2d {
                velocity: Vec2::ZERO,
                grounded: GroundedState {
                    grounded: true,
                    ..GroundedState::default()
                },
            },
            &PlatformerMotor2d {
                move_x: 0.0,
                jump_pressed: true,
                jump_held: true,
            },
            PlatformerFacing::Right,
            1.0 / 60.0,
        );

        assert!(result.jumped);
        assert!(result.velocity.y > 0.0);
    }

    #[test]
    fn animation_state_uses_ground_and_velocity() {
        assert_eq!(
            animation_state_for(Vec2::ZERO, true),
            PlatformerAnimationState::Idle
        );
        assert_eq!(
            animation_state_for(Vec2::new(12.0, 0.0), true),
            PlatformerAnimationState::Run
        );
        assert_eq!(
            animation_state_for(Vec2::new(0.0, 120.0), false),
            PlatformerAnimationState::Jump
        );
        assert_eq!(
            animation_state_for(Vec2::new(0.0, -80.0), false),
            PlatformerAnimationState::Fall
        );
    }
}
