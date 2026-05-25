#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraBinding {
    pub camera_id: String,
    pub recovery: CameraRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraRecovery {
    Main,
    None,
}

impl CameraBinding {
    pub fn main() -> Self {
        Self {
            camera_id: "main".to_string(),
            recovery: CameraRecovery::Main,
        }
    }

    pub fn none(camera_id: impl Into<String>) -> Self {
        Self {
            camera_id: camera_id.into(),
            recovery: CameraRecovery::None,
        }
    }
}
