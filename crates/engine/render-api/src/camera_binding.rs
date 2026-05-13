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

