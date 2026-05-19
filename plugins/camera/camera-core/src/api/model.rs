use crate::{CameraProjection, Viewport};
pub use amigo_camera_optics_plugin::runtime::CameraId;

#[derive(Debug, Clone)]
pub struct Camera {
    pub id: CameraId,
    pub projection: CameraProjection,
    pub viewport: Viewport,
}
