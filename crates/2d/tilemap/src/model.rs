use amigo_assets::AssetKey;
use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileCollisionKind2d {
    None,
    Solid,
    Full,
    OneWay,
    SlopeLeft,
    SlopeRight,
    Slope45Left,
    Slope45Right,
    HalfTop,
    HalfBottom,
    CustomPolygon,
    Trigger,
}

impl TileCollisionKind2d {
    pub fn from_contract_str(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "solid" => Some(Self::Solid),
            "full" => Some(Self::Full),
            "one_way" | "one-way" => Some(Self::OneWay),
            "slope_left" => Some(Self::SlopeLeft),
            "slope_right" => Some(Self::SlopeRight),
            "slope_45_left" => Some(Self::Slope45Left),
            "slope_45_right" => Some(Self::Slope45Right),
            "half_top" => Some(Self::HalfTop),
            "half_bottom" => Some(Self::HalfBottom),
            "custom_polygon" => Some(Self::CustomPolygon),
            "trigger" => Some(Self::Trigger),
            _ => None,
        }
    }

    pub fn creates_full_solid_collider(self) -> bool {
        matches!(self, Self::Solid | Self::Full)
    }
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

    pub fn iter_tile_ids(&self) -> impl Iterator<Item = u32> {
        [
            self.single,
            self.left_cap,
            self.middle,
            self.right_cap,
            self.side_left,
            self.side_right,
            self.center,
            self.top_cap,
            self.bottom_cap,
            self.vertical_middle,
            self.inner_corner_top_left,
            self.inner_corner_top_right,
            self.inner_corner_bottom_left,
            self.inner_corner_bottom_right,
            self.outer_corner_top_left,
            self.outer_corner_top_right,
            self.outer_corner_bottom_left,
            self.outer_corner_bottom_right,
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileTerrainRule2d {
    pub name: String,
    pub symbol: char,
    pub collision: TileCollisionKind2d,
    pub unknown_collision: Option<String>,
    pub paint: Option<TilePaintRule2d>,
    pub variants: TileVariantSet2d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TilePaintRule2d {
    pub brush: String,
    pub category: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileMarkerRule2d {
    pub name: String,
    pub symbol: char,
    pub label: String,
    pub entity_template: Option<String>,
    pub max_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileRuleSetSymbols2d {
    pub empty: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileRuleSet2d {
    pub tile_size: Option<(u32, u32)>,
    pub symbols: TileRuleSetSymbols2d,
    pub terrains: Vec<TileTerrainRule2d>,
    pub markers: Vec<TileMarkerRule2d>,
}

impl TileRuleSet2d {
    pub fn terrain_for_symbol(&self, symbol: char) -> Option<&TileTerrainRule2d> {
        self.terrains
            .iter()
            .find(|terrain| terrain.symbol == symbol)
    }

    pub fn marker_for_symbol(&self, symbol: char) -> Option<&TileMarkerRule2d> {
        self.markers.iter().find(|marker| marker.symbol == symbol)
    }

    pub fn empty_symbol(&self) -> char {
        self.symbols.empty.unwrap_or('.')
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
