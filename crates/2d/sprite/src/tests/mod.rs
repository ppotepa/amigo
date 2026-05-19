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
use amigo_render_api::RenderContributionSet;
use amigo_scene::{
    CameraOpticalResponse2dSceneCommand,
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    Material2dLightingSceneCommand, SceneCommand,
    SceneEntityId, SceneEvent, SceneEventQueue, SceneService, Sprite2dSceneCommand,
    SpriteAnimation2dSceneOverride,
};
use std::path::PathBuf;

#[test]
fn stores_sprite_draw_commands() {
    let service = SpriteSceneService::default();

    service.queue(SpriteDrawCommand {
        entity_id: SceneEntityId::new(7),
        entity_name: "playground-2d-sprite".to_owned(),
        render_layer: "world".to_owned(),
        sprite: Sprite {
            texture: AssetKey::new("playground-2d/spritesheets/sprite-lab"),
            size: Vec2::new(128.0, 128.0),
            sheet: None,
            sheet_is_explicit: false,
            animation_override: None,
            visual_maps: None,
            frame_index: 0,
            frame_elapsed: 0.0,
        },
        z_index: 0.0,
        transform: Transform2::default(),
        material: None,
        render_contributions: RenderContributionSet::default(),
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
        render_layer: "world".to_owned(),
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
            visual_maps: None,
            frame_index: 0,
            frame_elapsed: 0.0,
        },
        z_index: 0.0,
        transform: Transform2::default(),
        material: None,
        render_contributions: RenderContributionSet::default(),
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
        render_layer: "world".to_owned(),
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
            visual_maps: None,
            frame_index: 0,
            frame_elapsed: 0.0,
        },
        z_index: 0.0,
        transform: Transform2::default(),
        material: None,
        render_contributions: RenderContributionSet::default(),
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
fn queues_sprite_scene_command_with_material_and_render_contributions() {
    let scene = SceneService::default();
    let service = SpriteSceneService::default();

    let mut command = Sprite2dSceneCommand::new(
        "test-mod",
        "poster",
        AssetKey::new("test/poster"),
        Vec2::new(128.0, 128.0),
    );
    command.render_contributions.roles.insert("material.mask".to_owned(), true);
    command.render_contributions.roles.insert("optics.refract".to_owned(), true);
    command.material = Some(Material2dSceneCommand {
        optical: Material2dOpticalSceneCommand {
            mode: Material2dOpticalModeSceneCommand::Refractive,
            transmission: 0.45,
            refraction_px: 7.0,
            distortion: 0.2,
            dispersion: 0.0,
            roughness: 0.0,
            edge_boost: 0.0,
        },
        lighting: Material2dLightingSceneCommand {
            receives_light: false,
            response: 0.0,
        },
        camera_response: CameraOpticalResponse2dSceneCommand {
            enabled: false,
            intensity: 0.0,
            bloom: 0.0,
            glare: 0.0,
            ghosting: 0.0,
            streaks: 0.0,
            chromatic_smear: 0.0,
            dirt_response: 0.0,
            halation: 0.0,
            threshold: 0.0,
        },
    });

    queue_sprite_scene_command(&scene, &service, &command, None);

    let draw_command = service
        .commands()
        .into_iter()
        .next()
        .expect("sprite draw command should be queued");
    let material = draw_command
        .material
        .expect("sprite material should be carried to runtime");

    assert!(material.is_refractive());
    assert_eq!(
        draw_command
            .render_contributions
            .enabled_or("material.mask", false),
        true
    );
    assert_eq!(
        draw_command
            .render_contributions
            .enabled_or("optics.refract", false),
        true
    );
    assert_eq!(
        draw_command
            .render_contributions
            .enabled_or("world.color", false),
        true
    );
}

#[test]
fn infers_sprite_sheet_from_prepared_asset_metadata() {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-sidescroller/spritesheets/player"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: PathBuf::from(
            "mods/playground-sidescroller/spritesheets/player/spritesheet.yml",
        ),
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
        resolved_path: PathBuf::from(
            "mods/playground-sidescroller/spritesheets/player/spritesheet.yml",
        ),
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

#[test]
fn can_handle_sprite_scene_command_returns_true_for_sprite_command() {
    let command = SceneCommand::QueueSprite2d {
        command: Sprite2dSceneCommand::new(
            "playground-2d",
            "hero",
            AssetKey::new("playground-2d/sprites/hero"),
            Vec2::new(32.0, 32.0),
        ),
    };

    assert!(super::can_handle_sprite_scene_command(&command));
}

#[test]
fn handle_sprite_scene_command_queues_sprite_and_publishes_event() {
    let scene_service = SceneService::default();
    let sprite_scene_service = SpriteSceneService::default();
    let scene_event_queue = SceneEventQueue::default();
    let asset_catalog = AssetCatalog::default();
    let command = SceneCommand::QueueSprite2d {
        command: Sprite2dSceneCommand::new(
            "playground-2d",
            "hero",
            AssetKey::new("playground-2d/sprites/hero"),
            Vec2::new(32.0, 32.0),
        ),
    };

    let outcome = super::handle_sprite_scene_command(
        super::SpriteSceneCommandContext {
            scene_service: &scene_service,
            sprite_scene_service: &sprite_scene_service,
            scene_event_queue: &scene_event_queue,
            asset_catalog: &asset_catalog,
        },
        command,
    )
    .expect("sprite command should be handled");

    assert_eq!(outcome.entity_name, "hero");
    assert_eq!(outcome.source_mod, "playground-2d");
    assert_eq!(outcome.texture.as_str(), "playground-2d/sprites/hero");
    assert_eq!(scene_service.entity_names(), vec!["hero".to_owned()]);
    assert_eq!(sprite_scene_service.commands().len(), 1);

    let events = scene_event_queue.drain();
    assert_eq!(events.len(), 1);
    match &events[0] {
        SceneEvent::SpriteQueued {
            entity_id,
            entity_name,
            texture,
        } => {
            assert_eq!(*entity_id, 0);
            assert_eq!(entity_name, "hero");
            assert_eq!(texture.as_str(), "playground-2d/sprites/hero");
        }
        other => panic!("expected sprite queued event, got {other:?}"),
    }
}
