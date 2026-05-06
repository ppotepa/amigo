use super::super::dto::{EditorFrameDto, EditorFrameTransportKindDto};
use super::EditorRenderedFrame;

pub fn publish_stream_frame(frame: EditorRenderedFrame) -> Result<EditorFrameDto, String> {
    Ok(EditorFrameDto {
        session_id: frame.session_id.clone(),
        revision: frame.revision,
        transport: EditorFrameTransportKindDto::Stream,
        width: frame.width,
        height: frame.height,
        device_pixel_ratio: frame.device_pixel_ratio,
        image_url: None,
        stream_id: Some(frame.session_id),
        surface_id: None,
        render_time_ms: None,
        encoded_bytes: Some(frame.rgba.len() as u64),
    })
}
