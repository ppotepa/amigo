use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_2d_physics::{Physics2dSceneService, PhysicsBodyState2d};
use amigo_math::{Transform3, Vec2, Vec3};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{EntityPoolSceneService, SceneEntityId, SceneService};

pub const CANONICAL_MOTION_2D_PLUGIN_LABEL: &str = "amigo-2d-motion";
pub const CANONICAL_MOTION_2D_CAPABILITY: &str = "motion_2d";
pub const CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL: &str = "motion_2d via amigo-2d-motion";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity2d {
    pub linear: Vec2,
}

impl Default for Velocity2d {
    fn default() -> Self {
        Self { linear: Vec2::ZERO }
    }
}

impl Velocity2d {
    pub const fn new(linear: Vec2) -> Self {
        Self { linear }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds2d {
    pub min: Vec2,
    pub max: Vec2,
    pub behavior: BoundsBehavior2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsBehavior2d {
    Bounce { restitution: f32 },
    Wrap,
    Hide,
    Despawn,
    Clamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoundsContact2d {
    pub min_x: bool,
    pub max_x: bool,
    pub min_y: bool,
    pub max_y: bool,
}

impl BoundsContact2d {
    pub const fn any(self) -> bool {
        self.min_x || self.max_x || self.min_y || self.max_y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsOutcome2d {
    None,
    Bounced { contact: BoundsContact2d },
    Wrapped { contact: BoundsContact2d },
    Clamped { contact: BoundsContact2d },
    Hidden { contact: BoundsContact2d },
    Despawned { contact: BoundsContact2d },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundsApplyResult2d {
    pub translation: Vec2,
    pub velocity: Vec2,
    pub outcome: BoundsOutcome2d,
}

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
pub struct Velocity2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub velocity: Velocity2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub bounds: Bounds2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightMotion2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub profile: FreeflightMotionProfile2d,
    pub initial_state: FreeflightMotionState2d,
}

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
pub struct ProjectileEmitter2d {
    pub pool: String,
    pub speed: f32,
    pub spawn_offset: Vec2,
    pub inherit_velocity_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileEmitter2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub emitter: ProjectileEmitter2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileLaunch2d {
    pub transform: Transform3,
    pub velocity: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionDriveResult {
    pub velocity: Vec2,
    pub facing: Facing2d,
    pub jumped: bool,
}

#[derive(Debug, Default)]
struct MotionStateRegistry {
    commands: BTreeMap<String, MotionController2dCommand>,
    states: BTreeMap<String, MotionState2d>,
    motors: BTreeMap<String, MotionIntent2d>,
    velocities: BTreeMap<String, Velocity2dCommand>,
    bounds: BTreeMap<String, Bounds2dCommand>,
    freeflight_commands: BTreeMap<String, FreeflightMotion2dCommand>,
    freeflight_states: BTreeMap<String, FreeflightMotionState2d>,
    freeflight_intents: BTreeMap<String, FreeflightMotionIntent2d>,
    projectile_emitters: BTreeMap<String, ProjectileEmitter2dCommand>,
}

#[derive(Debug, Default)]
pub struct Motion2dSceneService {
    state: Mutex<MotionStateRegistry>,
}

impl Motion2dSceneService {
    pub fn queue_motion_controller(&self, command: MotionController2dCommand) {
        self.queue(command);
    }

    pub fn queue_projectile_emitter(&self, command: ProjectileEmitter2dCommand) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .projectile_emitters
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_velocity(&self, command: Velocity2dCommand) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .velocities
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_bounds(&self, command: Bounds2dCommand) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .bounds
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_freeflight(&self, command: FreeflightMotion2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state
            .freeflight_states
            .insert(command.entity_name.clone(), command.initial_state);
        state
            .freeflight_intents
            .entry(command.entity_name.clone())
            .or_insert_with(FreeflightMotionIntent2d::default);
        state
            .freeflight_commands
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue(&self, command: MotionController2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state
            .states
            .entry(command.entity_name.clone())
            .or_insert_with(MotionState2d::default);
        state
            .motors
            .entry(command.entity_name.clone())
            .or_insert_with(MotionIntent2d::default);
        state.commands.insert(command.entity_name.clone(), command);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state.commands.clear();
        state.states.clear();
        state.motors.clear();
        state.velocities.clear();
        state.bounds.clear();
        state.freeflight_commands.clear();
        state.freeflight_states.clear();
        state.freeflight_intents.clear();
        state.projectile_emitters.clear();
    }

    pub fn commands(&self) -> Vec<MotionController2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .commands
            .values()
            .cloned()
            .collect()
    }

    pub fn motion_controller_commands(&self) -> Vec<MotionController2dCommand> {
        self.commands()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }

    pub fn motion_entity_names(&self) -> Vec<String> {
        self.entity_names()
    }

    pub fn projectile_emitter(&self, entity_name: &str) -> Option<ProjectileEmitter2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .projectile_emitters
            .get(entity_name)
            .cloned()
    }

    pub fn velocities(&self) -> Vec<Velocity2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .velocities
            .values()
            .cloned()
            .collect()
    }

    pub fn velocity(&self, entity_name: &str) -> Option<Velocity2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .velocities
            .get(entity_name)
            .map(|command| command.velocity)
    }

    pub fn set_velocity(&self, entity_name: &str, velocity: Vec2) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state
            .velocities
            .entry(entity_name.to_owned())
            .and_modify(|command| command.velocity = Velocity2d::new(velocity))
            .or_insert_with(|| Velocity2dCommand {
                entity_id: SceneEntityId::new(0),
                entity_name: entity_name.to_owned(),
                velocity: Velocity2d::new(velocity),
            });
        true
    }

    pub fn bounds(&self) -> Vec<Bounds2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .bounds
            .values()
            .cloned()
            .collect()
    }

    pub fn bounds_for(&self, entity_name: &str) -> Option<Bounds2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .bounds
            .get(entity_name)
            .map(|command| command.bounds)
    }

    pub fn freeflight_commands(&self) -> Vec<FreeflightMotion2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_commands
            .values()
            .cloned()
            .collect()
    }

    pub fn freeflight_command(&self, entity_name: &str) -> Option<FreeflightMotion2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_commands
            .get(entity_name)
            .cloned()
    }

