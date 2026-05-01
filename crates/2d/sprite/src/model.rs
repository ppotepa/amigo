use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteSheet {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
    pub fps: f32,
    pub looping: bool,
}

impl SpriteSheet {
    pub fn visible_frame_count(&self) -> u32 {
        self.frame_count
            .max(1)
            .min(self.columns.max(1).saturating_mul(self.rows.max(1)))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SpriteAnimationOverride {
    pub fps: Option<f32>,
    pub looping: Option<bool>,
    pub start_frame: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: AssetKey,
    pub size: Vec2,
    pub sheet: Option<SpriteSheet>,
    pub sheet_is_explicit: bool,
    pub animation_override: Option<SpriteAnimationOverride>,
    pub frame_index: u32,
    pub frame_elapsed: f32,
}

#[derive(Debug, Clone)]
pub struct SpriteDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub sprite: Sprite,
    pub z_index: f32,
    pub transform: Transform2,
}
