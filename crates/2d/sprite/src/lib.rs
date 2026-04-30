use std::sync::Mutex;

use amigo_assets::{AssetCatalog, AssetKey, PreparedAsset, PreparedAssetKind};
use amigo_math::{Transform2, Vec2};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    SceneEntityId, SceneService, Sprite2dSceneCommand, SpriteAnimation2dSceneOverride,
};

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
            let merged_sheet =
                apply_animation_override(base_sheet, command.sprite.animation_override);
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

pub fn queue_sprite_scene_command(
    scene_service: &SceneService,
    sprite_scene_service: &SpriteSceneService,
    command: &Sprite2dSceneCommand,
    resolved_sheet: Option<SpriteSheet>,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    sprite_scene_service.queue(SpriteDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        sprite: Sprite {
            texture: command.texture.clone(),
            size: command.size,
            sheet: resolved_sheet,
            sheet_is_explicit: command.sheet.is_some(),
            animation_override: command.animation.as_ref().map(|animation| {
                SpriteAnimationOverride {
                    fps: animation.fps,
                    looping: animation.looping,
                    start_frame: animation.start_frame,
                }
            }),
            frame_index: command
                .animation
                .as_ref()
                .and_then(|animation| animation.start_frame)
                .unwrap_or(0),
            frame_elapsed: 0.0,
        },
        z_index: command.z_index,
        transform: command.transform,
    });
    entity
}

pub fn infer_sprite_sheet_from_prepared_asset(prepared: &PreparedAsset) -> Option<SpriteSheet> {
    if !matches!(prepared.kind, PreparedAssetKind::SpriteSheet2d) {
        return None;
    }

    let columns = prepared
        .metadata
        .get("columns")?
        .parse::<u32>()
        .ok()?
        .max(1);
    let rows = prepared.metadata.get("rows")?.parse::<u32>().ok()?.max(1);
    let frame_width = prepared.metadata.get("frame_size.x")?.parse::<f32>().ok()?;
    let frame_height = prepared.metadata.get("frame_size.y")?.parse::<f32>().ok()?;
    let fps = prepared
        .metadata
        .get("fps")
        .and_then(|value| value.parse::<f32>().ok())
        .or_else(|| first_animation_f32(prepared, "fps"))
        .unwrap_or(0.0);
    let looping = prepared
        .metadata
        .get("looping")
        .and_then(|value| value.parse::<bool>().ok())
        .or_else(|| first_animation_bool(prepared, "looping"))
        .unwrap_or(true);

    Some(SpriteSheet {
        columns,
        rows,
        frame_count: prepared
            .metadata
            .get("frame_count")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(columns.saturating_mul(rows))
            .max(1),
        frame_size: Vec2::new(frame_width, frame_height),
        fps,
        looping,
    })
}

pub fn resolve_sprite_sheet_for_command(
    asset_catalog: &AssetCatalog,
    command: &Sprite2dSceneCommand,
) -> Option<SpriteSheet> {
    let explicit_sheet = command.sheet.as_ref().map(|sheet| SpriteSheet {
        columns: sheet.columns,
        rows: sheet.rows,
        frame_count: sheet.frame_count,
        frame_size: sheet.frame_size,
        fps: sheet.fps,
        looping: sheet.looping,
    });

    let base_sheet = explicit_sheet.or_else(|| {
        asset_catalog
            .prepared_asset(&command.texture)
            .and_then(|prepared| infer_sprite_sheet_from_prepared_asset(&prepared))
    })?;

    Some(apply_scene_animation_override(
        base_sheet,
        command.animation.as_ref(),
    ))
}

fn apply_animation_override(
    mut sheet: SpriteSheet,
    override_: Option<SpriteAnimationOverride>,
) -> SpriteSheet {
    let Some(override_) = override_ else {
        return sheet;
    };

    if let Some(fps) = override_.fps {
        sheet.fps = fps.max(0.0);
    }
    if let Some(looping) = override_.looping {
        sheet.looping = looping;
    }
    sheet
}

fn apply_scene_animation_override(
    mut sheet: SpriteSheet,
    animation: Option<&SpriteAnimation2dSceneOverride>,
) -> SpriteSheet {
    let Some(animation) = animation else {
        return sheet;
    };

    if let Some(fps) = animation.fps {
        sheet.fps = fps.max(0.0);
    }
    if let Some(looping) = animation.looping {
        sheet.looping = looping;
    }
    sheet
}

fn first_animation_f32(prepared: &PreparedAsset, field: &str) -> Option<f32> {
    let suffix = format!(".{field}");
    prepared.metadata.iter().find_map(|(key, value)| {
        (key.starts_with("animations.") && key.ends_with(&suffix))
            .then(|| value.parse::<f32>().ok())
            .flatten()
    })
}

