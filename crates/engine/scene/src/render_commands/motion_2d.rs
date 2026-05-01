#[derive(Debug, Clone, PartialEq)]
pub struct MotionController2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub air_acceleration: f32,
    pub gravity: f32,
    pub jump_velocity: f32,
    pub terminal_velocity: f32,
}

impl MotionController2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        max_speed: f32,
        acceleration: f32,
        deceleration: f32,
        air_acceleration: f32,
        gravity: f32,
        jump_velocity: f32,
        terminal_velocity: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            max_speed,
            acceleration,
            deceleration,
            air_acceleration,
            gravity,
            jump_velocity,
            terminal_velocity,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CameraFollow2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub target: String,
    pub offset: Vec2,
    pub lerp: f32,
    pub lookahead_velocity_scale: f32,
    pub lookahead_max_distance: f32,
    pub sway_amount: f32,
    pub sway_frequency: f32,
}

impl CameraFollow2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        target: impl Into<String>,
        offset: Vec2,
        lerp: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            target: target.into(),
            offset,
            lerp,
            lookahead_velocity_scale: 0.0,
            lookahead_max_distance: 0.0,
            sway_amount: 0.0,
            sway_frequency: 0.0,
        }
    }

    pub fn with_lookahead(mut self, velocity_scale: f32, max_distance: f32) -> Self {
        self.lookahead_velocity_scale = velocity_scale;
        self.lookahead_max_distance = max_distance;
        self
    }

    pub fn with_sway(mut self, amount: f32, frequency: f32) -> Self {
        self.sway_amount = amount;
        self.sway_frequency = frequency;
        self
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Parallax2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub camera: String,
    pub factor: Vec2,
    pub anchor: Vec2,
    pub camera_origin: Option<Vec2>,
}

impl Parallax2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        camera: impl Into<String>,
        factor: Vec2,
        anchor: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            camera: camera.into(),
            factor,
            anchor,
            camera_origin: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TileMapMarker2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub tilemap_entity: Option<String>,
    pub symbol: String,
    pub index: usize,
    pub offset: Vec2,
}

impl TileMapMarker2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        tilemap_entity: Option<String>,
        symbol: impl Into<String>,
        index: usize,
        offset: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            tilemap_entity,
            symbol: symbol.into(),
            index,
            offset,
        }
    }
}
