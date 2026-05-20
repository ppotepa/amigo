#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraCaptureContract {
    pub binding: crate::CameraBinding,
    pub debug_view: crate::CameraDebugViewId,
}
