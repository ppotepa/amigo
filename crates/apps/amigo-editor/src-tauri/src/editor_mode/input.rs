use tauri::AppHandle;

use crate::cache::root::EditorPaths;

use super::dto::{EditorFrameResultDto, EditorPointerEventDto};
use super::renderer::render_editor_mode_frame;
use super::session::EditorModeSessionRegistry;

pub async fn handle_editor_pointer_event(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
    event: EditorPointerEventDto,
) -> Result<EditorFrameResultDto, String> {
    let session = registry.update(&editor_mode_session_id, |session| {
        session.viewport = event.viewport.clone();

        match event.r#type.as_str() {
            "pointerDown" | "pointerMove" | "pointerUp" | "pointerCancel" | "wheel" => {
                session.bump_revision();
            }
            _ => {}
        }

        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: None,
    })
}
