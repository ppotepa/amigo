use amigo_assets::{AssetKey, AssetSourceKind, LoadedAsset, prepare_asset_from_contents};
use amigo_scene::SceneEntityId;
use amigo_math::Vec2;

use crate::{
    TileMap2d, TileMap2dDrawCommand, TileMap2dSceneService, infer_tile_ruleset_from_prepared_asset,
};

#[test]
fn syncs_ruleset_resolution_for_matching_tilemap() {
    let service = TileMap2dSceneService::default();
    let ruleset_asset = AssetKey::new("playground-sidescroller/tilesets/platformer-rules");

    service.queue(TileMap2dDrawCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "playground-sidescroller-tilemap".to_owned(),
        tilemap: TileMap2d {
            tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
            ruleset: Some(ruleset_asset.clone()),
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec![".###.".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        },
        z_index: 0.0,
    });

    assert_eq!(
        service.sync_ruleset_for_asset(&ruleset_asset, &super::common::horizontal_ruleset()),
        1
    );

    let resolved = service.commands()[0]
        .tilemap
        .resolved
        .clone()
        .expect("tilemap should be resolved");
    assert_eq!(
        resolved.rows[0][1].variant,
        Some(crate::TileVariantKind2d::LeftCap)
    );
    assert_eq!(
        resolved.rows[0][2].variant,
        Some(crate::TileVariantKind2d::Middle)
    );
    assert_eq!(
        resolved.rows[0][3].variant,
        Some(crate::TileVariantKind2d::RightCap)
    );
}

#[test]
fn infers_tile_ruleset_from_prepared_asset_metadata() {
    let loaded = LoadedAsset {
        key: AssetKey::new("playground-sidescroller/tilesets/platformer-rules"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        resolved_path: "mods/playground-sidescroller/tilesets/platformer-rules.yml".into(),
        byte_len: 128,
    };
    let prepared = prepare_asset_from_contents(
        &loaded,
        r##"
kind: tile-ruleset-2d
terrains:
  ground:
    symbol: "#"
    collision: solid
    variants:
      left_cap: 2
      middle: 3
      right_cap: 4
"##,
    )
    .expect("prepared asset should parse");

    let ruleset =
        infer_tile_ruleset_from_prepared_asset(&prepared).expect("ruleset should be inferred");
    assert_eq!(ruleset.terrains.len(), 1);
    assert_eq!(ruleset.terrains[0].symbol, '#');
    assert_eq!(ruleset.terrains[0].variants.middle, Some(3));
}
