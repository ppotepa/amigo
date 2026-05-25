pub const KINEMATIC_BODY_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.2d.scene-command.KinematicBody2d";
pub const AABB_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.2d.scene-command.AabbCollider2d";
pub const STATIC_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.2d.scene-command.StaticCollider2d";
pub const CIRCLE_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.2d.scene-command.CircleCollider2d";
pub const TRIGGER_2D_PLUGIN_SCENE_COMMAND_TYPE: &str = "amigo.physics.2d.scene-command.Trigger2d";
pub const COLLISION_EVENT_RULE_2D_PLUGIN_SCENE_COMMAND_TYPE: &str =
    "amigo.physics.2d.scene-command.CollisionEventRule2d";

#[derive(Debug, Clone, PartialEq)]
pub struct KinematicBody2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub velocity: Vec2,
    pub gravity_scale: f32,
    pub terminal_velocity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KinematicBody2dPluginSceneCommandPayload(pub KinematicBody2dSceneCommand);

impl crate::PluginSceneCommandPayload for KinematicBody2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        KINEMATIC_BODY_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<KinematicBody2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn kinematic_body_2d_plugin_scene_command(
    command: KinematicBody2dSceneCommand,
) -> PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        KinematicBody2dPluginSceneCommandPayload(command),
    ))
}

impl KinematicBody2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        velocity: Vec2,
        gravity_scale: f32,
        terminal_velocity: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            velocity,
            gravity_scale,
            terminal_velocity,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct AabbCollider2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: String,
    pub mask: Vec<String>,
}

impl AabbCollider2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec2,
        offset: Vec2,
        layer: impl Into<String>,
        mask: Vec<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
            layer: layer.into(),
            mask,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AabbCollider2dPluginSceneCommandPayload(pub AabbCollider2dSceneCommand);

impl crate::PluginSceneCommandPayload for AabbCollider2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        AABB_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<AabbCollider2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn aabb_collider_2d_plugin_scene_command(
    command: AabbCollider2dSceneCommand,
) -> PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        AabbCollider2dPluginSceneCommandPayload(command),
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticCollider2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: String,
}

impl StaticCollider2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec2,
        offset: Vec2,
        layer: impl Into<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
            layer: layer.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticCollider2dPluginSceneCommandPayload(pub StaticCollider2dSceneCommand);

impl crate::PluginSceneCommandPayload for StaticCollider2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        STATIC_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<StaticCollider2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn static_collider_2d_plugin_scene_command(
    command: StaticCollider2dSceneCommand,
) -> PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        StaticCollider2dPluginSceneCommandPayload(command),
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircleCollider2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub radius: f32,
    pub offset: Vec2,
}

impl CircleCollider2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        radius: f32,
        offset: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            radius,
            offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircleCollider2dPluginSceneCommandPayload(pub CircleCollider2dSceneCommand);

impl crate::PluginSceneCommandPayload for CircleCollider2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        CIRCLE_COLLIDER_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<CircleCollider2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn circle_collider_2d_plugin_scene_command(
    command: CircleCollider2dSceneCommand,
) -> PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        CircleCollider2dPluginSceneCommandPayload(command),
    ))
}
#[derive(Debug, Clone, PartialEq)]
pub struct Trigger2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: String,
    pub mask: Vec<String>,
    pub event: Option<String>,
}

impl Trigger2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec2,
        offset: Vec2,
        layer: impl Into<String>,
        mask: Vec<String>,
        event: Option<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
            layer: layer.into(),
            mask,
            event,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trigger2dPluginSceneCommandPayload(pub Trigger2dSceneCommand);

impl crate::PluginSceneCommandPayload for Trigger2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        TRIGGER_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<Trigger2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn trigger_2d_plugin_scene_command(command: Trigger2dSceneCommand) -> PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(Trigger2dPluginSceneCommandPayload(
        command,
    )))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionEventRule2dSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub source: EntitySelector,
    pub target: EntitySelector,
    pub event: String,
    pub once_per_overlap: bool,
}

impl CollisionEventRule2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        id: impl Into<String>,
        source: EntitySelector,
        target: EntitySelector,
        event: impl Into<String>,
        once_per_overlap: bool,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            id: id.into(),
            source,
            target,
            event: event.into(),
            once_per_overlap,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionEventRule2dPluginSceneCommandPayload(pub CollisionEventRule2dSceneCommand);

impl crate::PluginSceneCommandPayload for CollisionEventRule2dPluginSceneCommandPayload {
    fn command_type(&self) -> &'static str {
        COLLISION_EVENT_RULE_2D_PLUGIN_SCENE_COMMAND_TYPE
    }

    fn command_as_any(&self) -> &dyn std::any::Any {
        &self.0
    }

    fn eq_payload(&self, other: &dyn crate::PluginSceneCommandPayload) -> bool {
        other
            .command_as_any()
            .downcast_ref::<CollisionEventRule2dSceneCommand>()
            .is_some_and(|command| command == &self.0)
    }
}

pub fn collision_event_rule_2d_plugin_scene_command(
    command: CollisionEventRule2dSceneCommand,
) -> PluginSceneCommand {
    crate::PluginSceneCommand::new(std::sync::Arc::new(
        CollisionEventRule2dPluginSceneCommandPayload(command),
    ))
}
