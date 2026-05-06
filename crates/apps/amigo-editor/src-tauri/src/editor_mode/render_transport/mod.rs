mod image_url;
mod native_surface;
mod stream;

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
        EditorFrameTransportKindDto::Stream => stream::publish_stream_frame(frame),
        EditorFrameTransportKindDto::NativeSurface => {
            native_surface::publish_native_surface_frame(frame)
        }
    }
}
