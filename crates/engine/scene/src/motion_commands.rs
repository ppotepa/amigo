use amigo_math::{Curve1d, Vec2};

pub const VELOCITY_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.shutter-motion.scene-command.Velocity2d";
pub const BOUNDS_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.shutter-motion.scene-command.Bounds2d";
pub const FREEFLIGHT_MOTION_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.camera.shutter-motion.scene-command.FreeflightMotion2d";

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

#[derive(Debug, Clone, PartialEq)]
pub struct Velocity2dPluginSceneCommandPayload(pub Velocity2dSceneCommand);

impl crate::PluginSceneCommandPayload for Velocity2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        VELOCITY_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Velocity2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn velocity_2d_plugin_scene_command(
    command: Velocity2dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(Velocity2dPluginSceneCommandPayload(
        command,
    )))
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
pub struct Bounds2dPluginSceneCommandPayload(pub Bounds2dSceneCommand);

impl crate::PluginSceneCommandPayload for Bounds2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        BOUNDS_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Bounds2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn bounds_2d_plugin_scene_command(command: Bounds2dSceneCommand) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(Bounds2dPluginSceneCommandPayload(
        command,
    )))
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

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightMotion2dPluginSceneCommandPayload(pub FreeflightMotion2dSceneCommand);

impl crate::PluginSceneCommandPayload for FreeflightMotion2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        FREEFLIGHT_MOTION_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<FreeflightMotion2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn freeflight_motion_2d_plugin_scene_command(
    command: FreeflightMotion2dSceneCommand,
) -> crate::PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        FreeflightMotion2dPluginSceneCommandPayload(command),
    ))
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