    pub fn freeflight_state(&self, entity_name: &str) -> Option<FreeflightMotionState2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_states
            .get(entity_name)
            .copied()
    }

    pub fn sync_freeflight_state(
        &self,
        entity_name: &str,
        state_value: FreeflightMotionState2d,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.freeflight_commands.contains_key(entity_name) {
            return false;
        }
        state
            .freeflight_states
            .insert(entity_name.to_owned(), state_value);
        true
    }

    pub fn reset_freeflight(&self, entity_name: &str, rotation_radians: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.freeflight_commands.contains_key(entity_name) {
            return false;
        }
        state.freeflight_states.insert(
            entity_name.to_owned(),
            FreeflightMotionState2d {
                velocity: Vec2::ZERO,
                angular_velocity: 0.0,
                rotation_radians,
            },
        );
        state
            .freeflight_intents
            .insert(entity_name.to_owned(), FreeflightMotionIntent2d::default());
        true
    }

    pub fn drive_freeflight(&self, entity_name: &str, intent: FreeflightMotionIntent2d) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.freeflight_commands.contains_key(entity_name) {
            return false;
        }
        state.freeflight_intents.insert(
            entity_name.to_owned(),
            FreeflightMotionIntent2d {
                thrust: intent.thrust.clamp(-1.0, 1.0),
                strafe: intent.strafe.clamp(-1.0, 1.0),
                turn: intent.turn.clamp(-1.0, 1.0),
            },
        );
        true
    }

    pub fn freeflight_intent(&self, entity_name: &str) -> Option<FreeflightMotionIntent2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_intents
            .get(entity_name)
            .cloned()
    }

    pub fn clear_freeflight_intent(&self, entity_name: &str) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_intents
            .insert(entity_name.to_owned(), FreeflightMotionIntent2d::default());
    }

    pub fn current_velocity(&self, entity_name: &str) -> Vec2 {
        self.freeflight_state(entity_name)
            .map(|state| state.velocity)
            .or_else(|| self.velocity(entity_name).map(|velocity| velocity.linear))
            .unwrap_or(Vec2::ZERO)
    }

    pub fn fire_projectile_from_emitter(
        &self,
        scene_service: &SceneService,
        pool_service: &EntityPoolSceneService,
        physics_scene_service: Option<&Physics2dSceneService>,
        emitter_entity_name: &str,
    ) -> Option<String> {
        let command = self.projectile_emitter(emitter_entity_name)?;
        let source_transform = scene_service.transform_of(emitter_entity_name)?;
        let source_velocity = physics_scene_service
            .and_then(|service| service.body_state(emitter_entity_name))
            .map(|state| state.velocity)
            .unwrap_or_else(|| self.current_velocity(emitter_entity_name));
        let projectile_entity = pool_service.acquire(scene_service, &command.emitter.pool)?;
        let launch = projectile_launch_2d(source_transform, source_velocity, &command.emitter);
        let _ = scene_service.set_transform(&projectile_entity, launch.transform);
        if let Some(physics_scene_service) = physics_scene_service {
            if let Some(mut body_state) = physics_scene_service.body_state(&projectile_entity) {
                body_state.velocity = launch.velocity;
                let _ = physics_scene_service.sync_body_state(&projectile_entity, body_state);
            }
        }
        let _ = self.set_velocity(&projectile_entity, launch.velocity);
        Some(projectile_entity)
    }

    pub fn controller(&self, entity_name: &str) -> Option<MotionController2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .commands
            .get(entity_name)
            .cloned()
    }

    pub fn motion_controller(&self, entity_name: &str) -> Option<MotionController2dCommand> {
        self.controller(entity_name)
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
            .expect("motion scene service mutex should not be poisoned");
        if !state.commands.contains_key(entity_name) {
            return false;
        }
        state.motors.insert(
            entity_name.to_owned(),
            MotionIntent2d {
                move_x: move_x.clamp(-1.0, 1.0),
                jump_pressed,
                jump_held,
            },
        );
        true
    }

    pub fn drive_motion(
        &self,
        entity_name: &str,
        move_x: f32,
        jump_pressed: bool,
        jump_held: bool,
    ) -> bool {
        self.drive(entity_name, move_x, jump_pressed, jump_held)
    }

    pub fn motor(&self, entity_name: &str) -> Option<MotionIntent2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .motors
            .get(entity_name)
            .cloned()
    }

    pub fn motion_intent(&self, entity_name: &str) -> Option<MotionIntent2d> {
        self.motor(entity_name)
    }

    pub fn clear_motor(&self, entity_name: &str) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .motors
            .insert(entity_name.to_owned(), MotionIntent2d::default());
    }

    pub fn clear_motion_intent(&self, entity_name: &str) {
        self.clear_motor(entity_name);
    }

    pub fn state(&self, entity_name: &str) -> Option<MotionState2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .states
            .get(entity_name)
            .cloned()
    }

    pub fn motion_state(&self, entity_name: &str) -> Option<MotionState2d> {
        self.state(entity_name)
    }

    pub fn sync_state(&self, entity_name: &str, state_value: MotionState2d) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.commands.contains_key(entity_name) {
            return false;
        }
        state.states.insert(entity_name.to_owned(), state_value);
        true
    }

    pub fn sync_motion_state(&self, entity_name: &str, state_value: MotionState2d) -> bool {
        self.sync_state(entity_name, state_value)
    }
}

