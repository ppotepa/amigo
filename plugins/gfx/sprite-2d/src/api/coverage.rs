#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sprite2dCoverage {
    TextureAlpha {
        entity_name: String,
        render_layer: String,
    },
    Unsupported {
        reason: String,
    },
}

impl Sprite2dCoverage {
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported { .. })
    }
}
