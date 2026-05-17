pub const COMPOSITION_2D_CAPABILITY: &str = "composition_2d";
pub const COMPOSITION_2D_PLUGIN_LABEL: &str = "amigo-2d-composition";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderDepthMode2d {
    #[default]
    DepthMap,
    Plane,
    Overlay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderDepth2d {
    pub mode: RenderDepthMode2d,
    pub value: f32,
    pub blur_scale: f32,
}

impl Default for RenderDepth2d {
    fn default() -> Self {
        Self {
            mode: RenderDepthMode2d::DepthMap,
            value: 0.5,
            blur_scale: 1.0,
        }
    }
}

impl RenderDepth2d {
    pub fn normalized(mut self) -> Self {
        self.value = self.value.clamp(0.0, 1.0);
        self.blur_scale = self.blur_scale.clamp(0.0, 4.0);
        self
    }

    pub fn is_depth_map(&self) -> bool {
        self.mode == RenderDepthMode2d::DepthMap
    }

    pub fn is_plane(&self) -> bool {
        self.mode == RenderDepthMode2d::Plane
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRoute2dCommand {
    pub source_mod: String,
    pub receiver_layer: String,
    pub groups: Vec<String>,
}
