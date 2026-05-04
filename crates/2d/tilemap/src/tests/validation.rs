use amigo_assets::AssetKey;
use amigo_math::Vec2;

use crate::{
    validate_tile_world_contract_with_context, TileCollisionKind2d, TileMap2d,
    TileMarkerRule2d, TileRuleSet2d, TileRuleSetSymbols2d, TileTerrainRule2d,
    TileVariantSet2d, TileWorldDiagnosticSeverity, TileWorldValidationContext,
};

#[test]
fn validates_grid_width_unknown_symbols_marker_counts_and_tile_ids() {
    let tilemap = TileMap2d {
        tileset: AssetKey::new("ink-wars/tilesets/missing"),
        ruleset: Some(AssetKey::new("ink-wars/tilesets/dirt-rules")),
        tile_size: Vec2::new(32.0, 64.0),
        grid: vec!["P#X".to_owned(), "P#".to_owned()],
        origin_offset: Vec2::new(0.0, 0.0),
        resolved: None,
    };
    let ruleset = TileRuleSet2d {
        tile_size: Some((64, 64)),
        symbols: TileRuleSetSymbols2d { empty: Some('.') },
        terrains: vec![TileTerrainRule2d {
            name: "ground".to_owned(),
            symbol: '#',
            collision: TileCollisionKind2d::Full,
            unknown_collision: Some("mystery".to_owned()),
            paint: None,
            variants: TileVariantSet2d {
                single: Some(9),
                ..TileVariantSet2d::default()
            },
        }],
        markers: vec![TileMarkerRule2d {
            name: "player".to_owned(),
            symbol: 'P',
            label: "Player".to_owned(),
            entity_template: Some("player".to_owned()),
            max_count: Some(1),
        }],
    };
    let diagnostics = validate_tile_world_contract_with_context(
        &tilemap,
        Some(&ruleset),
        &TileWorldValidationContext {
            tileset_exists: Some(false),
            ruleset_exists: Some(true),
            tile_count: Some(8),
            ruleset_tile_size: None,
        },
    );

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.severity == TileWorldDiagnosticSeverity::Error && diagnostic.message.contains("width")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("symbol 'X'")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("max_count")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("tile id 9")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("unknown collision")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("Tileset")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("tile_size")));
}
