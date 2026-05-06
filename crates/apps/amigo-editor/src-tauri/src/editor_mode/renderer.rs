use tauri::AppHandle;

use crate::cache::root::EditorPaths;
use crate::preview;

use super::dto::EditorFrameDto;
use super::render_transport::{EditorRenderedFrame, publish_editor_frame};
use super::session::EditorModeSession;

pub async fn render_editor_mode_frame(
    app: AppHandle,
    paths: &EditorPaths,
    session: &EditorModeSession,
) -> Result<EditorFrameDto, String> {
    let preview = preview::request_scene_preview_dto(
        app,
        paths,
        session.mod_id.clone(),
        session.scene_id.clone(),
        false,
    )
    .await?;

    let rendered = EditorRenderedFrame {
        session_id: session.editor_mode_session_id.clone(),
        revision: session.revision,
        width: preview.width,
        height: preview.height,
        device_pixel_ratio: session.viewport.device_pixel_ratio,
        image_url: preview
            .image_url
            .or_else(|| preview.frame_urls.first().cloned()),
        rgba: Vec::new(),
    };

    publish_editor_frame(session.transport, rendered)
}
