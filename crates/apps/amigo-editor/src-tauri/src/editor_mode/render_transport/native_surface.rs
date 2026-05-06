use super::super::dto::{EditorFrameDto, EditorFrameTransportKindDto};
use super::EditorRenderedFrame;

pub fn publish_native_surface_frame(frame: EditorRenderedFrame) -> Result<EditorFrameDto, String> {
    Ok(EditorFrameDto {
        session_id: frame.session_id.clone(),
        revision: frame.revision,
        transport: EditorFrameTransportKindDto::NativeSurface,
        width: frame.width,
        height: frame.height,
        device_pixel_ratio: frame.device_pixel_ratio,
        image_url: None,
        stream_id: None,
        surface_id: Some(frame.session_id),
        render_time_ms: None,
        encoded_bytes: Some(frame.rgba.len() as u64),
    })
}
