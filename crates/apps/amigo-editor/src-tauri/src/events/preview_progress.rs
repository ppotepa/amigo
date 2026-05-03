use tauri::{AppHandle, Emitter};

use crate::dto::ScenePreviewFrameGeneratedDto;

pub const PREVIEW_PROGRESS: &str = "preview-progress";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewProgressPayload {
    pub mod_id: String,
    pub scene_id: String,
    pub current: u32,
    pub total: u32,
    pub phase: String,
}

pub fn emit_preview_progress(
    app: &AppHandle,
    payload: PreviewProgressPayload,
) -> Result<(), String> {
    app.emit(PREVIEW_PROGRESS, payload)
        .map_err(|error| format!("failed to emit preview progress: {error}"))
}

impl From<ScenePreviewFrameGeneratedDto> for PreviewProgressPayload {
    fn from(value: ScenePreviewFrameGeneratedDto) -> Self {
        Self {
            mod_id: value.mod_id,
            scene_id: value.scene_id,
            current: value.current,
            total: value.total,
            phase: "rendering-frame".to_owned(),
        }
    }
}
