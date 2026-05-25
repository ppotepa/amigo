#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderDepthMode2d {
    #[default]
    DepthMap,
    Distance,
    ZDepth,
    Infinity,
    Overlay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDepth2d {
    pub mode: RenderDepthMode2d,
    pub distance_m: Option<f32>,
    pub z_depth: f32,
    pub blur_scale: f32,
}

impl Default for RenderDepth2d {
    fn default() -> Self {
        Self {
            mode: RenderDepthMode2d::DepthMap,
            distance_m: None,
            z_depth: 0.5,
            blur_scale: 1.0,
        }
    }
}

impl RenderDepth2d {
    pub fn normalized(mut self) -> Self {
        self.z_depth = self.z_depth.clamp(0.0, 1.0);
        self.blur_scale = self.blur_scale.clamp(0.0, 4.0);
        self.distance_m = self
            .distance_m
            .filter(|value| value.is_finite())
            .map(|value| value.max(0.0));
        self
    }

    pub fn is_depth_map(&self) -> bool {
        self.mode == RenderDepthMode2d::DepthMap
    }

    pub fn is_z_depth(&self) -> bool {
        self.mode == RenderDepthMode2d::ZDepth
    }

    pub fn is_distance(&self) -> bool {
        self.mode == RenderDepthMode2d::Distance
    }

    pub fn is_infinity(&self) -> bool {
        self.mode == RenderDepthMode2d::Infinity
    }

    pub fn is_constant_depth_plane(&self) -> bool {
        matches!(
            self.mode,
            RenderDepthMode2d::Distance | RenderDepthMode2d::ZDepth | RenderDepthMode2d::Infinity
        )
    }

    pub fn is_overlay(&self) -> bool {
        self.mode == RenderDepthMode2d::Overlay
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLayer2dCommand {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub order: f32,
    pub visible: bool,
    pub opacity: f32,
    pub depth: RenderDepth2d,
    pub optical_role: amigo_2d_spatial::OpticalLayerRole2d,
}

impl RenderLayer2dCommand {
    pub fn default_layer(source_mod: impl Into<String>) -> Self {
        Self {
            source_mod: source_mod.into(),
            id: "default".to_owned(),
            label: Some("Default".to_owned()),
            order: 0.0,
            visible: true,
            opacity: 1.0,
            depth: RenderDepth2d::default(),
            optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRoute2dCommand {
    pub source_mod: String,
    pub receiver_layer: String,
    pub groups: Vec<String>,
}

pub trait Composition2dRenderOutput {
    fn push_render_layer2d_command(&mut self, command: RenderLayer2dCommand);
    fn push_light_route2d_command(&mut self, command: LightRoute2dCommand);
}
