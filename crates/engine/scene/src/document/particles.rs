use serde::{Deserialize, Serialize};

use super::defaults::*;
use super::render_values::SceneVec2Document;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticleLineAnchor2dSceneDocument {
    Center,
    Start,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleShapeChoice2dSceneDocument {
    pub shape: ParticleShape2dSceneDocument,
    #[serde(default = "default_particle_shape_choice_weight")]
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleShapeKeyframe2dSceneDocument {
    pub t: f32,
    pub shape: ParticleShape2dSceneDocument,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ParticleMotionStretch2dSceneDocument {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub velocity_scale: f32,
    #[serde(default)]
    pub max_length: f32,
    #[serde(default)]
    pub shutter_seconds: f32,
    #[serde(default = "default_particle_motion_stretch_alpha")]
    pub tail_alpha: f32,
    #[serde(default = "default_particle_motion_stretch_alpha")]
    pub head_alpha: f32,
}

fn default_particle_motion_stretch_alpha() -> f32 {
    1.0
}

fn default_particle_receives_light() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleMaterial2dSceneDocument {
    #[serde(default)]
    pub lighting_mode: Option<Material2dLightingModeSceneDocument>,
    #[serde(default = "default_particle_receives_light")]
    pub receives_light: bool,
    #[serde(default = "default_particle_light_response")]
    pub light_response: f32,
    #[serde(default, alias = "lightmap")]
    pub light_receiver: Option<LightReceiver2dBindingSceneDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Material2dLightingModeSceneDocument {
    Unlit,
    DynamicLights,
    LightmapSampled,
    LightGroupSampled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LightSampleStrategy2dSceneDocument {
    Point,
    Line,
}

impl Default for LightSampleStrategy2dSceneDocument {
    fn default() -> Self {
        Self::Line
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LightReceiverDarkPolicy2dSceneDocument {
    Transparent,
    BaseColor,
    ShadowTint,
}

impl Default for LightReceiverDarkPolicy2dSceneDocument {
    fn default() -> Self {
        Self::Transparent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightReceiverGlobalLight2dSceneDocument {
    pub id: String,
    #[serde(default = "default_light_receiver_global_response")]
    pub response: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightReceiver2dBindingSceneDocument {
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub sample_strategy: LightSampleStrategy2dSceneDocument,
    #[serde(default = "default_lightmap_sample_points")]
    pub sample_points: u32,
    #[serde(default = "default_lightmap_sample_radius_px")]
    pub radius_px: f32,
    #[serde(default = "default_lightmap_exposure")]
    pub exposure: f32,
    #[serde(default)]
    pub dark_policy: LightReceiverDarkPolicy2dSceneDocument,
    #[serde(default)]
    pub global_lights: Vec<LightReceiverGlobalLight2dSceneDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ParticleLight2dSceneDocument {
    pub radius: f32,
    pub intensity: f32,
    #[serde(default)]
    pub mode: ParticleLightMode2dSceneDocument,
    #[serde(default)]
    pub glow: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticleLightMode2dSceneDocument {
    Source,
    Particle,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticleVelocityMode2dSceneDocument {
    Free,
    SourceInertial,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticleSimulationSpace2dSceneDocument {
    World,
    Source,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticleAlignMode2dSceneDocument {
    None,
    Velocity,
    Emitter,
    Random,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticleBlendMode2dSceneDocument {
    Alpha,
    Additive,
    Multiply,
    Screen,
}
