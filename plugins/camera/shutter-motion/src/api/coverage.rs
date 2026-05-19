#[derive(Clone, Debug, PartialEq)]
pub enum MotionShutterCoverage2d {
    SceneVelocity,
    CameraMotion,
    RenderLayer { layer_id: String },
    Unsupported { reason: String },
}

impl MotionShutterCoverage2d {
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