#[derive(Debug, Clone)]
pub struct Motion2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Motion2dPlugin;
pub const MOTION_2D_PLUGIN: Motion2dPlugin = Motion2dPlugin;

pub fn motion_2d_plugin() -> Motion2dPlugin {
    Motion2dPlugin
}

pub fn motion_runtime_plugin_report_label(plugin_name: &str) -> String {
    if plugin_name == CANONICAL_MOTION_2D_PLUGIN_LABEL {
        CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL.to_owned()
    } else {
        plugin_name.to_owned()
    }
}

pub fn motion_2d_domain_info() -> Motion2dDomainInfo {
    Motion2dDomainInfo::canonical()
}

impl Motion2dDomainInfo {
    pub const fn canonical() -> Self {
        Self {
            crate_name: CANONICAL_MOTION_2D_PLUGIN_LABEL,
            capability: CANONICAL_MOTION_2D_CAPABILITY,
        }
    }

    pub const fn canonical_plugin_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_PLUGIN_LABEL
    }

    pub const fn runtime_report_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL
    }
}

impl Motion2dPlugin {
    pub const fn canonical_motion_plugin_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_PLUGIN_LABEL
    }

    pub const fn canonical_motion_capability(&self) -> &'static str {
        CANONICAL_MOTION_2D_CAPABILITY
    }

    pub const fn runtime_report_label(&self) -> &'static str {
        CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL
    }
}

impl RuntimePlugin for Motion2dPlugin {
    fn name(&self) -> &'static str {
        CANONICAL_MOTION_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Motion2dSceneService::default())?;
        registry.register(Motion2dDomainInfo::canonical())
    }
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

pub fn step_velocity_2d(translation: Vec2, velocity: &Velocity2d, delta_seconds: f32) -> Vec2 {
    Vec2::new(
        translation.x + velocity.linear.x * delta_seconds,
        translation.y + velocity.linear.y * delta_seconds,
    )
}

