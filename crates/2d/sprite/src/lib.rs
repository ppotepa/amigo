use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
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

#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: AssetKey,
    pub size: Vec2,
    pub sheet: Option<SpriteSheet>,
    pub frame_index: u32,
    pub frame_elapsed: f32,
}

#[derive(Debug, Clone)]
pub struct SpriteDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub sprite: Sprite,
    pub transform: Transform2,
}

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

#[derive(Debug, Clone)]
pub struct SpriteDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct SpritePlugin;

impl RuntimePlugin for SpritePlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-sprite"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SpriteSceneService::default())?;
        registry.register(SpriteDomainInfo {
            crate_name: "amigo-2d-sprite",
            capability: "rendering_2d",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Sprite, SpriteDrawCommand, SpriteSceneService, SpriteSheet};
    use amigo_assets::AssetKey;
    use amigo_math::{Transform2, Vec2};
    use amigo_scene::SceneEntityId;

    #[test]
    fn stores_sprite_draw_commands() {
        let service = SpriteSceneService::default();

        service.queue(SpriteDrawCommand {
            entity_id: SceneEntityId::new(7),
            entity_name: "playground-2d-sprite".to_owned(),
            sprite: Sprite {
                texture: AssetKey::new("playground-2d/textures/sprite-lab"),
                size: Vec2::new(128.0, 128.0),
                sheet: None,
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            transform: Transform2::default(),
        });

        assert_eq!(service.commands().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["playground-2d-sprite".to_owned()]
        );

        service.clear();
        assert!(service.commands().is_empty());
    }

    #[test]
    fn advances_sprite_sheet_animation_frames() {
        let service = SpriteSceneService::default();
        service.queue(SpriteDrawCommand {
            entity_id: SceneEntityId::new(11),
            entity_name: "playground-2d-spritesheet".to_owned(),
            sprite: Sprite {
                texture: AssetKey::new("playground-2d/textures/hello-world-spritesheet"),
                size: Vec2::new(256.0, 128.0),
                sheet: Some(SpriteSheet {
                    columns: 4,
                    rows: 2,
                    frame_count: 8,
                    frame_size: Vec2::new(32.0, 32.0),
                    fps: 8.0,
                    looping: true,
                }),
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            transform: Transform2::default(),
        });

        assert!(service.advance_animation("playground-2d-spritesheet", 0.25));
        assert_eq!(service.frame_of("playground-2d-spritesheet"), Some(2));
        assert!(service.set_frame("playground-2d-spritesheet", 7));
        assert_eq!(service.frame_of("playground-2d-spritesheet"), Some(7));
        assert!(service.advance_animation("playground-2d-spritesheet", 0.125));
        assert_eq!(service.frame_of("playground-2d-spritesheet"), Some(0));
    }
}
