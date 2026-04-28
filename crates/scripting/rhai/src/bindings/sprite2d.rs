use std::sync::Arc;

use amigo_2d_sprite::SpriteSceneService;
use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::queue_sprite_spawn;

#[derive(Clone)]
pub struct Sprite2dApi {
    pub(crate) sprite_scene: Option<Arc<SpriteSceneService>>,
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Sprite2dApi {
    pub fn frame(&mut self, entity_name: &str) -> rhai::INT {
        sprite_frame(self.sprite_scene.as_ref(), entity_name)
    }

    pub fn set_frame(&mut self, entity_name: &str, frame_index: rhai::INT) -> bool {
        if frame_index < 0 {
            return false;
        }
        set_sprite_frame(self.sprite_scene.as_ref(), entity_name, frame_index as u32)
    }

    pub fn advance(&mut self, entity_name: &str, delta_seconds: rhai::FLOAT) -> bool {
        advance_sprite_animation(
            self.sprite_scene.as_ref(),
            entity_name,
            delta_seconds as f32,
        )
    }

    pub fn queue(
        &mut self,
        entity_name: &str,
        texture_key: &str,
        width: rhai::INT,
        height: rhai::INT,
    ) -> bool {
        if width <= 0 || height <= 0 {
            return false;
        }
        queue_sprite_spawn(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            texture_key,
            width,
            height,
        )
    }
}

pub fn sprite_frame(
    sprite_scene: Option<&Arc<SpriteSceneService>>,
    entity_name: &str,
) -> rhai::INT {
    sprite_scene
        .and_then(|sprite_scene| sprite_scene.frame_of(entity_name))
        .map(|frame| frame as rhai::INT)
        .unwrap_or(0)
}

pub fn set_sprite_frame(
    sprite_scene: Option<&Arc<SpriteSceneService>>,
    entity_name: &str,
    frame_index: u32,
) -> bool {
    sprite_scene
        .map(|sprite_scene| sprite_scene.set_frame(entity_name, frame_index))
        .unwrap_or(false)
}

pub fn advance_sprite_animation(
    sprite_scene: Option<&Arc<SpriteSceneService>>,
    entity_name: &str,
    delta_seconds: f32,
) -> bool {
    sprite_scene
        .map(|sprite_scene| sprite_scene.advance_animation(entity_name, delta_seconds))
        .unwrap_or(false)
}