pub fn projectile_launch_2d(
    source_transform: Transform3,
    source_velocity: Vec2,
    emitter: &ProjectileEmitter2d,
) -> ProjectileLaunch2d {
    let rotation = source_transform.rotation_euler.z;
    let forward = Vec2::new(rotation.cos(), rotation.sin());
    let right = Vec2::new(-rotation.sin(), rotation.cos());
    let offset = Vec2::new(
        forward.x * emitter.spawn_offset.x + right.x * emitter.spawn_offset.y,
        forward.y * emitter.spawn_offset.x + right.y * emitter.spawn_offset.y,
    );
    let mut transform = source_transform;
    transform.translation = Vec3::new(
        source_transform.translation.x + offset.x,
        source_transform.translation.y + offset.y,
        source_transform.translation.z,
    );

    ProjectileLaunch2d {
        transform,
        velocity: Vec2::new(
            forward.x * emitter.speed + source_velocity.x * emitter.inherit_velocity_scale,
            forward.y * emitter.speed + source_velocity.y * emitter.inherit_velocity_scale,
        ),
    }
}

pub fn apply_bounds_2d(
    translation: Vec2,
    velocity: Vec2,
    bounds: &Bounds2d,
) -> BoundsApplyResult2d {
    let contact = bounds_contact_2d(translation, bounds);
    if !contact.any() {
        return BoundsApplyResult2d {
            translation,
            velocity,
            outcome: BoundsOutcome2d::None,
        };
    }

    match bounds.behavior {
        BoundsBehavior2d::Bounce { restitution } => {
            let restitution = restitution.max(0.0);
            let mut next_translation = clamp_to_bounds_2d(translation, bounds);
            let mut next_velocity = velocity;
            if contact.min_x {
                next_translation.x = bounds.min.x;
                next_velocity.x = velocity.x.abs() * restitution;
            } else if contact.max_x {
                next_translation.x = bounds.max.x;
                next_velocity.x = -velocity.x.abs() * restitution;
            }
            if contact.min_y {
                next_translation.y = bounds.min.y;
                next_velocity.y = velocity.y.abs() * restitution;
            } else if contact.max_y {
                next_translation.y = bounds.max.y;
                next_velocity.y = -velocity.y.abs() * restitution;
            }
            BoundsApplyResult2d {
                translation: next_translation,
                velocity: next_velocity,
                outcome: BoundsOutcome2d::Bounced { contact },
            }
        }
        BoundsBehavior2d::Wrap => {
            let mut next_translation = translation;
            if contact.min_x {
                next_translation.x = bounds.max.x;
            } else if contact.max_x {
                next_translation.x = bounds.min.x;
            }
            if contact.min_y {
                next_translation.y = bounds.max.y;
            } else if contact.max_y {
                next_translation.y = bounds.min.y;
            }
            BoundsApplyResult2d {
                translation: next_translation,
                velocity,
                outcome: BoundsOutcome2d::Wrapped { contact },
            }
        }
        BoundsBehavior2d::Hide => BoundsApplyResult2d {
            translation,
            velocity,
            outcome: BoundsOutcome2d::Hidden { contact },
        },
        BoundsBehavior2d::Despawn => BoundsApplyResult2d {
            translation,
            velocity,
            outcome: BoundsOutcome2d::Despawned { contact },
        },
        BoundsBehavior2d::Clamp => BoundsApplyResult2d {
            translation: clamp_to_bounds_2d(translation, bounds),
            velocity,
            outcome: BoundsOutcome2d::Clamped { contact },
        },
    }
}

