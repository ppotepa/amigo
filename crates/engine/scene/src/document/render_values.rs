use serde::{Deserialize, Serialize};

use super::defaults::*;

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