fn first_animation_bool(prepared: &PreparedAsset, field: &str) -> Option<bool> {
    let suffix = format!(".{field}");
    prepared.metadata.iter().find_map(|(key, value)| {
        (key.starts_with("animations.") && key.ends_with(&suffix))
            .then(|| value.parse::<bool>().ok())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::{
        Sprite, SpriteAnimationOverride, SpriteDrawCommand, SpriteSceneService, SpriteSheet,
        infer_sprite_sheet_from_prepared_asset, queue_sprite_scene_command,
        resolve_sprite_sheet_for_command,
    };
    use amigo_assets::{
        AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest,
        AssetSourceKind, LoadedAsset, prepare_asset_from_contents,
    };
    use amigo_math::{Transform2, Vec2};
    use amigo_scene::{
        SceneEntityId, SceneService, Sprite2dSceneCommand, SpriteAnimation2dSceneOverride,
    };
    use std::path::PathBuf;

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
                sheet_is_explicit: false,
                animation_override: None,
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            z_index: 0.0,
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
                sheet_is_explicit: true,
                animation_override: None,
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            z_index: 0.0,
            transform: Transform2::default(),
        });

        assert!(service.advance_animation("playground-2d-spritesheet", 0.25));
        assert_eq!(service.frame_of("playground-2d-spritesheet"), Some(2));
        assert!(service.set_frame("playground-2d-spritesheet", 7));
        assert_eq!(service.frame_of("playground-2d-spritesheet"), Some(7));
        assert!(service.advance_animation("playground-2d-spritesheet", 0.125));
        assert_eq!(service.frame_of("playground-2d-spritesheet"), Some(0));
    }

    #[test]
    fn syncs_sheet_metadata_for_matching_texture() {
        let service = SpriteSceneService::default();
        let texture = AssetKey::new("playground-sidescroller/textures/coin");
        service.queue(SpriteDrawCommand {
            entity_id: SceneEntityId::new(13),
            entity_name: "playground-sidescroller-coin".to_owned(),
            sprite: Sprite {
                texture: texture.clone(),
                size: Vec2::new(16.0, 16.0),
                sheet: None,
                sheet_is_explicit: false,
                animation_override: Some(SpriteAnimationOverride {
                    fps: Some(8.0),
                    looping: Some(true),
                    start_frame: Some(1),
                }),
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            z_index: 0.0,
            transform: Transform2::default(),
        });

        let updated = service.sync_sheet_for_texture(
            &texture,
            SpriteSheet {
                columns: 4,
                rows: 1,
                frame_count: 4,
                frame_size: Vec2::new(16.0, 16.0),
                fps: 8.0,
                looping: true,
            },
        );

        assert_eq!(updated, 1);
        assert_eq!(service.frame_of("playground-sidescroller-coin"), Some(1));
        assert!(service.advance_animation("playground-sidescroller-coin", 0.25));
        assert_eq!(service.frame_of("playground-sidescroller-coin"), Some(3));
    }

    #[test]
    fn queues_sprite_scene_command() {
        let scene = SceneService::default();
        let service = SpriteSceneService::default();

        let mut command = Sprite2dSceneCommand::new(
            "playground-2d",
            "playground-2d-sprite",
            AssetKey::new("playground-2d/textures/sprite-lab"),
            Vec2::new(128.0, 128.0),
        );
        command.animation = Some(SpriteAnimation2dSceneOverride {
            fps: Some(8.0),
            looping: Some(true),
            start_frame: Some(1),
        });

        let entity = queue_sprite_scene_command(
            &scene,
            &service,
            &command,
            Some(SpriteSheet {
                columns: 4,
                rows: 1,
                frame_count: 4,
                frame_size: Vec2::new(32.0, 32.0),
                fps: 8.0,
                looping: true,
            }),
        );

        assert_eq!(entity.raw(), 0);
        assert_eq!(service.commands().len(), 1);
        assert_eq!(service.frame_of("playground-2d-sprite"), Some(1));
        assert_eq!(
            scene.entity_names(),
            vec!["playground-2d-sprite".to_owned()]
        );
    }

    #[test]
    fn infers_sprite_sheet_from_prepared_asset_metadata() {
        let loaded = LoadedAsset {
            key: AssetKey::new("playground-sidescroller/textures/player"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/textures/player.yml"),
            byte_len: 128,
        };
        let prepared = prepare_asset_from_contents(
            &loaded,
            r#"
kind: sprite-sheet-2d
frame_size:
  x: 32
  y: 32
columns: 8
rows: 4
animations:
  idle:
    frames: [0, 1, 2, 3]
    fps: 6
    looping: true
"#,
        )
        .expect("prepared asset should parse");

        let sheet = infer_sprite_sheet_from_prepared_asset(&prepared).expect("sheet should exist");
        assert_eq!(sheet.columns, 8);
        assert_eq!(sheet.rows, 4);
        assert_eq!(sheet.frame_size, Vec2::new(32.0, 32.0));
        assert_eq!(sheet.fps, 6.0);
        assert!(sheet.looping);
    }

    #[test]
    fn resolves_sprite_sheet_for_command_with_scene_override() {
        let asset_catalog = AssetCatalog::default();
        let key = AssetKey::new("playground-sidescroller/textures/player");
        asset_catalog.register_manifest(AssetManifest {
            key: key.clone(),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            tags: vec!["sprite".to_owned()],
        });
        asset_catalog.request_load(AssetLoadRequest::new(
            key.clone(),
            AssetLoadPriority::Immediate,
        ));
        let loaded = LoadedAsset {
            key: key.clone(),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            resolved_path: PathBuf::from("mods/playground-sidescroller/textures/player.yml"),
            byte_len: 128,
        };
        let prepared = prepare_asset_from_contents(
            &loaded,
            r#"
kind: sprite-sheet-2d
frame_size:
  x: 32
  y: 32
columns: 8
rows: 4
fps: 10
looping: true
"#,
        )
        .expect("prepared asset should parse");
        asset_catalog.mark_prepared(prepared);

        let mut command = Sprite2dSceneCommand::new(
            "playground-sidescroller",
            "player",
            key,
            Vec2::new(32.0, 32.0),
        );
        command.animation = Some(SpriteAnimation2dSceneOverride {
            fps: Some(5.0),
            looping: Some(false),
            start_frame: Some(2),
        });

        let sheet = resolve_sprite_sheet_for_command(&asset_catalog, &command)
            .expect("sheet should resolve");
        assert_eq!(sheet.fps, 5.0);
        assert!(!sheet.looping);
        assert_eq!(sheet.columns, 8);
    }
}
