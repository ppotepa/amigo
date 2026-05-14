use std::sync::Mutex;

use crate::model::{SpriteDrawCommand, SpriteSheet};
use amigo_assets::AssetKey;

#[derive(Debug, Default)]
pub struct SpriteSceneService {
    commands: Mutex<Vec<SpriteDrawCommand>>,
}

impl SpriteSceneService {
    pub fn queue(&self, command: SpriteDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("sprite scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("sprite scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<SpriteDrawCommand> {
        let commands = self
            .commands
            .lock()
            .expect("sprite scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn set_frame(&self, entity_name: &str, frame_index: u32) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("sprite scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        let Some(sheet) = command.sprite.sheet else {
            return false;
        };
        command.sprite.frame_index = frame_index.min(sheet.visible_frame_count().saturating_sub(1));
        command.sprite.frame_elapsed = 0.0;
        true
    }

    pub fn advance_animation(&self, entity_name: &str, delta_seconds: f32) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("sprite scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        let Some(sheet) = command.sprite.sheet else {
            return false;
        };
        if sheet.fps <= f32::EPSILON || sheet.visible_frame_count() <= 1 {
            return false;
        }

        let frame_duration = 1.0 / sheet.fps;
        command.sprite.frame_elapsed += delta_seconds.max(0.0);

        while command.sprite.frame_elapsed >= frame_duration {
            command.sprite.frame_elapsed -= frame_duration;
            if command.sprite.frame_index + 1 >= sheet.visible_frame_count() {
                if sheet.looping {
                    command.sprite.frame_index = 0;
                } else {
                    command.sprite.frame_index = sheet.visible_frame_count().saturating_sub(1);
                    command.sprite.frame_elapsed = 0.0;
                    break;
                }
            } else {
                command.sprite.frame_index += 1;
            }
        }

        true
    }

    pub fn sync_sheet_for_texture(&self, texture: &AssetKey, sheet: SpriteSheet) -> usize {
        let mut commands = self
            .commands
            .lock()
            .expect("sprite scene service mutex should not be poisoned");
        let mut updated = 0;

        for command in commands.iter_mut() {
            if &command.sprite.texture != texture {
                continue;
            }

            let base_sheet = if command.sprite.sheet_is_explicit {
                command.sprite.sheet.unwrap_or(sheet)
            } else {
                sheet
            };
            let merged_sheet = crate::scene_bridge::apply_animation_override(
                base_sheet,
                command.sprite.animation_override,
            );
            command.sprite.sheet = Some(merged_sheet);
            if let Some(start_frame) = command
                .sprite
                .animation_override
                .and_then(|override_| override_.start_frame)
            {
                command.sprite.frame_index =
                    start_frame.min(merged_sheet.visible_frame_count().saturating_sub(1));
            } else {
                command.sprite.frame_index = command
                    .sprite
                    .frame_index
                    .min(merged_sheet.visible_frame_count().saturating_sub(1));
            }
            updated += 1;
        }

        updated
    }

    pub fn frame_of(&self, entity_name: &str) -> Option<u32> {
        self.commands()
            .into_iter()
            .find(|command| command.entity_name == entity_name)
            .map(|command| command.sprite.frame_index)
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}
