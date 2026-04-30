use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{SceneDocumentError, SceneDocumentResult};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParticleShape2dSceneDocument {
    Circle {
        #[serde(default = "default_vector_segments")]
        segments: u32,
    },
    Quad,
    Line {
        length: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParticleSpawnArea2dSceneDocument {
    Point,
    Line {
        length: f32,
    },
    Rect {
        size: SceneVec2Document,
    },
    Circle {
        radius: f32,
    },
    Ring {
        inner_radius: f32,
        outer_radius: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParticleForce2dSceneDocument {
    Gravity {
        acceleration: SceneVec2Document,
    },
    ConstantAcceleration {
        acceleration: SceneVec2Document,
    },
    Drag {
        coefficient: f32,
    },
    Wind {
        velocity: SceneVec2Document,
        strength: f32,
    },
}

impl SceneEntityDocument {
    pub fn display_name(&self) -> String {
        if self.name.trim().is_empty() {
            self.id.clone()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SceneComponentDocument {
    #[serde(rename = "Camera2D")]
    Camera2d,
    #[serde(rename = "Camera3D")]
    Camera3d,
    #[serde(rename = "Light3D")]
    Light3d {
        #[serde(default)]
        kind: String,
    },
    #[serde(rename = "Sprite2D")]
    Sprite2d {
        texture: String,
        size: SceneVec2Document,
        #[serde(default)]
        sheet: Option<SceneSpriteSheetDocument>,
        #[serde(default)]
        animation: Option<SceneSpriteAnimationDocument>,
        #[serde(default)]
        z_index: f32,
    },
    #[serde(rename = "TileMap2D")]
    TileMap2d {
        tileset: String,
        #[serde(default)]
        ruleset: Option<String>,
        tile_size: SceneVec2Document,
        grid: Vec<String>,
        #[serde(default)]
        depth_fill_rows: usize,
        #[serde(default)]
        z_index: f32,
    },
    #[serde(rename = "Text2D")]
    Text2d {
        content: String,
        font: String,
        bounds: SceneVec2Document,
    },
    #[serde(rename = "VectorShape2D")]
    VectorShape2d {
        kind: SceneVectorShapeKindComponentDocument,
        #[serde(default)]
        points: Vec<SceneVec2Document>,
        #[serde(default)]
        closed: bool,
        #[serde(default)]
        radius: f32,
        #[serde(default = "default_vector_segments")]
        segments: u32,
        #[serde(default)]
        stroke_color: Option<String>,
        #[serde(default = "default_vector_stroke_width")]
        stroke_width: f32,
        #[serde(default)]
        fill_color: Option<String>,
        #[serde(default)]
        z_index: f32,
    },
    #[serde(rename = "EntityPool")]
    EntityPool {
        #[serde(default)]
        pool: Option<String>,
        members: Vec<String>,
    },
    #[serde(rename = "Lifetime")]
    Lifetime {
        seconds: f32,
        outcome: SceneLifetimeExpirationOutcomeDocument,
        #[serde(default)]
        pool: Option<String>,
    },
    #[serde(rename = "ProjectileEmitter2D")]
    ProjectileEmitter2d {
        pool: String,
        speed: f32,
        #[serde(default = "default_vec2_zero")]
        spawn_offset: SceneVec2Document,
        #[serde(default)]
        inherit_velocity_scale: f32,
    },
    #[serde(rename = "ParticleEmitter2D")]
    ParticleEmitter2d {
        #[serde(default)]
        attached_to: Option<String>,
        #[serde(default = "default_vec2_zero")]
        local_offset: SceneVec2Document,
        #[serde(default)]
        local_direction_degrees: f32,
        #[serde(default)]
        spawn_area: Option<ParticleSpawnArea2dSceneDocument>,
        #[serde(default)]
        active: bool,
        #[serde(default = "default_particle_spawn_rate")]
        spawn_rate: f32,
        #[serde(default = "default_particle_max_particles")]
        max_particles: usize,
        #[serde(default = "default_particle_lifetime")]
        particle_lifetime: f32,
        #[serde(default)]
        lifetime_jitter: f32,
        #[serde(default)]
        initial_speed: f32,
        #[serde(default)]
        speed_jitter: f32,
        #[serde(default)]
        spread_degrees: f32,
        #[serde(default)]
        inherit_parent_velocity: f32,
        #[serde(default = "default_particle_initial_size")]
        initial_size: f32,
        #[serde(default = "default_particle_final_size")]
        final_size: f32,
        #[serde(default)]
        color: Option<String>,
        #[serde(default)]
        color_ramp: Option<ColorRampSceneDocument>,
        #[serde(default)]
        z_index: f32,
        #[serde(default)]
        shape: Option<ParticleShape2dSceneDocument>,
        #[serde(default)]
        emission_rate_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        size_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        alpha_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        speed_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        forces: Vec<ParticleForce2dSceneDocument>,
    },
    #[serde(rename = "Velocity2D")]
    Velocity2d {
        #[serde(default = "default_vec2_zero")]
        velocity: SceneVec2Document,
    },
    #[serde(rename = "Bounds2D")]
    Bounds2d {
        min: SceneVec2Document,
        max: SceneVec2Document,
        behavior: SceneBoundsBehavior2dDocument,
        #[serde(default = "default_bounds_restitution")]
        restitution: f32,
    },
    #[serde(rename = "FreeflightMotion2D")]
    FreeflightMotion2d {
        thrust_acceleration: f32,
        reverse_acceleration: f32,
        strafe_acceleration: f32,
        turn_acceleration: f32,
        linear_damping: f32,
        turn_damping: f32,
        max_speed: f32,
        max_angular_speed: f32,
        #[serde(default = "default_vec2_zero")]
        initial_velocity: SceneVec2Document,
        #[serde(default)]
        initial_angular_velocity: f32,
        #[serde(default)]
        thrust_response_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        reverse_response_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        strafe_response_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        turn_response_curve: Option<Curve1dSceneDocument>,
    },
    #[serde(rename = "KinematicBody2D")]
    KinematicBody2d {
        #[serde(default = "default_vec2_zero")]
        velocity: SceneVec2Document,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        terminal_velocity: f32,
    },
    #[serde(rename = "AabbCollider2D")]
    AabbCollider2d {
        size: SceneVec2Document,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        layer: String,
        #[serde(default)]
        mask: Vec<String>,
    },
    #[serde(rename = "CircleCollider2D")]
    CircleCollider2d {
        radius: f32,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
    },
    #[serde(rename = "Trigger2D")]
    Trigger2d {
        size: SceneVec2Document,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        layer: String,
        #[serde(default)]
        mask: Vec<String>,
        #[serde(default)]
        event: Option<String>,
    },
    #[serde(rename = "MotionController2D")]
    MotionController2d {
        max_speed: f32,
        acceleration: f32,
        deceleration: f32,
        air_acceleration: f32,
        gravity: f32,
        jump_velocity: f32,
        terminal_velocity: f32,
    },
    #[serde(rename = "CameraFollow2D")]
    CameraFollow2d {
        target: String,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        #[serde(default = "default_camera_follow_lerp")]
        lerp: f32,
    },
    #[serde(rename = "Parallax2D")]
    Parallax2d {
        camera: String,
        factor: SceneVec2Document,
    },
    #[serde(rename = "TileMapMarker2D")]
    TileMapMarker2d {
        symbol: String,
        #[serde(default)]
        tilemap_entity: Option<String>,
        #[serde(default)]
        index: usize,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
    },
    #[serde(rename = "Mesh3D")]
    Mesh3d { mesh: String },
    #[serde(rename = "Material3D")]
    Material3d {
        label: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        albedo: Option<String>,
    },
    #[serde(rename = "Text3D")]
    Text3d {
        content: String,
        font: String,
        size: f32,
    },
    #[serde(rename = "UiDocument")]
    UiDocument {
        target: SceneUiTargetComponentDocument,
        root: SceneUiNodeComponentDocument,
    },
    #[serde(rename = "UiThemeSet")]
    UiThemeSet {
        #[serde(default)]
        active: Option<String>,
        themes: Vec<SceneUiThemeComponentDocument>,
    },
}

impl SceneComponentDocument {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Camera2d => "Camera2D",
            Self::Camera3d => "Camera3D",
            Self::Light3d { .. } => "Light3D",
            Self::Sprite2d { .. } => "Sprite2D",
            Self::TileMap2d { .. } => "TileMap2D",
            Self::Text2d { .. } => "Text2D",
            Self::VectorShape2d { .. } => "VectorShape2D",
            Self::EntityPool { .. } => "EntityPool",
            Self::Lifetime { .. } => "Lifetime",
            Self::ProjectileEmitter2d { .. } => "ProjectileEmitter2D",
            Self::ParticleEmitter2d { .. } => "ParticleEmitter2D",
            Self::Velocity2d { .. } => "Velocity2D",
            Self::Bounds2d { .. } => "Bounds2D",
            Self::FreeflightMotion2d { .. } => "FreeflightMotion2D",
            Self::KinematicBody2d { .. } => "KinematicBody2D",
            Self::AabbCollider2d { .. } => "AabbCollider2D",
            Self::CircleCollider2d { .. } => "CircleCollider2D",
            Self::Trigger2d { .. } => "Trigger2D",
            Self::MotionController2d { .. } => "MotionController2D",
            Self::CameraFollow2d { .. } => "CameraFollow2D",
            Self::Parallax2d { .. } => "Parallax2D",
            Self::TileMapMarker2d { .. } => "TileMapMarker2D",
            Self::Mesh3d { .. } => "Mesh3D",
            Self::Material3d { .. } => "Material3D",
            Self::Text3d { .. } => "Text3D",
            Self::UiDocument { .. } => "UiDocument",
            Self::UiThemeSet { .. } => "UiThemeSet",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiThemeComponentDocument {
    pub id: String,
    pub palette: SceneUiThemePaletteComponentDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiThemePaletteComponentDocument {
    pub background: String,
    pub surface: String,
    pub surface_alt: String,
    pub text: String,
    pub text_muted: String,
    pub border: String,
    pub accent: String,
    pub accent_text: String,
    pub danger: String,
    pub warning: String,
    pub success: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneLifetimeExpirationOutcomeDocument {
    Hide,
    Disable,
    Despawn,
    ReturnToPool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneBoundsBehavior2dDocument {
    Bounce,
    Wrap,
    Hide,
    Despawn,
    Clamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneVectorShapeKindComponentDocument {
    Polyline,
    Polygon,
    Circle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiTargetComponentDocument {
    #[serde(rename = "type")]
    pub kind: SceneUiTargetTypeComponentDocument,
    pub layer: SceneUiLayerComponentDocument,
    #[serde(default)]
    pub viewport: Option<SceneUiViewportComponentDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneUiViewportComponentDocument {
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub scaling: SceneUiViewportScalingComponentDocument,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiViewportScalingComponentDocument {
    #[default]
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiTargetTypeComponentDocument {
    ScreenSpace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiLayerComponentDocument {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiNodeComponentDocument {
    #[serde(rename = "type")]
    pub kind: SceneUiNodeTypeComponentDocument,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub style_class: Option<String>,
    #[serde(default)]
    pub style: SceneUiStyleComponentDocument,
    #[serde(default)]
    pub children: Vec<SceneUiNodeComponentDocument>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub value: Option<f32>,
    #[serde(default)]
    pub min: Option<f32>,
    #[serde(default)]
    pub max: Option<f32>,
    #[serde(default)]
    pub step: Option<f32>,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default)]
    pub selected: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub text_bind: Option<String>,
    #[serde(default)]
    pub visible_bind: Option<String>,
    #[serde(default)]
    pub enabled_bind: Option<String>,
    #[serde(default)]
    pub value_bind: Option<String>,
    #[serde(default)]
    pub on_click: Option<SceneUiEventBindingComponentDocument>,
    #[serde(default)]
    pub on_change: Option<SceneUiEventBindingComponentDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiNodeTypeComponentDocument {
    Panel,
    Row,
    Column,
    Stack,
    Text,
    Button,
    ProgressBar,
    Slider,
    Toggle,
    OptionSet,
    Dropdown,
    ColorPickerRgb,
    Spacer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneUiStyleComponentDocument {
    #[serde(default)]
    pub left: Option<f32>,
    #[serde(default)]
    pub top: Option<f32>,
    #[serde(default)]
    pub right: Option<f32>,
    #[serde(default)]
    pub bottom: Option<f32>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub padding: f32,
    #[serde(default)]
    pub gap: f32,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub border_width: f32,
    #[serde(default)]
    pub border_radius: f32,
    #[serde(default = "default_ui_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub word_wrap: bool,
    #[serde(default)]
    pub fit_to_width: bool,
    #[serde(default)]
    pub align: Option<SceneUiTextAlignComponentDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiTextAlignComponentDocument {
    Start,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneUiEventBindingComponentDocument {
    pub event: String,
    #[serde(default)]
    pub payload: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneSpriteSheetDocument {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: SceneVec2Document,
    #[serde(default = "default_sprite_sheet_fps")]
    pub fps: f32,
    #[serde(default = "default_sprite_sheet_looping")]
    pub looping: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneSpriteAnimationDocument {
    #[serde(default)]
    pub fps: Option<f32>,
    #[serde(default)]
    pub looping: Option<bool>,
    #[serde(default)]
    pub start_frame: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneVec2Document {
    pub x: f32,
    pub y: f32,
}

impl SceneVec2Document {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneVec3Document {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl SceneVec3Document {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneTransform2Document {
    #[serde(default = "default_vec2_zero")]
    pub translation: SceneVec2Document,
    #[serde(default)]
    pub rotation_radians: f32,
    #[serde(default = "default_vec2_one")]
    pub scale: SceneVec2Document,
}

impl Default for SceneTransform2Document {
    fn default() -> Self {
        Self {
            translation: default_vec2_zero(),
            rotation_radians: 0.0,
            scale: default_vec2_one(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneTransform3Document {
    #[serde(default = "default_vec3_zero")]
    pub translation: SceneVec3Document,
    #[serde(default = "default_vec3_zero")]
    pub rotation_euler: SceneVec3Document,
    #[serde(default = "default_vec3_one")]
    pub scale: SceneVec3Document,
}

impl Default for SceneTransform3Document {
    fn default() -> Self {
        Self {
            translation: default_vec3_zero(),
            rotation_euler: default_vec3_zero(),
            scale: default_vec3_one(),
        }
    }
}

pub fn load_scene_document_from_str(source: &str) -> SceneDocumentResult<SceneDocument> {
    serde_yaml::from_str::<SceneDocument>(source)
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

pub fn load_scene_document_from_path(path: impl AsRef<Path>) -> SceneDocumentResult<SceneDocument> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| SceneDocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    serde_yaml::from_str::<SceneDocument>(&raw).map_err(|source| SceneDocumentError::Parse {
        path: Some(path.to_path_buf()),
        source,
    })
}

pub fn scene_document_path(
    mod_root: impl AsRef<Path>,
    relative_document_path: impl AsRef<Path>,
) -> PathBuf {
    mod_root.as_ref().join(relative_document_path.as_ref())
}

fn default_scene_document_version() -> u32 {
    1
}

fn default_vec2_zero() -> SceneVec2Document {
    SceneVec2Document::ZERO
}

fn default_vec2_one() -> SceneVec2Document {
    SceneVec2Document::ONE
}

fn default_vec3_zero() -> SceneVec3Document {
    SceneVec3Document::ZERO
}

fn default_vec3_one() -> SceneVec3Document {
    SceneVec3Document::ONE
}

fn default_sprite_sheet_fps() -> f32 {
    8.0
}

fn default_sprite_sheet_looping() -> bool {
    true
}

fn default_gravity_scale() -> f32 {
    1.0
}

fn default_vector_segments() -> u32 {
    16
}

fn default_vector_stroke_width() -> f32 {
    1.0
}

fn default_particle_spawn_rate() -> f32 {
    10.0
}

fn default_particle_max_particles() -> usize {
    128
}

fn default_particle_lifetime() -> f32 {
    1.0
}

fn default_particle_initial_size() -> f32 {
    1.0
}

fn default_particle_final_size() -> f32 {
    1.0
}

fn default_ui_font_size() -> f32 {
    16.0
}

fn default_camera_follow_lerp() -> f32 {
    1.0
}

fn default_bounds_restitution() -> f32 {
    1.0
}

fn default_entity_lifecycle_flag() -> bool {
    true
}

fn default_once_per_overlap() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        SceneComponentDocument, SceneEntitySelectorDocument, SceneEntitySelectorKindDocument,
        load_scene_document_from_path, load_scene_document_from_str,
    };

    #[test]
    fn parses_scene_document_from_yaml() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: sprite-lab
  label: Sprite Lab
entities:
  - id: camera
    name: playground-2d-camera
    components:
      - type: Camera2D
  - id: sprite
    name: playground-2d-sprite
    transform2:
      translation: { x: 12.0, y: -4.0 }
    components:
      - type: Sprite2D
        texture: playground-2d/textures/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
        )
        .expect("scene document should parse");

        assert_eq!(document.scene.id, "sprite-lab");
        assert_eq!(document.entities.len(), 2);
        assert_eq!(document.entity_names()[1], "playground-2d-sprite");
        assert_eq!(
            document.component_kind_counts().get("Sprite2D"),
            Some(&1usize)
        );
        assert!(matches!(
            document.entities[1].components[0],
            SceneComponentDocument::Sprite2d { .. }
        ));
    }

    #[test]
    fn parses_entity_lifecycle_groups_and_properties() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: metadata-preview
entities:
  - id: actor
    tags: [enemy, flying]
    groups: [wave-1]
    visible: false
    simulation_enabled: true
    collision_enabled: false
    properties:
      score_value: 100
      speed: 2.5
      elite: true
      label: scout
"#,
        )
        .expect("scene document should parse");

        let entity = &document.entities[0];
        assert_eq!(entity.tags, vec!["enemy".to_owned(), "flying".to_owned()]);
        assert_eq!(entity.groups, vec!["wave-1".to_owned()]);
        assert!(!entity.visible);
        assert!(entity.simulation_enabled);
        assert!(!entity.collision_enabled);
        assert!(entity.properties.contains_key("score_value"));
        assert!(entity.properties.contains_key("speed"));
        assert!(entity.properties.contains_key("elite"));
        assert!(entity.properties.contains_key("label"));
    }

    #[test]
    fn parses_entity_selector_documents_from_yaml() {
        let selectors = serde_yaml::from_str::<Vec<SceneEntitySelectorDocument>>(
            r#"
- kind: entity
  value: player
- kind: tag
  value: enemy
- kind: group
  value: wave-1
- kind: pool
  value: bullets
"#,
        )
        .expect("selector documents should parse");

        assert_eq!(
            selectors,
            vec![
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Entity,
                    value: "player".to_owned(),
                },
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Tag,
                    value: "enemy".to_owned(),
                },
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Group,
                    value: "wave-1".to_owned(),
                },
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Pool,
                    value: "bullets".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_collision_event_rules_from_yaml() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: collision-preview
collision_events:
  - id: projectile-hits-target
    source:
      kind: tag
      value: projectile
    target:
      kind: group
      value: targets
    event: collision.hit
entities: []
"#,
        )
        .expect("scene document should parse");

        assert_eq!(document.collision_events.len(), 1);
        assert_eq!(document.collision_events[0].id, "projectile-hits-target");
        assert_eq!(
            document.collision_events[0].source,
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Tag,
                value: "projectile".to_owned(),
            }
        );
        assert!(document.collision_events[0].once_per_overlap);
    }

    #[test]
    fn parses_playground_scene_documents_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let sprite_doc = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/sprite-lab/scene.yml"),
        )
        .expect("sprite lab scene should parse");
        let material_doc = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
        )
        .expect("material lab scene should parse");

        assert_eq!(sprite_doc.scene.id, "sprite-lab");
        assert_eq!(material_doc.scene.id, "material-lab");
        assert!(sprite_doc.component_kind_counts().contains_key("Sprite2D"));
        assert!(
            material_doc
                .component_kind_counts()
                .contains_key("Material3D")
        );
    }

    #[test]
    fn parses_playground_2d_main_scene_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/hello-world-spritesheet/scene.yml"),
        )
        .expect("playground 2d main scene should parse");

        assert_eq!(document.scene.id, "hello-world-spritesheet");
        assert_eq!(document.transitions.len(), 1);
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(document.component_kind_counts().contains_key("Text2D"));
    }

    #[test]
    fn parses_playground_3d_main_scene_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/hello-world-cube/scene.yml"),
        )
        .expect("playground 3d main scene should parse");

        assert_eq!(document.scene.id, "hello-world-cube");
        assert!(document.component_kind_counts().contains_key("Mesh3D"));
        assert!(document.component_kind_counts().contains_key("Material3D"));
        assert!(document.component_kind_counts().contains_key("Text3D"));
    }

    #[test]
    fn parses_playground_2d_screen_space_preview_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/screen-space-preview/scene.yml"),
        )
        .expect("screen-space preview scene should parse");

        assert_eq!(document.scene.id, "screen-space-preview");
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(document.component_kind_counts().contains_key("UiDocument"));
    }

    #[test]
    fn parses_playground_2d_asteroids_game_from_disk() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d-asteroids/scenes/game/scene.yml"),
        )
        .expect("asteroids game scene should parse");

        assert_eq!(document.scene.id, "game");
        assert!(
            document
                .component_kind_counts()
                .contains_key("VectorShape2D")
        );
    }

    #[test]
    fn parses_sidescroller_component_document_from_yaml() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: vertical-slice
  label: Vertical Slice
entities:
  - id: camera
    name: playground-sidescroller-camera
    components:
      - type: Camera2D
      - type: CameraFollow2D
        target: playground-sidescroller-player
  - id: tilemap
    name: playground-sidescroller-tilemap
    components:
      - type: TileMap2D
        tileset: playground-sidescroller/tilesets/platformer
        ruleset: playground-sidescroller/tilesets/platformer-rules
        tile_size: { x: 16.0, y: 16.0 }
        grid:
          - "...."
          - ".P.."
          - "####"
  - id: player
    name: playground-sidescroller-player
    components:
      - type: TileMapMarker2D
        tilemap_entity: playground-sidescroller-tilemap
        symbol: "P"
        offset: { x: 0.0, y: 8.0 }
      - type: KinematicBody2D
        velocity: { x: 0.0, y: 0.0 }
        gravity_scale: 1.0
        terminal_velocity: 720.0
      - type: AabbCollider2D
        size: { x: 20.0, y: 30.0 }
        offset: { x: 0.0, y: 1.0 }
        layer: player
        mask: [world, trigger]
      - type: MotionController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
  - id: coin
    name: playground-sidescroller-coin
    components:
      - type: Sprite2D
        texture: playground-sidescroller/textures/coin
        size: { x: 16.0, y: 16.0 }
        animation:
          fps: 10.0
          looping: true
      - type: Trigger2D
        size: { x: 16.0, y: 16.0 }
        layer: trigger
        mask: [player]
        event: coin.collected
"#####,
        )
        .expect("sidescroller scene document should parse");

        assert_eq!(document.scene.id, "vertical-slice");
        assert!(document.component_kind_counts().contains_key("TileMap2D"));
        let tilemap_component = document
            .entities
            .iter()
            .find(|entity| entity.name == "playground-sidescroller-tilemap")
            .and_then(|entity| {
                entity
                    .components
                    .iter()
                    .find(|component| matches!(component, SceneComponentDocument::TileMap2d { .. }))
            })
            .expect("tilemap component should exist");
        match tilemap_component {
            SceneComponentDocument::TileMap2d { ruleset, .. } => {
                assert_eq!(
                    ruleset.as_deref(),
                    Some("playground-sidescroller/tilesets/platformer-rules")
                );
            }
            _ => unreachable!("expected tilemap component"),
        }
        assert!(
            document
                .component_kind_counts()
                .contains_key("KinematicBody2D")
        );
        assert!(
            document
                .component_kind_counts()
                .contains_key("AabbCollider2D")
        );
        assert!(document.component_kind_counts().contains_key("Trigger2D"));
        assert!(
            document
                .component_kind_counts()
                .contains_key("MotionController2D")
        );
        assert!(document.component_kind_counts().contains_key("Sprite2D"));
        assert!(
            document
                .component_kind_counts()
                .contains_key("CameraFollow2D")
        );
        assert!(
            document
                .component_kind_counts()
                .contains_key("TileMapMarker2D")
        );
    }

    #[test]
    fn rejects_legacy_platformer_controller_component_alias() {
        let result = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: legacy-motion-alias
  label: Legacy Motion Alias
entities:
  - id: player
    components:
      - type: PlatformerController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
"#,
        );

        assert!(result.is_err());
    }
}
