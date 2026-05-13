#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraBinding {
    pub camera_id: String,
    pub fallback: CameraFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraFallback {
    Main,
    None,
}

impl CameraBinding {
    pub fn main() -> Self {
        Self {
            camera_id: "main".to_string(),
            fallback: CameraFallback::Main,
        }
    }

    pub fn none(camera_id: impl Into<String>) -> Self {
        Self {
            camera_id: camera_id.into(),
            fallback: CameraFallback::None,
        }
    }
}

