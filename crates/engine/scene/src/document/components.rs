use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::behavior::*;
use super::core::*;
use super::defaults::*;
use super::render_values::{SceneVec2Document, SceneVec3Document};
use super::ui::*;
use super::visual2d::PostFx2dDocument;

impl SceneEntityDocument {
    pub fn display_name(&self) -> String {
        if self.name.trim().is_empty() {
            self.id.clone()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneComponentSemanticClass {
    Sprite2d,
    LayeredImage2d,
    TileMap2d,
    Text2d,
    VectorShape2d,
    ParticleEmitter2d,
    BeaconLight2d,
    Camera2d,
    Motion2d,
    Physics2d,
    Physics3d,
    Script,
    Plugin,
    Generic2d,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NprLine3dDocument {
    Enabled(bool),
    Settings(NprLine3dSettingsDocument),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NprLine3dSettingsDocument {
    #[serde(default = "default_bool_true")]
    pub enabled: bool,
    #[serde(default = "default_bool_true")]
    pub boundary: bool,
    #[serde(default = "default_bool_true")]
    pub silhouette: bool,
    #[serde(default = "default_bool_true")]
    pub feature: bool,
    #[serde(default = "default_npr_feature_angle_degrees")]
    pub feature_angle_degrees: f32,
    #[serde(default = "default_npr_min_screen_length_px")]
    pub min_screen_length_px: f32,
    #[serde(default = "default_npr_ink_color")]
    pub ink_color: String,
    #[serde(default = "default_npr_width_px")]
    pub width_px: f32,
    #[serde(default = "default_npr_width_jitter_px")]
    pub width_jitter_px: f32,
    #[serde(default = "default_npr_path_jitter_px")]
    pub path_jitter_px: f32,
    #[serde(default = "default_npr_taper")]
    pub taper: f32,
    #[serde(default = "default_npr_overshoot_px")]
    pub overshoot_px: f32,
    #[serde(default = "default_npr_dropout")]
    pub dropout: f32,
    #[serde(default = "default_npr_passes")]
    pub passes: u8,
    #[serde(default = "default_npr_seed")]
    pub seed: u64,
}

fn default_bool_true() -> bool {
    true
}

fn default_npr_feature_angle_degrees() -> f32 {
    32.0
}

fn default_npr_min_screen_length_px() -> f32 {
    2.0
}

fn default_npr_ink_color() -> String {
    "#100E0BFF".to_owned()
}

fn default_npr_width_px() -> f32 {
    2.4
}

fn default_npr_width_jitter_px() -> f32 {
    0.55
}

fn default_npr_path_jitter_px() -> f32 {
    0.75
}

fn default_npr_taper() -> f32 {
    0.65
}

fn default_npr_overshoot_px() -> f32 {
    1.8
}

fn default_npr_dropout() -> f32 {
    0.035
}

fn default_npr_passes() -> u8 {
    2
}

fn default_npr_seed() -> u64 {
    2002
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SceneComponentDocumentModel {
    #[serde(rename = "Camera3D")]
    Camera3d {
        #[serde(default = "default_camera3d_fov_y_degrees")]
        fov_y_degrees: f32,
        #[serde(default = "default_camera3d_near_clip")]
        near_clip: f32,
        #[serde(default = "default_camera3d_far_clip")]
        far_clip: f32,
    },
    #[serde(rename = "Light3D")]
    Light3d {
        #[serde(default)]
        kind: String,
        #[serde(default = "default_light3d_direction")]
        direction: SceneVec3Document,
        #[serde(default)]
        color: Option<String>,
        #[serde(default = "default_light3d_intensity")]
        intensity: f32,
        #[serde(default = "default_light3d_ambient")]
        ambient: f32,
    },
    #[serde(rename = "LightMap2DSource")]
    LightMap2dSource {
        id: String,
        source: LightMap2dSourceRefDocument,
        #[serde(default)]
        channels: Vec<LightMap2dChannelDocument>,
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
    #[serde(rename = "InputActionMap")]
    InputActionMap {
        id: String,
        #[serde(default)]
        active: bool,
        #[serde(default)]
        actions: BTreeMap<String, SceneInputActionBindingDocument>,
    },
    #[serde(rename = "Behavior")]
    Behavior {
        #[serde(default)]
        enabled_when: Option<SceneBehaviorConditionDocument>,
        #[serde(flatten)]
        behavior: SceneBehaviorDocument,
    },
    #[serde(rename = "EventPipeline")]
    EventPipeline {
        id: String,
        topic: String,
        #[serde(default)]
        steps: Vec<SceneEventPipelineStepDocument>,
    },
    #[serde(rename = "UiModelBindings")]
    UiModelBindings {
        #[serde(default)]
        bindings: Vec<SceneUiModelBindingDocument>,
    },
    #[serde(rename = "ScriptComponent")]
    ScriptComponent {
        script: String,
        #[serde(default)]
        params: BTreeMap<String, ScenePropertyValueDocument>,
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
    #[serde(rename = "StaticCollider2D")]
    StaticCollider2d {
        size: SceneVec2Document,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        layer: String,
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
        #[serde(default)]
        lookahead_velocity_scale: f32,
        #[serde(default)]
        lookahead_max_distance: f32,
        #[serde(default)]
        sway_amount: f32,
        #[serde(default)]
        sway_frequency: f32,
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
    Mesh3d {
        mesh: String,
        #[serde(default)]
        npr: Option<NprLine3dDocument>,
    },
    #[serde(rename = "Material3D")]
    Material3d {
        label: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        albedo: Option<String>,
        #[serde(default)]
        render_order: i32,
    },
    #[serde(rename = "Text3D")]
    Text3d {
        content: String,
        font: String,
        size: f32,
    },
    #[serde(rename = "PhysicsWorld3D")]
    PhysicsWorld3d {
        #[serde(default = "default_physics3d_gravity")]
        gravity: SceneVec3Document,
        #[serde(default = "default_physics3d_substeps")]
        substeps: u32,
        #[serde(default = "default_physics3d_solver_iterations")]
        solver_iterations: u32,
        #[serde(default = "default_physics3d_ccd_substeps")]
        ccd_substeps: u32,
    },
    #[serde(rename = "RigidBody3D")]
    RigidBody3d {
        #[serde(default = "default_vec3_zero")]
        velocity: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        angular_velocity: SceneVec3Document,
        #[serde(default = "default_rigid_body_mass_3d")]
        mass: f32,
        #[serde(default = "default_linear_damping_3d")]
        linear_damping: f32,
        #[serde(default = "default_angular_damping_3d")]
        angular_damping: f32,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        restitution: f32,
        #[serde(default = "default_rigid_body_friction_3d")]
        friction: f32,
        #[serde(default)]
        ccd: bool,
    },
    #[serde(rename = "BoxCollider3D")]
    BoxCollider3d {
        size: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        offset: SceneVec3Document,
    },
    #[serde(rename = "StaticBoxCollider3D")]
    StaticBoxCollider3d {
        size: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        offset: SceneVec3Document,
        #[serde(default = "default_rigid_body_friction_3d")]
        friction: f32,
        #[serde(default)]
        restitution: f32,
    },
    #[serde(rename = "PhysicsSpawner3D")]
    PhysicsSpawner3d {
        entity_prefix: String,
        mesh: String,
        material: String,
        #[serde(default)]
        material_label: Option<String>,
        #[serde(default = "default_spawn_interval_seconds")]
        interval_seconds: f32,
        #[serde(default = "default_vec3_zero")]
        origin: SceneVec3Document,
        #[serde(default = "default_vec3_one")]
        spawn_scale: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        grid_spacing: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        initial_velocity: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        angular_velocity: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        spawn_position_jitter: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        spawn_rotation_jitter: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        initial_velocity_jitter: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        angular_velocity_jitter: SceneVec3Document,
        #[serde(default = "default_rigid_body_mass_3d")]
        mass: f32,
        #[serde(default = "default_linear_damping_3d")]
        linear_damping: f32,
        #[serde(default = "default_angular_damping_3d")]
        angular_damping: f32,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        restitution: f32,
        #[serde(default = "default_rigid_body_friction_3d")]
        friction: f32,
        #[serde(default)]
        ccd: bool,
        #[serde(default = "default_vec3_one")]
        collider_size: SceneVec3Document,
        #[serde(default)]
        max_alive: u32,
        #[serde(default)]
        counter_entity: Option<String>,
        #[serde(default = "default_physics3d_counter_prefix")]
        counter_prefix: String,
        #[serde(default)]
        counter_font: Option<String>,
        #[serde(default = "default_physics3d_counter_size")]
        counter_size: f32,
        #[serde(default = "default_vec3_zero")]
        counter_position: SceneVec3Document,
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
    Plugin {
        component_type: String,
        payload: Value,
    },
}

pub type SceneComponentDocument = SceneComponentDocumentModel;

pub fn is_builtin_component_type(kind: &str) -> bool {
    matches!(
        kind,
        "Camera3D"
            | "Light3D"
            | "LightMap2DSource"
            | "EntityPool"
            | "Lifetime"
            | "ProjectileEmitter2D"
            | "InputActionMap"
            | "Behavior"
            | "EventPipeline"
            | "UiModelBindings"
            | "ScriptComponent"
            | "Velocity2D"
            | "Bounds2D"
            | "FreeflightMotion2D"
            | "KinematicBody2D"
            | "AabbCollider2D"
            | "StaticCollider2D"
            | "CircleCollider2D"
            | "Trigger2D"
            | "MotionController2D"
            | "CameraFollow2D"
            | "Parallax2D"
            | "TileMapMarker2D"
            | "Mesh3D"
            | "Material3D"
            | "Text3D"
            | "PhysicsWorld3D"
            | "RigidBody3D"
            | "BoxCollider3D"
            | "StaticBoxCollider3D"
            | "PhysicsSpawner3D"
            | "UiDocument"
            | "UiThemeSet"
    )
}

pub fn is_rejected_retired_component_type(kind: &str) -> bool {
    matches!(kind, "PlatformerController2D")
}

pub fn plugin_component_document(component_type: String, payload: Value) -> SceneComponentDocument {
    type ComponentDocument = SceneComponentDocument;

    ComponentDocument::Plugin {
        component_type,
        payload,
    }
}

impl SceneComponentDocument {
    pub fn kind(&self) -> &str {
        match self {
            Self::Camera3d { .. } => "Camera3D",
            Self::Light3d { .. } => "Light3D",
            Self::LightMap2dSource { .. } => "LightMap2DSource",
            Self::EntityPool { .. } => "EntityPool",
            Self::Lifetime { .. } => "Lifetime",
            Self::ProjectileEmitter2d { .. } => "ProjectileEmitter2D",
            Self::InputActionMap { .. } => "InputActionMap",
            Self::Behavior { .. } => "Behavior",
            Self::EventPipeline { .. } => "EventPipeline",
            Self::UiModelBindings { .. } => "UiModelBindings",
            Self::ScriptComponent { .. } => "ScriptComponent",
            Self::Velocity2d { .. } => "Velocity2D",
            Self::Bounds2d { .. } => "Bounds2D",
            Self::FreeflightMotion2d { .. } => "FreeflightMotion2D",
            Self::KinematicBody2d { .. } => "KinematicBody2D",
            Self::AabbCollider2d { .. } => "AabbCollider2D",
            Self::StaticCollider2d { .. } => "StaticCollider2D",
            Self::CircleCollider2d { .. } => "CircleCollider2D",
            Self::Trigger2d { .. } => "Trigger2D",
            Self::MotionController2d { .. } => "MotionController2D",
            Self::CameraFollow2d { .. } => "CameraFollow2D",
            Self::Parallax2d { .. } => "Parallax2D",
            Self::TileMapMarker2d { .. } => "TileMapMarker2D",
            Self::Mesh3d { .. } => "Mesh3D",
            Self::Material3d { .. } => "Material3D",
            Self::Text3d { .. } => "Text3D",
            Self::PhysicsWorld3d { .. } => "PhysicsWorld3D",
            Self::RigidBody3d { .. } => "RigidBody3D",
            Self::BoxCollider3d { .. } => "BoxCollider3D",
            Self::StaticBoxCollider3d { .. } => "StaticBoxCollider3D",
            Self::PhysicsSpawner3d { .. } => "PhysicsSpawner3D",
            Self::UiDocument { .. } => "UiDocument",
            Self::UiThemeSet { .. } => "UiThemeSet",
            Self::Plugin { component_type, .. } => component_type.as_str(),
        }
    }

    pub fn primary_render_layer(&self) -> Option<&str> {
        None
    }

    pub fn plugin_payload(&self) -> Option<(&str, &Value)> {
        match self {
            Self::Plugin {
                component_type,
                payload,
            } => Some((component_type.as_str(), payload)),
            _ => None,
        }
    }

    pub fn post_fx_documents(&self) -> Option<&[PostFx2dDocument]> {
        None
    }

    pub fn layered_image_part_post_fx_documents(&self) -> Option<Vec<(&str, &[PostFx2dDocument])>> {
        None
    }

    pub fn is_particle_emitter_2d(&self) -> bool {
        false
    }

    pub fn semantic_class(&self) -> SceneComponentSemanticClass {
        match self {
            Self::CameraFollow2d { .. } => SceneComponentSemanticClass::Camera2d,
            Self::Velocity2d { .. }
            | Self::FreeflightMotion2d { .. }
            | Self::MotionController2d { .. } => SceneComponentSemanticClass::Motion2d,
            Self::KinematicBody2d { .. }
            | Self::AabbCollider2d { .. }
            | Self::StaticCollider2d { .. }
            | Self::CircleCollider2d { .. }
            | Self::Trigger2d { .. } => SceneComponentSemanticClass::Physics2d,
            Self::PhysicsWorld3d { .. }
            | Self::RigidBody3d { .. }
            | Self::BoxCollider3d { .. }
            | Self::StaticBoxCollider3d { .. }
            | Self::PhysicsSpawner3d { .. } => SceneComponentSemanticClass::Physics3d,
            Self::ScriptComponent { .. } => SceneComponentSemanticClass::Script,
            Self::Plugin { .. } => SceneComponentSemanticClass::Plugin,
            _ => SceneComponentSemanticClass::Generic2d,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TileMap2dEditorDocument {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub lock_size: bool,
    #[serde(default)]
    pub show_grid: bool,
    #[serde(default)]
    pub default_brush: Option<String>,
    #[serde(default)]
    pub snap: Option<String>,
    #[serde(default)]
    pub palette: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LayeredImageBlendMode2dDocument {
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LayeredImageViewportFit2dDocument {
    #[default]
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepthAuxMap2dChannelsDocument {
    #[serde(default = "default_depth_aux_r_channel")]
    pub r: String,
    #[serde(default = "default_depth_aux_g_channel")]
    pub g: String,
    #[serde(default = "default_depth_aux_b_channel")]
    pub b: String,
    #[serde(default = "default_depth_aux_a_channel")]
    pub a: String,
}

impl Default for DepthAuxMap2dChannelsDocument {
    fn default() -> Self {
        Self {
            r: default_depth_aux_r_channel(),
            g: default_depth_aux_g_channel(),
            b: default_depth_aux_b_channel(),
            a: default_depth_aux_a_channel(),
        }
    }
}

fn default_depth_aux_r_channel() -> String {
    "auxiliary_depth".to_owned()
}

fn default_depth_aux_g_channel() -> String {
    "local_height".to_owned()
}

fn default_depth_aux_b_channel() -> String {
    "occluder_strength".to_owned()
}

fn default_depth_aux_a_channel() -> String {
    "valid_mask".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayeredImageLayerOverrideDocument {
    pub id: String,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub blend: Option<LayeredImageBlendMode2dDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_maps: Option<VisualMaps2dDocument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct VisualMaps2dDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wetness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissive: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LightMap2dSourceRefDocument {
    #[serde(rename = "layered_image_2d", alias = "layered_image2d")]
    LayeredImage2d { entity: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightMap2dChannelDocument {
    pub id: String,
    #[serde(default)]
    pub layers: Vec<String>,
}
