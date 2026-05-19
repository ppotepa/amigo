use crate::api::FocusDepthResponse2d;

use super::FocusDepthResponse2dDocument;

pub fn focus_depth_response_from_document(
    document: FocusDepthResponse2dDocument,
) -> FocusDepthResponse2d {
    FocusDepthResponse2d {
        enabled: document.enabled,
        strength: document.strength,
        focus_width_m: document.focus_width_m,
        max_blur_px: document.max_blur_px,
    }
    .normalized()
}
