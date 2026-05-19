use amigo_assets::AssetCatalog;
use amigo_assets::AssetKey;
use amigo_math::Vec2;
use amigo_scene::{
    SceneCommand, SceneEntityId, SceneEvent, SceneEventQueue, SceneService,
    TileMap2dSceneCommand as SceneTileMap2dSceneCommand, TileMapMarker2dSceneCommand,
};

use crate::{
    TileMap2d, TileMap2dDrawCommand, TileMap2dSceneService, build_tilemap_from_scene_command,
    queue_tilemap_scene_command,
};

#[test]
fn stores_tilemap_draw_commands() {
    let service = TileMap2dSceneService::default();

    service.queue(TileMap2dDrawCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "playground-sidescroller-tilemap".to_owned(),
        render_layer: "world".to_owned(),
        tilemap: TileMap2d {
            tileset: AssetKey::new(
                "playground-sidescroller/spritesheets/platformer/tilesets/platform/base",
            ),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec!["....".to_owned(), ".P..".to_owned(), "####".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        },
        z_index: 0.0,
    });

    assert_eq!(service.commands().len(), 1);
    assert_eq!(
        service.entity_names(),
        vec!["playground-sidescroller-tilemap".to_owned()]
    );

    service.clear();
    assert!(service.commands().is_empty());
}

#[test]
fn builds_tilemap_from_scene_command_with_depth_fill() {
    let asset_catalog = AssetCatalog::default();
    let mut command = SceneTileMap2dSceneCommand::new(
        "playground-sidescroller",
        "tilemap",
        AssetKey::new("playground-sidescroller/spritesheets/platformer/tilesets/platform/base"),
        Vec2::new(16.0, 16.0),
        vec!["....".to_owned(), "####".to_owned()],
    );
    command.depth_fill_rows = 2;

    let tilemap = build_tilemap_from_scene_command(&asset_catalog, &command);
    assert_eq!(tilemap.grid.len(), 4);
    assert_eq!(tilemap.grid[2], "####");
    assert_eq!(tilemap.grid[3], "####");
    assert_eq!(tilemap.origin_offset, Vec2::new(0.0, -32.0));
}

#[test]
fn queues_tilemap_scene_command_and_static_colliders() {
    use amigo_2d_physics::Physics2dSceneService;

    let scene_service = SceneService::default();
    let tilemap_scene_service = TileMap2dSceneService::default();
    let physics_scene_service = Physics2dSceneService::default();
    let asset_catalog = AssetCatalog::default();
    let command = SceneTileMap2dSceneCommand::new(
        "playground-sidescroller",
        "playground-sidescroller-tilemap",
        AssetKey::new("playground-sidescroller/spritesheets/platformer/tilesets/platform/base"),
        Vec2::new(16.0, 16.0),
        vec!["....".to_owned(), ".##.".to_owned()],
    );

    let entity = queue_tilemap_scene_command(
        &scene_service,
        &tilemap_scene_service,
        &physics_scene_service,
        &asset_catalog,
        &command,
    );

    assert_eq!(
        scene_service.entity_names(),
        vec!["playground-sidescroller-tilemap".to_owned()]
    );
    assert_eq!(tilemap_scene_service.commands().len(), 1);
    assert_eq!(tilemap_scene_service.commands()[0].entity_id, entity);
    assert_eq!(physics_scene_service.static_colliders().len(), 2);
}

#[test]
fn can_handle_tilemap_scene_command_returns_true_for_tilemap_command() {
    let command = SceneCommand::QueueTileMap2d {
        command: SceneTileMap2dSceneCommand {
            source_mod: "playground-2d".to_owned(),
            entity_name: "arena".to_owned(),
            tileset: AssetKey::new("playground-2d/tilesets/basic"),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec!["..".to_owned(), "..".to_owned()],
            depth_fill_rows: 0,
            render_layer: "world".to_owned(),
            z_index: 0.0,
        },
    };

    assert!(crate::can_handle_tilemap_scene_command(&command));
}

