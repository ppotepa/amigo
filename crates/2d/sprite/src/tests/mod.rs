use super::{
    Sprite, SpriteAnimationOverride, SpriteDrawCommand, SpriteSceneService, SpriteSheet,
    infer_sprite_sheet_from_prepared_asset, queue_sprite_scene_command,
    resolve_sprite_sheet_for_command,
};
use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
    LoadedAsset, prepare_asset_from_contents,
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
            texture: AssetKey::new("playground-2d/spritesheets/sprite-lab"),
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
            texture: AssetKey::new("playground-2d/spritesheets/hello-world-spritesheet"),
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
    let texture = AssetKey::new("playground-sidescroller/spritesheets/coin");
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
        AssetKey::new("playground-2d/spritesheets/sprite-lab"),
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
        key: AssetKey::new("playground-sidescroller/spritesheets/player"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from("mods/playground-sidescroller/spritesheets/player/spritesheet.yml"),
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
    let key = AssetKey::new("playground-sidescroller/spritesheets/player");
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
        resolved_path: PathBuf::from("mods/playground-sidescroller/spritesheets/player/spritesheet.yml"),
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

    let sheet =
        resolve_sprite_sheet_for_command(&asset_catalog, &command).expect("sheet should resolve");
    assert_eq!(sheet.fps, 5.0);
    assert!(!sheet.looping);
    assert_eq!(sheet.columns, 8);
}
