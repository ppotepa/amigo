use tauri::AppHandle;

use crate::cache::root::EditorPaths;

use super::dto::EditorFrameResultDto;
use super::renderer::render_editor_mode_frame;
use super::session::EditorModeSessionRegistry;

pub enum EditorDocumentPatchOperation {
    SetTransform2 { entity_id: String },
    SetTransform3 { entity_id: String },
    SetTilemapMarkerOffset { entity_id: String },
    SetAttachedLocalOffset { entity_id: String },
}

pub struct EditorDocumentPatchPlan {
    pub scene_id: String,
    pub operations: Vec<EditorDocumentPatchOperation>,
}

pub async fn save_editor_mode_session_changes(
    app: AppHandle,
    paths: &EditorPaths,
    registry: &EditorModeSessionRegistry,
    editor_mode_session_id: String,
) -> Result<EditorFrameResultDto, String> {
    let session = registry.update(&editor_mode_session_id, |session| {
        session.dirty = false;
        session.bump_revision();
        Ok(())
    })?;

    let frame = render_editor_mode_frame(app, paths, &session).await?;

    Ok(EditorFrameResultDto {
        ok: true,
        session: Some(session.dto()),
        snapshot: Some(session.snapshot.clone()),
        frame: Some(frame),
        diagnostics: session.diagnostics.clone(),
        message: Some("Editor mode session saved.".to_owned()),
    })
}
