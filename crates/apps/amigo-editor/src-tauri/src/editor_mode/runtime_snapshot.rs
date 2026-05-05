use crate::editor_mode::dto::EditorSceneSnapshotDto;
use crate::editor_mode::live_session::EditorLiveSceneSessions;

pub fn runtime_editor_snapshot_from_live_session(
    live_sessions: &EditorLiveSceneSessions,
    editor_session_id: &str,
    editor_scene_session_id: &str,
) -> Result<EditorSceneSnapshotDto, String> {
    live_sessions.snapshot(editor_session_id, editor_scene_session_id)
}
