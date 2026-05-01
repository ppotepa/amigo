use std::collections::BTreeMap;

use crate::model::{TileCollisionKind2d, TileRuleSet2d, TileTerrainRule2d, TileVariantSet2d};
use amigo_assets::{PreparedAsset, PreparedAssetKind};

pub fn infer_tile_ruleset_from_prepared_asset(prepared: &PreparedAsset) -> Option<TileRuleSet2d> {
    if !matches!(prepared.kind, PreparedAssetKind::TileRuleSet2d) {
        return None;
    }

    let mut terrains = BTreeMap::<String, TileTerrainRule2d>::new();

    for key in prepared.metadata.keys() {
        let Some(terrain_name) = key.strip_prefix("terrains.") else {
            continue;
        };
        let Some((terrain_name, field_path)) = terrain_name.split_once('.') else {
            continue;
        };

        let terrain =
            terrains
                .entry(terrain_name.to_owned())
                .or_insert_with(|| TileTerrainRule2d {
                    name: terrain_name.to_owned(),
                    symbol: '\0',
                    collision: TileCollisionKind2d::None,
                    variants: TileVariantSet2d::default(),
                });

        match field_path {
            "symbol" => {
                if let Some(symbol) = prepared
                    .metadata
                    .get(key)
                    .and_then(|value| value.chars().next())
                {
                    terrain.symbol = symbol;
                }
            }
            "collision" => {
                terrain.collision = match prepared.metadata.get(key).map(String::as_str) {
                    Some("solid") => TileCollisionKind2d::Solid,
                    Some("trigger") => TileCollisionKind2d::Trigger,
                    _ => TileCollisionKind2d::None,
                };
            }
            "variants.single" => terrain.variants.single = metadata_u32(prepared, key),
            "variants.left_cap" => terrain.variants.left_cap = metadata_u32(prepared, key),
            "variants.middle" => terrain.variants.middle = metadata_u32(prepared, key),
            "variants.right_cap" => terrain.variants.right_cap = metadata_u32(prepared, key),
            "variants.side_left" => terrain.variants.side_left = metadata_u32(prepared, key),
            "variants.side_right" => terrain.variants.side_right = metadata_u32(prepared, key),
            "variants.center" => terrain.variants.center = metadata_u32(prepared, key),
            "variants.top_cap" => terrain.variants.top_cap = metadata_u32(prepared, key),
            "variants.bottom_cap" => terrain.variants.bottom_cap = metadata_u32(prepared, key),
            "variants.vertical_middle" => {
                terrain.variants.vertical_middle = metadata_u32(prepared, key)
            }
            "variants.inner_corner_top_left" => {
                terrain.variants.inner_corner_top_left = metadata_u32(prepared, key)
            }
            "variants.inner_corner_top_right" => {
                terrain.variants.inner_corner_top_right = metadata_u32(prepared, key)
            }
            "variants.inner_corner_bottom_left" => {
                terrain.variants.inner_corner_bottom_left = metadata_u32(prepared, key)
            }
            "variants.inner_corner_bottom_right" => {
                terrain.variants.inner_corner_bottom_right = metadata_u32(prepared, key)
            }
            "variants.outer_corner_top_left" => {
                terrain.variants.outer_corner_top_left = metadata_u32(prepared, key)
            }
            "variants.outer_corner_top_right" => {
                terrain.variants.outer_corner_top_right = metadata_u32(prepared, key)
            }
            "variants.outer_corner_bottom_left" => {
                terrain.variants.outer_corner_bottom_left = metadata_u32(prepared, key)
            }
            "variants.outer_corner_bottom_right" => {
                terrain.variants.outer_corner_bottom_right = metadata_u32(prepared, key)
            }
            _ => {}
        }
    }

    let terrains = terrains
        .into_values()
        .filter(|terrain| terrain.symbol != '\0')
        .collect::<Vec<_>>();
    if terrains.is_empty() {
        return None;
    }

    Some(TileRuleSet2d { terrains })
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Option<u32> {
    prepared
        .metadata
        .get(key)
        .and_then(|value| value.parse::<u32>().ok())
}
