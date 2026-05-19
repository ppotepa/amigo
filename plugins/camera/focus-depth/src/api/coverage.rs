#[derive(Clone, Debug, PartialEq)]
pub enum FocusDepthCoverage2d {
    SceneDepth,
    RenderLayer { layer_id: String },
    SceneObject { entity_name: String },
    Distance { meters: f32 },
    Unsupported { reason: String },
}

impl FocusDepthCoverage2d {
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported { .. })
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match self {
            Self::Unsupported { reason } => Some(reason.as_str()),
            _ => None,
        }
    }
}
