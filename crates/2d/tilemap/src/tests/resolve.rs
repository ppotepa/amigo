use amigo_math::Vec2;
use amigo_assets::AssetKey;

use crate::{TileCollisionKind2d, TileMap2d, TileRuleSet2d, TileVariantKind2d};
use super::common::{horizontal_ruleset, resolve_rows};

#[test]
fn resolves_single_horizontal_tile() {
    let resolved = resolve_rows(&[".#."]);
    let tile = &resolved.rows[0][1];

    assert_eq!(tile.variant, Some(TileVariantKind2d::Single));
    assert_eq!(tile.tile_id, Some(1));
    assert_eq!(tile.collision, TileCollisionKind2d::Solid);
    assert!(!tile.neighbors.left);
    assert!(!tile.neighbors.right);
}

#[test]
fn resolves_double_horizontal_tiles_into_caps() {
    let resolved = resolve_rows(&[".##."]);
    let left_tile = &resolved.rows[0][1];
    let right_tile = &resolved.rows[0][2];

    assert_eq!(left_tile.variant, Some(TileVariantKind2d::LeftCap));
    assert_eq!(left_tile.tile_id, Some(2));
    assert_eq!(right_tile.variant, Some(TileVariantKind2d::RightCap));
    assert_eq!(right_tile.tile_id, Some(4));
}

#[test]
fn resolves_triple_horizontal_tiles_with_middle() {
    let resolved = resolve_rows(&[".###."]);
    let left_tile = &resolved.rows[0][1];
    let middle_tile = &resolved.rows[0][2];
    let right_tile = &resolved.rows[0][3];

    assert_eq!(left_tile.variant, Some(TileVariantKind2d::LeftCap));
    assert_eq!(middle_tile.variant, Some(TileVariantKind2d::Middle));
    assert_eq!(middle_tile.tile_id, Some(3));
    assert_eq!(right_tile.variant, Some(TileVariantKind2d::RightCap));
}

#[test]
fn resolves_double_vertical_tiles_into_caps() {
    let resolved = resolve_rows(&[".#.", ".#."]);
    let top_tile = &resolved.rows[0][1];
    let bottom_tile = &resolved.rows[1][1];

    assert_eq!(top_tile.variant, Some(TileVariantKind2d::TopCap));
    assert_eq!(top_tile.tile_id, Some(8));
    assert_eq!(bottom_tile.variant, Some(TileVariantKind2d::BottomCap));
    assert_eq!(bottom_tile.tile_id, Some(9));
}

#[test]
fn resolves_triple_vertical_tiles_with_middle() {
    let resolved = resolve_rows(&[".#.", ".#.", ".#."]);
    let top_tile = &resolved.rows[0][1];
    let middle_tile = &resolved.rows[1][1];
    let bottom_tile = &resolved.rows[2][1];

    assert_eq!(top_tile.variant, Some(TileVariantKind2d::TopCap));
    assert_eq!(middle_tile.variant, Some(TileVariantKind2d::VerticalMiddle));
    assert_eq!(middle_tile.tile_id, Some(10));
    assert_eq!(bottom_tile.variant, Some(TileVariantKind2d::BottomCap));
}

#[test]
fn resolves_outer_corners_from_orthogonal_neighbors() {
    let resolved = resolve_rows(&[".##.", ".##."]);

    assert_eq!(
        resolved.rows[0][1].variant,
        Some(TileVariantKind2d::OuterCornerTopLeft)
    );
    assert_eq!(resolved.rows[0][1].tile_id, Some(15));
    assert_eq!(
        resolved.rows[0][2].variant,
        Some(TileVariantKind2d::OuterCornerTopRight)
    );
    assert_eq!(resolved.rows[0][2].tile_id, Some(16));
    assert_eq!(
        resolved.rows[1][1].variant,
        Some(TileVariantKind2d::OuterCornerBottomLeft)
    );
    assert_eq!(resolved.rows[1][1].tile_id, Some(17));
    assert_eq!(
        resolved.rows[1][2].variant,
        Some(TileVariantKind2d::OuterCornerBottomRight)
    );
    assert_eq!(resolved.rows[1][2].tile_id, Some(18));
}

