use std::{any::Any, collections::BTreeMap};

use amigo_render_npr::{ComicInk, NprMotionPolicy, NprSurfaceMode};
use amigo_scene::{
    SceneComponentPayload, SceneComponentSchemaProvider, SceneDocumentError, SceneDocumentResult,
};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::state::{ConstructionMarkSettings, ObjectSettings, Settings};

pub const NPR_SETTINGS_COMPONENT_TYPE: &str = "amigo.gfx.npr-playground.NprSettings";

/// Declarative scene settings. Fields are deliberately optional: scene authors
/// state only their intent and inherit the canonical workshop defaults for the
/// rest of the gallery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprPlaygroundSceneDocument {
    #[serde(default)]
    pub gallery: bool,
    #[serde(default)]
    pub selected: Option<String>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub motion: Option<NprMotionPolicy>,
    #[serde(default)]
    pub global_style: Option<ComicInk>,
    #[serde(default)]
    pub camera: NprCameraSceneSettings,
    #[serde(default)]
    pub objects: BTreeMap<String, NprObjectSceneSettings>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprCameraSceneSettings {
    #[serde(default)]
    pub target: Option<Vec3>,
    #[serde(default)]
    pub yaw: Option<f32>,
    #[serde(default)]
    pub pitch: Option<f32>,
    #[serde(default)]
    pub distance: Option<f32>,
    #[serde(default)]
    pub fov: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NprObjectSceneSettings {
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub rotating: Option<bool>,
    #[serde(default)]
    pub position: Option<Vec3>,
    #[serde(default)]
    pub rotation: Option<Vec3>,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub angular_speed: Option<Vec3>,
    #[serde(default)]
    pub surface_mode: Option<NprSurfaceMode>,
    #[serde(default)]
    pub surface_subdivision_level: Option<u8>,
    #[serde(default)]
    pub override_style: Option<bool>,
    #[serde(default)]
    pub style: Option<ComicInk>,
    #[serde(default)]
    pub construction_marks: Option<Vec<ConstructionMarkSettings>>,
}

impl NprPlaygroundSceneDocument {
    pub fn apply_to(&self, settings: &mut Settings) -> Result<(), String> {
        settings.gallery = self.gallery;
        if let Some(selected) = &self.selected {
            settings.selected = selected.clone();
        }
        if let Some(seed) = self.seed {
            settings.seed = seed;
        }
        if let Some(motion) = self.motion {
            settings.motion = motion;
        }
        if let Some(style) = self.global_style {
            settings.global = style;
        }
        if let Some(target) = self.camera.target {
            settings.camera_target = target;
        }
        if let Some(yaw) = self.camera.yaw {
            settings.camera_yaw = yaw;
        }
        if let Some(pitch) = self.camera.pitch {
            settings.camera_pitch = pitch;
        }
        if let Some(distance) = self.camera.distance {
            settings.camera_distance = distance;
        }
        if let Some(fov) = self.camera.fov {
            settings.camera_fov = fov;
        }
        for (id, authored) in &self.objects {
            let Some(object) = settings.objects.get_mut(id) else {
                return Err(format!("unknown NPR object `{id}`"));
            };
            apply_object_settings(object, authored);
        }
        settings.validate()
    }
}

fn apply_object_settings(object: &mut ObjectSettings, authored: &NprObjectSceneSettings) {
    if let Some(value) = authored.visible { object.visible = value; }
    if let Some(value) = authored.rotating { object.rotating = value; }
    if let Some(value) = authored.position { object.position = value; }
    if let Some(value) = authored.rotation { object.rotation = value; }
    if let Some(value) = authored.scale { object.scale = value; }
    if let Some(value) = authored.angular_speed { object.angular_speed = value; }
    if let Some(value) = authored.surface_mode { object.surface_mode = value; }
    if let Some(value) = authored.surface_subdivision_level { object.surface_subdivision_level = value; }
    if let Some(value) = authored.override_style { object.override_style = value; }
    if let Some(value) = authored.style { object.style = value; }
    if let Some(value) = &authored.construction_marks { object.construction_marks = value.clone(); }
}

impl amigo_scene::SceneComponentPayload for NprPlaygroundSceneDocument {
    fn component_type(&self) -> &'static str { NPR_SETTINGS_COMPONENT_TYPE }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NprPlaygroundSceneSchemaProvider;

impl SceneComponentSchemaProvider for NprPlaygroundSceneSchemaProvider {
    fn component_type(&self) -> &'static str { NPR_SETTINGS_COMPONENT_TYPE }

    fn parse_yaml(&self, payload: serde_yaml::Mapping) -> Result<Value, serde_yaml::Error> {
        serde_yaml::to_value(serde_yaml::from_value::<NprPlaygroundSceneDocument>(Value::Mapping(payload))?)
    }

    fn parse_payload_value(&self, payload: &Value) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        serde_yaml::from_value::<NprPlaygroundSceneDocument>(payload.clone())
            .map(|document| Box::new(document) as Box<dyn SceneComponentPayload>)
            .map_err(|source| SceneDocumentError::Parse { path: None, source })
    }
}