#[test]
fn handle_tilemap_scene_command_queues_tilemap_and_publishes_event() {
    use amigo_2d_physics::Physics2dSceneService;

    let scene_service = SceneService::default();
    let tilemap_scene_service = TileMap2dSceneService::default();
    let physics_scene_service = Physics2dSceneService::default();
    let scene_event_queue = SceneEventQueue::default();
    let asset_catalog = AssetCatalog::default();
    let command = SceneCommand::QueueTileMap2d {
        command: SceneTileMap2dSceneCommand {
            source_mod: "playground-2d".to_owned(),
            entity_name: "arena".to_owned(),
            tileset: AssetKey::new("playground-2d/tilesets/basic"),
            ruleset: Some(AssetKey::new("playground-2d/rulesets/basic")),
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec!["..".to_owned(), "..".to_owned()],
            depth_fill_rows: 0,
            render_layer: "world".to_owned(),
            z_index: 0.0,
        },
    };

    let outcome = crate::handle_tilemap_scene_command(
        crate::TileMapSceneCommandContext {
            scene_service: &scene_service,
            tilemap_scene_service: &tilemap_scene_service,
            physics_scene_service: &physics_scene_service,
            asset_catalog: &asset_catalog,
            scene_event_queue: &scene_event_queue,
        },
        command,
    )
    .expect("tilemap command should be handled");

    assert_eq!(outcome.entity_name, "arena");
    assert_eq!(outcome.source_mod, "playground-2d");
    assert_eq!(outcome.tileset.as_str(), "playground-2d/tilesets/basic");
    assert_eq!(
        outcome.ruleset.as_ref().map(|key| key.as_str()),
        Some("playground-2d/rulesets/basic")
    );
    assert_eq!(scene_service.entity_names(), vec!["arena".to_owned()]);
    assert_eq!(tilemap_scene_service.commands().len(), 1);

    let events = scene_event_queue.drain();
    assert_eq!(events.len(), 1);
    match &events[0] {
        SceneEvent::TileMapQueued {
            entity_id,
            entity_name,
            tileset,
        } => {
            assert_eq!(*entity_id, 0);
            assert_eq!(entity_name, "arena");
            assert_eq!(tileset.as_str(), "playground-2d/tilesets/basic");
        }
        other => panic!("expected tilemap queued event, got {other:?}"),
    }
}

#[test]
fn handle_tilemap_marker_scene_command_anchors_entity_and_publishes_event() {
    let scene_service = SceneService::default();
    let tilemap_scene_service = TileMap2dSceneService::default();
    let scene_event_queue = SceneEventQueue::default();

    tilemap_scene_service.queue(TileMap2dDrawCommand {
        entity_id: SceneEntityId::new(42),
        entity_name: "arena".to_owned(),
        render_layer: "world".to_owned(),
        tilemap: TileMap2d {
            tileset: AssetKey::new("playground-2d/tilesets/basic"),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec![".P.".to_owned()],
            origin_offset: Vec2::ZERO,
            resolved: None,
        },
        z_index: 0.0,
    });

    let command = SceneCommand::QueueTileMapMarker2d {
        command: TileMapMarker2dSceneCommand::new(
            "playground-2d",
            "player",
            Some("arena".to_owned()),
            "P",
            0,
            Vec2::new(1.0, 2.0),
        ),
    };

    assert!(crate::can_handle_tilemap_marker_scene_command(&command));

    let outcome = crate::handle_tilemap_marker_scene_command(
        crate::TileMapMarkerSceneCommandContext {
            scene_service: &scene_service,
            tilemap_scene_service: &tilemap_scene_service,
            scene_event_queue: &scene_event_queue,
        },
        command,
    )
    .expect("tilemap marker command should be handled");

    assert_eq!(
        outcome,
        crate::TileMapMarkerSceneCommandOutcome::Anchored {
            entity_name: "player".to_owned(),
            source_mod: "playground-2d".to_owned(),
            symbol: "P".to_owned(),
            index: 0,
            tilemap_entity: "arena".to_owned(),
        }
    );

    let transform = scene_service
        .transform_of("player")
        .expect("player transform should be written from marker");
    assert_eq!(transform.translation.x, 25.0);
    assert_eq!(transform.translation.y, 10.0);

    let entity = scene_service
        .entity_by_name("player")
        .expect("player entity should be spawned");

    assert_eq!(
        scene_event_queue.pending(),
        [SceneEvent::TileMapMarkerQueued {
            entity_id: entity.id.raw(),
            entity_name: "player".to_owned(),
            symbol: "P".to_owned(),
        }]
    );
}

#[test]
fn handle_tilemap_marker_scene_command_reports_missing_tilemap() {
    let scene_service = SceneService::default();
    let tilemap_scene_service = TileMap2dSceneService::default();
    let scene_event_queue = SceneEventQueue::default();

    let outcome = crate::handle_tilemap_marker_scene_command(
        crate::TileMapMarkerSceneCommandContext {
            scene_service: &scene_service,
            tilemap_scene_service: &tilemap_scene_service,
            scene_event_queue: &scene_event_queue,
        },
        SceneCommand::QueueTileMapMarker2d {
            command: TileMapMarker2dSceneCommand::new(
                "playground-2d",
                "player",
                Some("arena".to_owned()),
                "P",
                0,
                Vec2::ZERO,
            ),
        },
    )
    .expect("missing tilemap should be reported as non-fatal outcome");

    assert_eq!(
        outcome,
        crate::TileMapMarkerSceneCommandOutcome::MissingTileMap {
            entity_name: "player".to_owned(),
            source_mod: "playground-2d".to_owned(),
            symbol: "P".to_owned(),
        }
    );
    assert!(scene_event_queue.pending().is_empty());
}
