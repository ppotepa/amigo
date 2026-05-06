use super::super::dto::{EditorFrameDto, EditorFrameTransportKindDto};
use super::EditorRenderedFrame;

pub fn publish_image_url_frame(frame: EditorRenderedFrame) -> Result<EditorFrameDto, String> {
    Ok(EditorFrameDto {
        session_id: frame.session_id,
        revision: frame.revision,
        transport: EditorFrameTransportKindDto::ImageUrl,
        width: frame.width,
        height: frame.height,
        device_pixel_ratio: frame.device_pixel_ratio,
        image_url: frame.image_url,
        stream_id: None,
        surface_id: None,
        render_time_ms: None,
        encoded_bytes: Some(frame.rgba.len() as u64),
    })
}
