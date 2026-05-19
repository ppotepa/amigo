#[derive(Debug, Clone, PartialEq)]
pub enum CameraOpticalCoverage2d {
    LightMapChannel { source: String, channel: String },
    Hotspot { entity_name: String, radius_px: f32 },
    Glyphs { entity_name: String, render_layer: String },
    TextureAlpha { entity_name: String, render_layer: String },
    VectorCoverage { entity_name: String, render_layer: String },
    ParticleCoverage { emitter_entity_name: String },
    Unsupported { reason: String },
}

impl CameraOpticalCoverage2d {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::LightMapChannel { .. } => "lightmap_channel",
            Self::Hotspot { .. } => "hotspot",
            Self::Glyphs { .. } => "glyphs",
            Self::TextureAlpha { .. } => "texture_alpha",
            Self::VectorCoverage { .. } => "vector_coverage",
            Self::ParticleCoverage { .. } => "particle_coverage",
            Self::Unsupported { .. } => "unsupported",
        }
    }

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