pub fn step_freeflight_motion_2d(
    profile: &FreeflightMotionProfile2d,
    intent: &FreeflightMotionIntent2d,
    state: FreeflightMotionState2d,
    delta_seconds: f32,
) -> FreeflightMotionStep2d {
    let thrust = intent.thrust.clamp(-1.0, 1.0);
    let strafe = intent.strafe.clamp(-1.0, 1.0);
    let turn = intent.turn.clamp(-1.0, 1.0);
    let forward = Vec2::new(state.rotation_radians.cos(), state.rotation_radians.sin());
    let right = Vec2::new(-state.rotation_radians.sin(), state.rotation_radians.cos());
    let thrust_acceleration = if thrust >= 0.0 {
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

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

fn bounds_contact_2d(translation: Vec2, bounds: &Bounds2d) -> BoundsContact2d {
    BoundsContact2d {
        min_x: translation.x < bounds.min.x,
        max_x: translation.x > bounds.max.x,
        min_y: translation.y < bounds.min.y,
        max_y: translation.y > bounds.max.y,
    }
}

fn clamp_to_bounds_2d(translation: Vec2, bounds: &Bounds2d) -> Vec2 {
    Vec2::new(
        translation.x.clamp(bounds.min.x, bounds.max.x),
        translation.y.clamp(bounds.min.y, bounds.max.y),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bounds(behavior: BoundsBehavior2d) -> Bounds2d {
        Bounds2d {
            min: Vec2::new(-10.0, -5.0),
            max: Vec2::new(10.0, 5.0),
            behavior,
        }
    }

    fn test_freeflight_profile() -> FreeflightMotionProfile2d {
        FreeflightMotionProfile2d {
            thrust_acceleration: 20.0,
            reverse_acceleration: 10.0,
            strafe_acceleration: 12.0,
            turn_acceleration: 8.0,
            linear_damping: 2.0,
            turn_damping: 4.0,
            max_speed: 30.0,
            max_angular_speed: 6.0,
        }
    }

    #[test]
    fn step_velocity_applies_linear_displacement() {
        let next = step_velocity_2d(
            Vec2::new(2.0, -3.0),
            &Velocity2d::new(Vec2::new(4.0, 8.0)),
            0.5,
        );

        assert_eq!(next, Vec2::new(4.0, 1.0));
    }

    #[test]
    fn projectile_launch_uses_transform_facing_offset_and_inherited_velocity() {
        let launch = projectile_launch_2d(
            Transform3 {
                translation: Vec3::new(10.0, 20.0, 0.0),
                rotation_euler: Vec3::new(0.0, 0.0, std::f32::consts::FRAC_PI_2),
                ..Transform3::default()
            },
            Vec2::new(4.0, -2.0),
            &ProjectileEmitter2d {
                pool: "projectiles".to_owned(),
                speed: 100.0,
                spawn_offset: Vec2::new(8.0, 2.0),
                inherit_velocity_scale: 0.5,
            },
        );

        assert!((launch.transform.translation.x - 8.0).abs() < 0.001);
        assert!((launch.transform.translation.y - 28.0).abs() < 0.001);
        assert!((launch.velocity.x - 2.0).abs() < 0.001);
        assert!((launch.velocity.y - 99.0).abs() < 0.001);
    }

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

    #[test]
    fn bounds_bounce_reflects_velocity_and_reports_contact() {
        let result = apply_bounds_2d(
            Vec2::new(12.0, 0.0),
            Vec2::new(8.0, 1.0),
            &test_bounds(BoundsBehavior2d::Bounce { restitution: 0.5 }),
        );

        assert_eq!(result.translation, Vec2::new(10.0, 0.0));
        assert_eq!(result.velocity, Vec2::new(-4.0, 1.0));
        assert_eq!(
            result.outcome,
            BoundsOutcome2d::Bounced {
                contact: BoundsContact2d {
                    max_x: true,
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn bounds_wrap_moves_to_opposite_edge() {
        let result = apply_bounds_2d(
            Vec2::new(-12.0, 6.0),
            Vec2::new(-2.0, 3.0),
            &test_bounds(BoundsBehavior2d::Wrap),
        );

        assert_eq!(result.translation, Vec2::new(10.0, -5.0));
        assert_eq!(result.velocity, Vec2::new(-2.0, 3.0));
        assert_eq!(
            result.outcome,
            BoundsOutcome2d::Wrapped {
                contact: BoundsContact2d {
                    min_x: true,
                    max_y: true,
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn bounds_clamp_limits_translation_without_lifecycle_outcome() {
        let result = apply_bounds_2d(
            Vec2::new(2.0, -8.0),
            Vec2::new(1.0, -9.0),
            &test_bounds(BoundsBehavior2d::Clamp),
        );

        assert_eq!(result.translation, Vec2::new(2.0, -5.0));
        assert_eq!(result.velocity, Vec2::new(1.0, -9.0));
        assert_eq!(
            result.outcome,
            BoundsOutcome2d::Clamped {
                contact: BoundsContact2d {
                    min_y: true,
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn bounds_hide_reports_generic_hidden_outcome() {
        let result = apply_bounds_2d(
            Vec2::new(11.0, 0.0),
            Vec2::new(1.0, 0.0),
            &test_bounds(BoundsBehavior2d::Hide),
        );

        assert_eq!(
            result.outcome,
            BoundsOutcome2d::Hidden {
                contact: BoundsContact2d {
                    max_x: true,
                    ..Default::default()
                }
            }
        );
    }

    #[test]
    fn bounds_despawn_reports_generic_despawn_outcome() {
        let result = apply_bounds_2d(
            Vec2::new(0.0, -6.0),
            Vec2::new(0.0, -1.0),
            &test_bounds(BoundsBehavior2d::Despawn),
        );

        assert_eq!(
            result.outcome,
            BoundsOutcome2d::Despawned {
                contact: BoundsContact2d {
                    min_y: true,
                    ..Default::default()
                }
            }
        );
    }
}
