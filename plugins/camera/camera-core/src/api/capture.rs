#[derive(Clone, Debug, PartialEq)]
pub struct CameraCaptureInput2d {
    pub exposure: f32,
    pub sensor_gain: f32,
    pub viewport_scale: f32,
}

impl Default for CameraCaptureInput2d {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            sensor_gain: 1.0,
            viewport_scale: 1.0,
        }
    }
}
