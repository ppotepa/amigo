use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_math::Vec2;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileCollisionKind2d {
    None,
    Solid,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileVariantKind2d {
    Single,
    LeftCap,
    Middle,
    RightCap,
    SideLeft,
    SideRight,
    Center,
    TopCap,
    BottomCap,
    VerticalMiddle,
    InnerCornerTopLeft,
    InnerCornerTopRight,
    InnerCornerBottomLeft,
    InnerCornerBottomRight,
    OuterCornerTopLeft,
    OuterCornerTopRight,
    OuterCornerBottomLeft,
    OuterCornerBottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileNeighborInfo2d {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileVariantSet2d {
    pub single: Option<u32>,
    pub left_cap: Option<u32>,
    pub middle: Option<u32>,
    pub right_cap: Option<u32>,
    pub side_left: Option<u32>,
    pub side_right: Option<u32>,
    pub center: Option<u32>,
    pub top_cap: Option<u32>,
    pub bottom_cap: Option<u32>,
    pub vertical_middle: Option<u32>,
    pub inner_corner_top_left: Option<u32>,
    pub inner_corner_top_right: Option<u32>,
    pub inner_corner_bottom_left: Option<u32>,
    pub inner_corner_bottom_right: Option<u32>,
    pub outer_corner_top_left: Option<u32>,
    pub outer_corner_top_right: Option<u32>,
    pub outer_corner_bottom_left: Option<u32>,
    pub outer_corner_bottom_right: Option<u32>,
}

impl TileVariantSet2d {
    pub fn tile_id_for(&self, variant: TileVariantKind2d) -> Option<u32> {
        let exact = match variant {
            TileVariantKind2d::Single => self.single,
            TileVariantKind2d::LeftCap => self.left_cap,
            TileVariantKind2d::Middle => self.middle,
            TileVariantKind2d::RightCap => self.right_cap,
            TileVariantKind2d::SideLeft => self.side_left,
            TileVariantKind2d::SideRight => self.side_right,
            TileVariantKind2d::Center => self.center,
            TileVariantKind2d::TopCap => self.top_cap,
            TileVariantKind2d::BottomCap => self.bottom_cap,
            TileVariantKind2d::VerticalMiddle => self.vertical_middle,
            TileVariantKind2d::InnerCornerTopLeft => self.inner_corner_top_left,
            TileVariantKind2d::InnerCornerTopRight => self.inner_corner_top_right,
            TileVariantKind2d::InnerCornerBottomLeft => self.inner_corner_bottom_left,
            TileVariantKind2d::InnerCornerBottomRight => self.inner_corner_bottom_right,
            TileVariantKind2d::OuterCornerTopLeft => self.outer_corner_top_left,
            TileVariantKind2d::OuterCornerTopRight => self.outer_corner_top_right,
            TileVariantKind2d::OuterCornerBottomLeft => self.outer_corner_bottom_left,
            TileVariantKind2d::OuterCornerBottomRight => self.outer_corner_bottom_right,
        };

        exact
            .or(self.center)
            .or(self.middle)
            .or(self.side_left)
            .or(self.side_right)
            .or(self.single)
            .or(self.left_cap)
            .or(self.right_cap)
            .or(self.top_cap)
            .or(self.bottom_cap)
            .or(self.vertical_middle)
            .or(self.inner_corner_top_left)
            .or(self.inner_corner_top_right)
            .or(self.inner_corner_bottom_left)
            .or(self.inner_corner_bottom_right)
            .or(self.outer_corner_top_left)
            .or(self.outer_corner_top_right)
            .or(self.outer_corner_bottom_left)
            .or(self.outer_corner_bottom_right)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileTerrainRule2d {
    pub name: String,
    pub symbol: char,
    pub collision: TileCollisionKind2d,
    pub variants: TileVariantSet2d,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileRuleSet2d {
    pub terrains: Vec<TileTerrainRule2d>,
}

impl TileRuleSet2d {
    pub fn terrain_for_symbol(&self, symbol: char) -> Option<&TileTerrainRule2d> {
        self.terrains
            .iter()
            .find(|terrain| terrain.symbol == symbol)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileCell2d {
    pub symbol: char,
    pub tile_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileLayer2d {
    pub rows: Vec<Vec<TileCell2d>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionTileLayer {
    pub rows: Vec<Vec<bool>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileSet2d {
    pub asset: AssetKey,
    pub tile_size: Vec2,
    pub columns: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMap2d {
    pub tileset: AssetKey,
    pub ruleset: Option<AssetKey>,
    pub tile_size: Vec2,
    pub grid: Vec<String>,
    pub origin_offset: Vec2,
    pub resolved: Option<ResolvedTileMap2d>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMapSolidCell {
    pub column: usize,
    pub row_from_bottom: usize,
    pub symbol: char,
    pub origin: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMapMarkerCell {
    pub column: usize,
    pub row_from_bottom: usize,
    pub symbol: char,
    pub origin: Vec2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTile2d {
    pub symbol: char,
    pub terrain_name: Option<String>,
    pub tile_id: Option<u32>,
    pub collision: TileCollisionKind2d,
    pub variant: Option<TileVariantKind2d>,
    pub neighbors: TileNeighborInfo2d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTileMap2d {
    pub rows: Vec<Vec<ResolvedTile2d>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMap2dDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub tilemap: TileMap2d,
    pub z_index: f32,
}

#[derive(Debug, Default)]
pub struct TileMap2dSceneService {
    commands: Mutex<Vec<TileMap2dDrawCommand>>,
}

impl TileMap2dSceneService {
    pub fn queue(&self, command: TileMap2dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned");
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<TileMap2dDrawCommand> {
        self.commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned")
            .clone()
    }

    pub fn sync_ruleset_for_asset(
        &self,
        ruleset_asset: &AssetKey,
        ruleset: &TileRuleSet2d,
    ) -> usize {
        let mut commands = self
            .commands
            .lock()
            .expect("tilemap2d scene service mutex should not be poisoned");
        let mut updated = 0;

        for command in commands.iter_mut() {
            if command.tilemap.ruleset.as_ref() != Some(ruleset_asset) {
                continue;
            }
            command.tilemap.resolved = Some(resolve_tilemap(&command.tilemap, ruleset));
            updated += 1;
        }

        updated
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TileMap2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct TileMap2dPlugin;

impl RuntimePlugin for TileMap2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-tilemap"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(TileMap2dSceneService::default())?;
        registry.register(TileMap2dDomainInfo {
            crate_name: "amigo-2d-tilemap",
            capability: "tilemap_2d",
        })
    }
}

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

#[cfg(test)]
mod tests {
    use super::{
        ResolvedTileMap2d, TileCollisionKind2d, TileMap2d, TileMap2dDrawCommand,
        TileMap2dSceneService, TileRuleSet2d, TileTerrainRule2d, TileVariantKind2d,
        TileVariantSet2d, marker_cells, resolve_tilemap, solid_cells,
    };
    use amigo_assets::AssetKey;
    use amigo_math::Vec2;
    use amigo_scene::SceneEntityId;

    fn horizontal_ruleset() -> TileRuleSet2d {
        TileRuleSet2d {
            terrains: vec![TileTerrainRule2d {
                name: "ground".to_owned(),
                symbol: '#',
                collision: TileCollisionKind2d::Solid,
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
        }
    }

    fn resolve_rows(grid: &[&str]) -> ResolvedTileMap2d {
        let tilemap = TileMap2d {
            tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: grid.iter().map(|row| (*row).to_owned()).collect(),
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        };

        resolve_tilemap(&tilemap, &horizontal_ruleset())
    }

    #[test]
    fn stores_tilemap_draw_commands() {
        let service = TileMap2dSceneService::default();

        service.queue(TileMap2dDrawCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "playground-sidescroller-tilemap".to_owned(),
            tilemap: TileMap2d {
                tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
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
            tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec![".###.".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        };

        let resolved = resolve_tilemap(&tilemap, &horizontal_ruleset());
        let solid_variants = resolved.rows[0]
            .iter()
            .filter(|tile| tile.collision == TileCollisionKind2d::Solid)
            .count();

        assert_eq!(solid_cells(&tilemap).len(), 3);
        assert_eq!(solid_variants, 3);
        assert_eq!(resolved.rows[0][2].variant, Some(TileVariantKind2d::Middle));
    }

    #[test]
    fn falls_back_predictably_when_variant_is_missing() {
        let ruleset = TileRuleSet2d {
            terrains: vec![TileTerrainRule2d {
                name: "ground".to_owned(),
                symbol: '#',
                collision: TileCollisionKind2d::Solid,
                variants: TileVariantSet2d {
                    middle: Some(7),
                    ..TileVariantSet2d::default()
                },
            }],
        };
        let tilemap = TileMap2d {
            tileset: AssetKey::new("playground-sidescroller/tilesets/platformer"),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec![".##.".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        };

        let resolved = resolve_tilemap(&tilemap, &ruleset);
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
            service.sync_ruleset_for_asset(&ruleset_asset, &horizontal_ruleset()),
            1
        );

        let resolved = service.commands()[0]
            .tilemap
            .resolved
            .clone()
            .expect("tilemap should be resolved");
        assert_eq!(
            resolved.rows[0][1].variant,
            Some(TileVariantKind2d::LeftCap)
        );
        assert_eq!(resolved.rows[0][2].variant, Some(TileVariantKind2d::Middle));
        assert_eq!(
            resolved.rows[0][3].variant,
            Some(TileVariantKind2d::RightCap)
        );
    }
}
