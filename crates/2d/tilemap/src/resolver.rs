use crate::model::{
    ResolvedTile2d, ResolvedTileMap2d, TileCollisionKind2d, TileMap2d, TileMapMarkerCell,
    TileMapSolidCell, TileNeighborInfo2d, TileRuleSet2d, TileVariantKind2d,
};
use amigo_math::Vec2;

pub fn solid_cells(tilemap: &TileMap2d) -> Vec<TileMapSolidCell> {
    let row_count = tilemap.grid.len();
    let mut solids = Vec::new();

    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let row_from_bottom = row_count.saturating_sub(row_index + 1);
        for (column, symbol) in row.chars().enumerate() {
            if symbol != '#' && symbol != '=' {
                continue;
            }
            solids.push(TileMapSolidCell {
                column,
                row_from_bottom,
                symbol,
                origin: Vec2::new(
                    column as f32 * tilemap.tile_size.x + tilemap.origin_offset.x,
                    row_from_bottom as f32 * tilemap.tile_size.y + tilemap.origin_offset.y,
                ),
            });
        }
    }

    solids
}

pub fn marker_cells(tilemap: &TileMap2d, symbol_filter: char) -> Vec<TileMapMarkerCell> {
    let row_count = tilemap.grid.len();
    let mut markers = Vec::new();

    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let row_from_bottom = row_count.saturating_sub(row_index + 1);
        for (column, symbol) in row.chars().enumerate() {
            if symbol != symbol_filter {
                continue;
            }
            markers.push(TileMapMarkerCell {
                column,
                row_from_bottom,
                symbol,
                origin: Vec2::new(
                    column as f32 * tilemap.tile_size.x + tilemap.origin_offset.x,
                    row_from_bottom as f32 * tilemap.tile_size.y + tilemap.origin_offset.y,
                ),
            });
        }
    }

    markers
}

pub fn resolve_tilemap(tilemap: &TileMap2d, ruleset: &TileRuleSet2d) -> ResolvedTileMap2d {
    let rows = tilemap
        .grid
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            row.chars()
                .enumerate()
                .map(|(column, symbol)| {
                    let neighbors = neighbor_info(tilemap, row_index, column, symbol);
                    let Some(terrain) = ruleset.terrain_for_symbol(symbol) else {
                        return ResolvedTile2d {
                            symbol,
                            terrain_name: None,
                            tile_id: None,
                            collision: TileCollisionKind2d::None,
                            variant: None,
                            neighbors,
                        };
                    };

                    let variant = resolve_variant(neighbors);
                    ResolvedTile2d {
                        symbol,
                        terrain_name: Some(terrain.name.clone()),
                        tile_id: terrain.variants.tile_id_for(variant),
                        collision: terrain.collision,
                        variant: Some(variant),
                        neighbors,
                    }
                })
                .collect()
        })
        .collect();

    ResolvedTileMap2d { rows }
}

fn neighbor_info(
    tilemap: &TileMap2d,
    row_index: usize,
    column: usize,
    symbol: char,
) -> TileNeighborInfo2d {
    TileNeighborInfo2d {
        left: symbol_at(tilemap, row_index, column.checked_sub(1)) == Some(symbol),
        right: symbol_at(tilemap, row_index, column.checked_add(1)) == Some(symbol),
        top: row_index
            .checked_sub(1)
            .and_then(|top_row| symbol_at(tilemap, top_row, Some(column)))
            == Some(symbol),
        bottom: symbol_at(
            tilemap,
            row_index.checked_add(1).unwrap_or(row_index),
            Some(column),
        ) == Some(symbol)
            && row_index + 1 < tilemap.grid.len(),
        top_left: row_index
            .checked_sub(1)
            .and_then(|top_row| symbol_at(tilemap, top_row, column.checked_sub(1)))
            == Some(symbol),
        top_right: row_index
            .checked_sub(1)
            .and_then(|top_row| symbol_at(tilemap, top_row, column.checked_add(1)))
            == Some(symbol),
        bottom_left: symbol_at(
            tilemap,
            row_index.checked_add(1).unwrap_or(row_index),
            column.checked_sub(1),
        ) == Some(symbol)
            && row_index + 1 < tilemap.grid.len(),
        bottom_right: symbol_at(
            tilemap,
            row_index.checked_add(1).unwrap_or(row_index),
            column.checked_add(1),
        ) == Some(symbol)
            && row_index + 1 < tilemap.grid.len(),
    }
}

fn symbol_at(tilemap: &TileMap2d, row_index: usize, column: Option<usize>) -> Option<char> {
    let column = column?;
    tilemap
        .grid
        .get(row_index)
        .and_then(|row| row.chars().nth(column))
}

fn resolve_variant(neighbors: TileNeighborInfo2d) -> TileVariantKind2d {
    if neighbors.top && neighbors.right && neighbors.bottom && neighbors.left {
        if !neighbors.top_left {
            return TileVariantKind2d::InnerCornerTopLeft;
        }
        if !neighbors.top_right {
            return TileVariantKind2d::InnerCornerTopRight;
        }
        if !neighbors.bottom_left {
            return TileVariantKind2d::InnerCornerBottomLeft;
        }
        if !neighbors.bottom_right {
            return TileVariantKind2d::InnerCornerBottomRight;
        }

        return TileVariantKind2d::Center;
    }

    match (
        neighbors.top,
        neighbors.right,
        neighbors.bottom,
        neighbors.left,
    ) {
        (false, true, true, false) => return TileVariantKind2d::OuterCornerTopLeft,
        (false, false, true, true) => return TileVariantKind2d::OuterCornerTopRight,
        (true, true, false, false) => return TileVariantKind2d::OuterCornerBottomLeft,
        (true, false, false, true) => return TileVariantKind2d::OuterCornerBottomRight,
        _ => {}
    }

    if !neighbors.top && neighbors.left && neighbors.right && neighbors.bottom {
        return TileVariantKind2d::TopCap;
    }

    if !neighbors.bottom && neighbors.left && neighbors.right && neighbors.top {
        return TileVariantKind2d::BottomCap;
    }

    if !neighbors.left && neighbors.top && neighbors.bottom && neighbors.right {
        return TileVariantKind2d::SideLeft;
    }

    if !neighbors.right && neighbors.top && neighbors.bottom && neighbors.left {
        return TileVariantKind2d::SideRight;
    }

    if neighbors.left || neighbors.right {
        return resolve_horizontal_variant(neighbors);
    }

    if neighbors.top || neighbors.bottom {
        return resolve_vertical_variant(neighbors);
    }

    TileVariantKind2d::Single
}

fn resolve_horizontal_variant(neighbors: TileNeighborInfo2d) -> TileVariantKind2d {
    match (neighbors.left, neighbors.right) {
        (false, false) => TileVariantKind2d::Single,
        (false, true) => TileVariantKind2d::LeftCap,
        (true, true) => TileVariantKind2d::Middle,
        (true, false) => TileVariantKind2d::RightCap,
    }
}

fn resolve_vertical_variant(neighbors: TileNeighborInfo2d) -> TileVariantKind2d {
    match (neighbors.top, neighbors.bottom) {
        (false, false) => TileVariantKind2d::Single,
        (false, true) => TileVariantKind2d::TopCap,
        (true, true) => TileVariantKind2d::VerticalMiddle,
        (true, false) => TileVariantKind2d::BottomCap,
    }
}
