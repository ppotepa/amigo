use amigo_math::Vec2;
use amigo_assets::AssetKey;

use crate::{
    ResolvedTileMap2d, TileCollisionKind2d, TileMap2d, TileRuleSet2d,
    TileTerrainRule2d, TileVariantSet2d, resolve_tilemap,
};

pub fn horizontal_ruleset() -> TileRuleSet2d {
    TileRuleSet2d {
        terrains: vec![TileTerrainRule2d {
            name: "ground".to_owned(),
            symbol: '#',
            collision: TileCollisionKind2d::Solid,
            unknown_collision: None,
            paint: None,
            variants: TileVariantSet2d {
                single: Some(1),
                left_cap: Some(2),
                middle: Some(3),
                right_cap: Some(4),
                side_left: Some(5),
                side_right: Some(6),
                center: Some(7),
                top_cap: Some(8),
                bottom_cap: Some(9),
                vertical_middle: Some(10),
                inner_corner_top_left: Some(11),
                inner_corner_top_right: Some(12),
                inner_corner_bottom_left: Some(13),
                inner_corner_bottom_right: Some(14),
                outer_corner_top_left: Some(15),
                outer_corner_top_right: Some(16),
                outer_corner_bottom_left: Some(17),
                outer_corner_bottom_right: Some(18),
                ..TileVariantSet2d::default()
            },
        }],
        ..TileRuleSet2d::default()
    }
}

pub fn resolve_rows(grid: &[&str]) -> ResolvedTileMap2d {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/spritesheets/platformer/tilesets/platform/base"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: grid.iter().map(|row| (*row).to_owned()).collect(),
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };

    resolve_tilemap(&tilemap, &horizontal_ruleset())
}

