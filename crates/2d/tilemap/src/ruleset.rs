use std::collections::BTreeMap;

use crate::model::{
    TileCollisionKind2d, TileMarkerRule2d, TilePaintRule2d, TileRuleSet2d,
    TileRuleSetSymbols2d, TileTerrainRule2d, TileVariantSet2d,
};
use amigo_assets::{PreparedAsset, PreparedAssetKind};

pub fn infer_tile_ruleset_from_prepared_asset(prepared: &PreparedAsset) -> Option<TileRuleSet2d> {
    if !matches!(prepared.kind, PreparedAssetKind::TileRuleSet2d) {
        return None;
    }

    let mut terrains = BTreeMap::<String, TileTerrainRule2d>::new();
    let mut markers = BTreeMap::<String, TileMarkerRule2d>::new();
    let mut symbols = TileRuleSetSymbols2d::default();
    let tile_size = match (
        metadata_u32(prepared, "tile_size.x"),
        metadata_u32(prepared, "tile_size.y"),
    ) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    };

    for key in prepared.metadata.keys() {
        if key == "symbols.empty" {
            symbols.empty = prepared.metadata.get(key).and_then(|value| value.chars().next());
            continue;
        }

        if let Some(marker_path) = key.strip_prefix("markers.") {
            let Some((marker_name, field_path)) = marker_path.split_once('.') else {
                continue;
            };
            let marker = markers
                .entry(marker_name.to_owned())
                .or_insert_with(|| TileMarkerRule2d {
                    name: marker_name.to_owned(),
                    symbol: '\0',
                    label: marker_name.to_owned(),
                    entity_template: None,
                    max_count: None,
                });

            match field_path {
                "symbol" => {
                    marker.symbol = prepared
                        .metadata
                        .get(key)
                        .and_then(|value| value.chars().next())
                        .unwrap_or('\0');
                }
                "label" => {
                    if let Some(value) = prepared.metadata.get(key) {
                        marker.label = value.clone();
                    }
                }
                "entity_template" | "template" => {
                    marker.entity_template = prepared.metadata.get(key).cloned();
                }
                "max_count" => marker.max_count = metadata_usize(prepared, key),
                _ => {}
            }
            continue;
        }

        let Some(terrain_path) = key.strip_prefix("terrains.") else {
            continue;
        };
        let Some((terrain_name, field_path)) = terrain_path.split_once('.') else {
            continue;
        };

        let terrain =
            terrains
                .entry(terrain_name.to_owned())
                .or_insert_with(|| TileTerrainRule2d {
                    name: terrain_name.to_owned(),
                    symbol: '\0',
                    collision: TileCollisionKind2d::None,
                    unknown_collision: None,
                    paint: None,
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
                    Some(value) => match TileCollisionKind2d::from_contract_str(value) {
                        Some(collision) => {
                            terrain.unknown_collision = None;
                            collision
                        }
                        None => {
                            terrain.unknown_collision = Some(value.to_owned());
                            TileCollisionKind2d::None
                        }
                    },
                    _ => TileCollisionKind2d::None,
                };
            }
            "paint.brush" | "paint.category" | "paint.label" => {
                let paint = terrain.paint.get_or_insert_with(|| TilePaintRule2d {
                    brush: "terrain".to_owned(),
                    category: "terrain".to_owned(),
                    label: terrain.name.clone(),
                });
                if let Some(value) = prepared.metadata.get(key) {
                    match field_path {
                        "paint.brush" => paint.brush = value.clone(),
                        "paint.category" => paint.category = value.clone(),
                        "paint.label" => paint.label = value.clone(),
                        _ => {}
                    }
                }
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
    let markers = markers
        .into_values()
        .filter(|marker| marker.symbol != '\0')
        .collect::<Vec<_>>();
    if terrains.is_empty() && markers.is_empty() {
        return None;
    }

    Some(TileRuleSet2d {
        tile_size,
        symbols,
        terrains,
        markers,
    })
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Option<u32> {
    prepared
        .metadata
        .get(key)
        .and_then(|value| value.parse::<u32>().ok())
}

fn metadata_usize(prepared: &PreparedAsset, key: &str) -> Option<usize> {
    prepared
        .metadata
        .get(key)
        .and_then(|value| value.parse::<usize>().ok())
}
