mod image_url;

use super::dto::{EditorFrameDto, EditorFrameTransportKindDto};

pub struct EditorRenderedFrame {
    pub session_id: String,
    pub revision: u64,
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f32,
    pub image_url: Option<String>,
    pub rgba: Vec<u8>,
}

pub fn publish_editor_frame(
    kind: EditorFrameTransportKindDto,
    frame: EditorRenderedFrame,
) -> Result<EditorFrameDto, String> {
    match kind {
        EditorFrameTransportKindDto::ImageUrl => image_url::publish_image_url_frame(frame),
    }
}
