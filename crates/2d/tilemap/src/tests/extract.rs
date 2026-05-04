use amigo_math::Vec2;
use amigo_assets::AssetKey;

use crate::{
    ResolvedTile2d, ResolvedTileMap2d, TileMap2d, TileCollisionKind2d, TileNeighborInfo2d,
    marker_cells, solid_cells,
};

#[test]
fn extracts_solid_cells_from_grid_symbols() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec!["....".to_owned(), ".#..".to_owned(), "#==#".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };

    let solids = solid_cells(&tilemap);
    assert_eq!(solids.len(), 5);
    assert_eq!(solids[0].column, 1);
    assert_eq!(solids[0].row_from_bottom, 1);
    assert_eq!(solids[0].origin, Vec2::new(16.0, 16.0));
}

#[test]
fn extracts_marker_cells_from_grid_symbols() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec!["..F.".to_owned(), ".P..".to_owned(), "#C=#".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };

    let player_markers = marker_cells(&tilemap, 'P');
    let coin_markers = marker_cells(&tilemap, 'C');
    let finish_markers = marker_cells(&tilemap, 'F');

    assert_eq!(player_markers.len(), 1);
    assert_eq!(player_markers[0].origin, Vec2::new(16.0, 16.0));
    assert_eq!(coin_markers.len(), 1);
    assert_eq!(coin_markers[0].origin, Vec2::new(16.0, 0.0));
    assert_eq!(finish_markers.len(), 1);
    assert_eq!(finish_markers[0].origin, Vec2::new(32.0, 32.0));
}

#[test]
fn counts_solid_cells_with_collision_only_when_symbol_is_solid() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec![".###.".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };

    let resolved = crate::resolve_tilemap(&tilemap, &super::common::horizontal_ruleset());
    let solid_variants = resolved
        .rows[0]
        .iter()
        .filter(|tile| tile.collision == TileCollisionKind2d::Solid)
        .count();

    assert_eq!(solid_cells(&tilemap).len(), 3);
    assert_eq!(solid_variants, 3);
}

#[test]
fn extracts_solid_cells_from_resolved_collision_before_legacy_symbols() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec!["....".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: Some(ResolvedTileMap2d {
            rows: vec![vec![
                resolved('.', TileCollisionKind2d::None),
                resolved('G', TileCollisionKind2d::Full),
                resolved('=', TileCollisionKind2d::OneWay),
            ]],
        }),
    };

    let solids = solid_cells(&tilemap);
    assert_eq!(solids.len(), 1);
    assert_eq!(solids[0].symbol, 'G');
    assert_eq!(solids[0].column, 1);
}

#[test]
fn keeps_legacy_solid_symbol_fallback_without_resolved_data() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec![".#=.".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };

    assert_eq!(solid_cells(&tilemap).len(), 2);
}

fn resolved(symbol: char, collision: TileCollisionKind2d) -> ResolvedTile2d {
    ResolvedTile2d {
        symbol,
        terrain_name: None,
        tile_id: None,
        collision,
        variant: None,
        neighbors: TileNeighborInfo2d::default(),
    }
}
