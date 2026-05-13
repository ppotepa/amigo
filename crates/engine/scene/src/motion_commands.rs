use amigo_math::{Curve1d, Vec2};

#[derive(Debug, Clone, PartialEq)]
pub struct Velocity2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub velocity: Vec2,
}

impl Velocity2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        velocity: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsBehavior2dSceneCommand {
    Bounce { restitution: f32 },
    Wrap,
    Hide,
    Despawn,
    Clamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub min: Vec2,
    pub max: Vec2,
    pub behavior: BoundsBehavior2dSceneCommand,
}

impl Bounds2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        min: Vec2,
        max: Vec2,
        behavior: BoundsBehavior2dSceneCommand,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            min,
            max,
            behavior,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightMotion2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub thrust_acceleration: f32,
    pub reverse_acceleration: f32,
    pub strafe_acceleration: f32,
    pub turn_acceleration: f32,
    pub linear_damping: f32,
    pub turn_damping: f32,
    pub max_speed: f32,
    pub max_angular_speed: f32,
    pub initial_velocity: Vec2,
    pub initial_angular_velocity: f32,
    pub thrust_response_curve: Curve1d,
    pub reverse_response_curve: Curve1d,
    pub strafe_response_curve: Curve1d,
    pub turn_response_curve: Curve1d,
}

impl FreeflightMotion2dSceneCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        thrust_acceleration: f32,
        reverse_acceleration: f32,
        strafe_acceleration: f32,
        turn_acceleration: f32,
        linear_damping: f32,
        turn_damping: f32,
        max_speed: f32,
        max_angular_speed: f32,
        initial_velocity: Vec2,
        initial_angular_velocity: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            thrust_acceleration,
            reverse_acceleration,
            strafe_acceleration,
            turn_acceleration,
            linear_damping,
            turn_damping,
            max_speed,
            max_angular_speed,
            initial_velocity,
            initial_angular_velocity,
            thrust_response_curve: Curve1d::Linear,
            reverse_response_curve: Curve1d::Linear,
            strafe_response_curve: Curve1d::Linear,
            turn_response_curve: Curve1d::Linear,
        }
    }

    pub fn with_response_curves(
        mut self,
        thrust: Curve1d,
        reverse: Curve1d,
        strafe: Curve1d,
        turn: Curve1d,
    ) -> Self {
        self.thrust_response_curve = thrust;
        self.reverse_response_curve = reverse;
        self.strafe_response_curve = strafe;
        self.turn_response_curve = turn;
        self
    }
}

