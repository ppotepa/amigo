use crate::{CameraProjection, Viewport};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CameraId(pub String);

impl CameraId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn main() -> Self {
        Self::new("main")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub id: CameraId,
    pub projection: CameraProjection,
    pub viewport: Viewport,
}
