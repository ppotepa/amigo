pub const COMPOSITION_2D_CAPABILITY: &str = "composition_2d";
pub const COMPOSITION_2D_PLUGIN_LABEL: &str = "amigo-2d-composition";

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLayer2dCommand {
    pub source_mod: String,
    pub id: String,
    pub label: Option<String>,
    pub order: f32,
    pub visible: bool,
    pub opacity: f32,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRoute2dCommand {
    pub source_mod: String,
    pub receiver_layer: String,
    pub groups: Vec<String>,
}
