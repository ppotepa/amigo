use amigo_2d_physics::{
    CollisionLayer, Physics2dSceneService, StaticCollider2d, StaticCollider2dCommand,
};
use amigo_math::Vec2;
use amigo_scene::{
    SceneEntityId, SceneService, TileMap2dSceneCommand as SceneTileMap2dSceneCommand,
};

use crate::{
    TileMap2dDrawCommand, TileMap2dSceneService, infer_tile_ruleset_from_prepared_asset,
    model::TileMap2d, resolver, resolver::resolve_tilemap,
};

pub fn queue_tilemap_scene_command(
    scene_service: &SceneService,
    tilemap_scene_service: &TileMap2dSceneService,
    physics_scene_service: &Physics2dSceneService,
    asset_catalog: &amigo_assets::AssetCatalog,
    command: &SceneTileMap2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    let tilemap = build_tilemap_from_scene_command(asset_catalog, command);

    tilemap_scene_service.queue(TileMap2dDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        tilemap: tilemap.clone(),
        z_index: command.z_index,
    });

    for cell in resolver::solid_cells(&tilemap) {
        physics_scene_service.queue_static_collider(StaticCollider2dCommand {
            entity_id: entity,
            entity_name: format!(
                "{}.solid.{}.{}",
                command.entity_name, cell.row_from_bottom, cell.column
            ),
            collider: StaticCollider2d {
                size: command.tile_size,
                offset: Vec2::new(
                    cell.origin.x + command.tile_size.x * 0.5,
                    cell.origin.y + command.tile_size.y * 0.5,
                ),
                layer: CollisionLayer::new("world"),
            },
        });
    }

    entity
}

pub fn build_tilemap_from_scene_command(
    asset_catalog: &amigo_assets::AssetCatalog,
    command: &SceneTileMap2dSceneCommand,
) -> TileMap2d {
    let mut grid = command.grid.clone();
    let depth_fill_rows = command.depth_fill_rows;
    if depth_fill_rows > 0 && !grid.is_empty() {
        let fill_row = grid
            .last()
            .cloned()
            .unwrap_or_else(|| ".".repeat(grid[0].chars().count().max(1)));
        for _ in 0..depth_fill_rows {
            grid.push(fill_row.clone());
        }
    }
    let origin_offset = Vec2::new(0.0, -(depth_fill_rows as f32) * command.tile_size.y);
    let mut tilemap = TileMap2d {
        tileset: command.tileset.clone(),
        ruleset: command.ruleset.clone(),
        tile_size: command.tile_size,
        grid,
        origin_offset,
        resolved: None,
    };
    if let Some(ruleset_key) = command.ruleset.as_ref() {
        if let Some(prepared) = asset_catalog.prepared_asset(ruleset_key) {
            if let Some(ruleset) = infer_tile_ruleset_from_prepared_asset(&prepared) {
                tilemap.resolved = Some(resolve_tilemap(&tilemap, &ruleset));
            }
        }
    }
    tilemap
}
