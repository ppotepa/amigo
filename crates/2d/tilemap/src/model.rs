use amigo_assets::AssetKey;
use amigo_math::Vec2;
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