#[test]
fn resolves_inner_corner_from_missing_diagonal() {
    let resolved = resolve_rows(&[".##.", "####", "####", "####"]);
    let tile = &resolved.rows[1][1];

    assert_eq!(tile.variant, Some(TileVariantKind2d::InnerCornerTopLeft));
    assert_eq!(tile.tile_id, Some(11));
    assert!(tile.neighbors.top);
    assert!(tile.neighbors.right);
    assert!(tile.neighbors.bottom);
    assert!(tile.neighbors.left);
    assert!(!tile.neighbors.top_left);
}

#[test]
fn resolves_top_edge_from_mixed_neighbors() {
    let resolved = resolve_rows(&[".###.", ".###."]);
    let tile = &resolved.rows[0][2];

    assert_eq!(tile.variant, Some(TileVariantKind2d::TopCap));
    assert_eq!(tile.tile_id, Some(8));
}

#[test]
fn resolves_center_from_fully_surrounded_tile() {
    let resolved = resolve_rows(&["#####", "#####", "#####"]);
    let tile = &resolved.rows[1][2];

    assert_eq!(tile.variant, Some(TileVariantKind2d::Center));
    assert_eq!(tile.tile_id, Some(7));
}

#[test]
fn resolves_side_edges_from_mixed_neighbors() {
    let resolved = resolve_rows(&[".##.", ".##.", ".##."]);

    assert_eq!(
        resolved.rows[1][1].variant,
        Some(TileVariantKind2d::SideLeft)
    );
    assert_eq!(resolved.rows[1][1].tile_id, Some(5));
    assert_eq!(
        resolved.rows[1][2].variant,
        Some(TileVariantKind2d::SideRight)
    );
    assert_eq!(resolved.rows[1][2].tile_id, Some(6));
}

#[test]
fn keeps_logical_collision_separate_from_visual_variant_resolution() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/spritesheets/platformer/tilesets/platform/base"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec![".###.".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };
    let resolved = crate::resolve_tilemap(&tilemap, &horizontal_ruleset());
    let solid_variants = resolved
        .rows[0]
        .iter()
        .filter(|tile| tile.collision == TileCollisionKind2d::Solid)
        .count();

    assert_eq!(crate::solid_cells(&tilemap).len(), 3);
    assert_eq!(solid_variants, 3);
    assert_eq!(resolved.rows[0][2].variant, Some(TileVariantKind2d::Middle));
}

#[test]
fn falls_back_predictably_when_variant_is_missing() {
    let ruleset = TileRuleSet2d {
        terrains: vec![crate::TileTerrainRule2d {
            name: "ground".to_owned(),
            symbol: '#',
            collision: TileCollisionKind2d::Solid,
            unknown_collision: None,
            paint: None,
            variants: crate::TileVariantSet2d {
                middle: Some(7),
                ..crate::TileVariantSet2d::default()
            },
        }],
        ..TileRuleSet2d::default()
    };
    let tilemap = TileMap2d {
        tileset: AssetKey::new("playground-sidescroller/spritesheets/platformer/tilesets/platform/base"),
        ruleset: None,
        tile_size: Vec2::new(16.0, 16.0),
        grid: vec![".##.".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };

    let resolved = crate::resolve_tilemap(&tilemap, &ruleset);
    assert_eq!(resolved.rows[0][1].tile_id, Some(7));
    assert_eq!(resolved.rows[0][2].tile_id, Some(7));
    assert_eq!(
        resolved.rows[0][1].variant,
        Some(TileVariantKind2d::LeftCap)
    );
    assert_eq!(
        resolved.rows[0][2].variant,
        Some(TileVariantKind2d::RightCap)
    );
}

