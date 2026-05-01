use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::behavior::*;
use super::core::*;
use super::defaults::*;
use super::particles::*;
use super::render_values::*;
use super::ui::*;

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
        #[serde(default)]
        velocity_mode: Option<ParticleVelocityMode2dSceneDocument>,
        #[serde(default)]
        simulation_space: Option<ParticleSimulationSpace2dSceneDocument>,
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
        shape_choices: Vec<ParticleShapeChoice2dSceneDocument>,
        #[serde(default)]
        shape_over_lifetime: Vec<ParticleShapeKeyframe2dSceneDocument>,
        #[serde(default)]
        line_anchor: Option<ParticleLineAnchor2dSceneDocument>,
        #[serde(default)]
        align: Option<ParticleAlignMode2dSceneDocument>,
        #[serde(default)]
        blend_mode: Option<ParticleBlendMode2dSceneDocument>,
        #[serde(default)]
        motion_stretch: Option<ParticleMotionStretch2dSceneDocument>,
        #[serde(default)]
        material: Option<ParticleMaterial2dSceneDocument>,
        #[serde(default)]
        light: Option<ParticleLight2dSceneDocument>,
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
            Self::InputActionMap { .. } => "InputActionMap",
            Self::Behavior { .. } => "Behavior",
            Self::EventPipeline { .. } => "EventPipeline",
            Self::UiModelBindings { .. } => "UiModelBindings",
            Self::ScriptComponent { .. } => "ScriptComponent",
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
