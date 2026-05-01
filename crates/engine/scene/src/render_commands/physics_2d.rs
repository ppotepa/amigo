#[derive(Debug, Clone, PartialEq)]
pub struct KinematicBody2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub velocity: Vec2,
    pub gravity_scale: f32,
    pub terminal_velocity: f32,
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
