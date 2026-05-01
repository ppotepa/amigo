use std::collections::BTreeMap;
use std::path::PathBuf;

use amigo_core::TypedId;
use amigo_math::Transform3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SceneEntityTag;
pub type SceneEntityId = TypedId<SceneEntityTag>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneKey(String);

impl SceneKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntitySelector {
    Entity(String),
    Tag(String),
    Group(String),
    Pool(String),
}

#[derive(Debug, Clone)]
pub struct SceneEntity {
    pub id: SceneEntityId,
    pub name: String,
    pub transform: Transform3,
    pub lifecycle: SceneEntityLifecycle,
    pub tags: Vec<String>,
    pub groups: Vec<String>,
    pub properties: BTreeMap<String, ScenePropertyValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneEntityLifecycle {
    pub visible: bool,
    pub simulation_enabled: bool,
    pub collision_enabled: bool,
}

impl Default for SceneEntityLifecycle {
    fn default() -> Self {
        Self {
            visible: true,
            simulation_enabled: true,
            collision_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPoolSceneCommand {
    pub source_mod: String,
    pub pool: String,
    pub members: Vec<String>,
}

impl EntityPoolSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        pool: impl Into<String>,
        members: Vec<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            pool: pool.into(),
            members,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifetimeExpirationOutcome {
    Hide,
    Disable,
    Despawn,
    ReturnToPool { pool: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub seconds: f32,
    pub outcome: LifetimeExpirationOutcome,
}

impl LifetimeSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        seconds: f32,
        outcome: LifetimeExpirationOutcome,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            seconds,
            outcome,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ScenePropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HydratedSceneSnapshot {
    pub source_mod: Option<String>,
    pub scene_id: Option<String>,
    pub relative_document_path: Option<PathBuf>,
    pub entity_names: Vec<String>,
    pub component_kinds: Vec<String>,
}

