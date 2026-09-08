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
    /// Captures the declarative portion of live workshop settings.
    ///
    /// Runtime-only controls such as pause state, diagnostics and preset UI
    /// selection are deliberately omitted. The result can therefore become a
    /// component payload in a scene command or future editor transaction.
    pub fn from_settings(settings: &Settings) -> Result<Self, String> {
        settings.validate()?;
        let defaults = Settings::for_scene(settings.gallery);
        let mut objects = BTreeMap::new();
        for (id, object) in &settings.objects {
            let default = &defaults.objects[id];
            let authored = NprObjectSceneSettings {
                visible: (object.visible != default.visible).then_some(object.visible),
                rotating: (object.rotating != default.rotating).then_some(object.rotating),
                position: (object.position != default.position).then_some(object.position),
                rotation: (object.rotation != default.rotation).then_some(object.rotation),
                scale: (object.scale != default.scale).then_some(object.scale),
                angular_speed: (object.angular_speed != default.angular_speed)
                    .then_some(object.angular_speed),
                surface_mode: (object.surface_mode != default.surface_mode)
                    .then_some(object.surface_mode),
                surface_subdivision_level: (object.surface_subdivision_level
                    != default.surface_subdivision_level)
                    .then_some(object.surface_subdivision_level),
                override_style: (object.override_style != default.override_style)
                    .then_some(object.override_style),
                style: (object.style != default.style).then_some(object.style),
                construction_marks: (object.construction_marks != default.construction_marks)
                    .then_some(object.construction_marks.clone()),
            };
            if authored != NprObjectSceneSettings::default() {
                objects.insert(id.clone(), authored);
            }
        }
        Ok(Self {
            gallery: settings.gallery,
            selected: (settings.selected != defaults.selected).then_some(settings.selected.clone()),
            seed: (settings.seed != defaults.seed).then_some(settings.seed),
            motion: (settings.motion != defaults.motion).then_some(settings.motion),
            global_style: (settings.global != defaults.global).then_some(settings.global),
            camera: NprCameraSceneSettings {
                target: (settings.camera_target != defaults.camera_target)
                    .then_some(settings.camera_target),
                yaw: (settings.camera_yaw != defaults.camera_yaw).then_some(settings.camera_yaw),
                pitch: (settings.camera_pitch != defaults.camera_pitch)
                    .then_some(settings.camera_pitch),
                distance: (settings.camera_distance != defaults.camera_distance)
                    .then_some(settings.camera_distance),
                fov: (settings.camera_fov != defaults.camera_fov).then_some(settings.camera_fov),
            },
            objects,
        })
    }

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
