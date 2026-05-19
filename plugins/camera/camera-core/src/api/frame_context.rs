#[derive(Clone, Debug, PartialEq)]
pub struct CameraFrameContext2d {
    pub camera_id: String,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub frame_index: u64,
    pub delta_time_s: f32,
}
