use super::defaults::*;
use super::components::SceneComponentDocument;
use super::render_values::{SceneTransform2Document, SceneTransform3Document, SceneVec2Document};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneDocument {
    #[serde(default = "default_scene_document_version")]
    pub version: u32,
    pub scene: SceneMetadataDocument,
    #[serde(default)]
    pub transitions: Vec<SceneTransitionDocument>,
    #[serde(default)]
    pub collision_events: Vec<SceneCollisionEventRule2dDocument>,
    #[serde(default)]
    pub audio_cues: Vec<SceneAudioCueDocument>,
    #[serde(default)]
    pub activation_sets: Vec<SceneActivationSetDocument>,
    #[serde(default)]
    pub entities: Vec<SceneEntityDocument>,
}

impl SceneDocument {
    pub fn entity_names(&self) -> Vec<String> {
        self.entities
            .iter()
            .map(SceneEntityDocument::display_name)
            .collect()
    }

    pub fn component_kind_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();

        for entity in &self.entities {
            for component in &entity.components {
                *counts.entry(component.kind().to_owned()).or_insert(0) += 1;
            }
        }

        counts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneMetadataDocument {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneTransitionDocument {
    #[serde(default)]
    pub id: String,
    pub to: String,
    pub when: SceneTransitionConditionDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneTransitionConditionDocument {
    AfterSeconds {
        seconds: f32,
    },
    ScriptEvent {
        topic: String,
        #[serde(default)]
        payload: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneEntityDocument {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default = "default_entity_lifecycle_flag")]
    pub visible: bool,
    #[serde(default = "default_entity_lifecycle_flag")]
    pub simulation_enabled: bool,
    #[serde(default = "default_entity_lifecycle_flag")]
    pub collision_enabled: bool,
    #[serde(default)]
    pub properties: BTreeMap<String, ScenePropertyValueDocument>,
    #[serde(default)]
    pub transform2: Option<SceneTransform2Document>,
    #[serde(default)]
    pub transform3: Option<SceneTransform3Document>,
    #[serde(default)]
    pub components: Vec<SceneComponentDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ScenePropertyValueDocument {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneEntitySelectorDocument {
    pub kind: SceneEntitySelectorKindDocument,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneEntitySelectorKindDocument {
    Entity,
    Tag,
    Group,
    Pool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneCollisionEventRule2dDocument {
    pub id: String,
    pub source: SceneEntitySelectorDocument,
    pub target: SceneEntitySelectorDocument,
    pub event: String,
    #[serde(default = "default_once_per_overlap")]
    pub once_per_overlap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneAudioCueDocument {
    pub name: String,
    pub clip: String,
    #[serde(default)]
    pub min_interval: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneActivationSetDocument {
    pub id: String,
    #[serde(default)]
    pub entries: Vec<SceneActivationEntryDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneActivationEntryDocument {
    pub target: SceneEntitySelectorDocument,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub simulation_enabled: Option<bool>,
    #[serde(default)]
    pub collision_enabled: Option<bool>,
    #[serde(default)]
    pub transform2: Option<SceneTransform2Document>,
    #[serde(default)]
    pub transform3: Option<SceneTransform3Document>,
    #[serde(default)]
    pub velocity: Option<SceneVec2Document>,
    #[serde(default)]
    pub angular_velocity: Option<f32>,
    #[serde(default)]
    pub properties: BTreeMap<String, ScenePropertyValueDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CurvePoint1dSceneDocument {
    pub t: f32,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorRampSceneDocument {
    #[serde(default)]
    pub interpolation: ColorInterpolationSceneDocument,
    pub stops: Vec<ColorStopSceneDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorStopSceneDocument {
    pub t: f32,
    pub color: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ColorInterpolationSceneDocument {
    #[default]
    LinearRgb,
    Step,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Curve1dSceneDocument {
    Constant {
        value: f32,
    },
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    SmoothStep,
    Custom {
        points: Vec<CurvePoint1dSceneDocument>,
    },
}
