use super::{CameraOpticalResponse2dDocument, CameraOpticalResponse2dSceneCommand};

pub fn camera_optical_response_from_document(
    response: CameraOpticalResponse2dDocument,
) -> CameraOpticalResponse2dSceneCommand {
    CameraOpticalResponse2dSceneCommand {
        enabled: response.enabled,
        intensity: response.intensity,
        bloom: response.bloom,
        glare: response.glare,
        ghosting: response.ghosting,
        streaks: response.streaks,
        chromatic_smear: response.chromatic_smear,
        dirt_response: response.dirt_response,
        halation: response.halation,
        threshold: response.threshold,
    }
}
