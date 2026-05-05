use tauri::State;

use crate::editor_mode::dto::{
    EditorLiveCommandResultDto, EditorSceneSnapshotDto, EditorTransform2Dto,
    OpenEditorLiveSceneSessionResultDto,
};
use crate::editor_mode::live_session::EditorLiveSceneSessions;
use crate::editor_mode::runtime_snapshot::runtime_editor_snapshot_from_live_session;
use crate::session::EditorSessionRegistry;

pub fn open_editor_scene_session(
    session_id: String,
    scene_id: String,
    sessions: State<'_, EditorSessionRegistry>,
    live_sessions: State<'_, EditorLiveSceneSessions>,
) -> Result<OpenEditorLiveSceneSessionResultDto, String> {
    let session = sessions.get_session(&session_id)?;
    live_sessions.open_session(
        session.session_id,
        session.mod_id,
        session.root_path,
        scene_id,
    )
}

pub fn close_editor_scene_session(
    session_id: String,
    editor_scene_session_id: String,
    live_sessions: State<'_, EditorLiveSceneSessions>,
) -> Result<EditorLiveCommandResultDto, String> {
    live_sessions.close_session(&session_id, &editor_scene_session_id)
}

pub fn get_runtime_editor_snapshot(
    session_id: String,
    editor_scene_session_id: String,
    live_sessions: State<'_, EditorLiveSceneSessions>,
) -> Result<EditorSceneSnapshotDto, String> {
    runtime_editor_snapshot_from_live_session(&live_sessions, &session_id, &editor_scene_session_id)
}

pub fn apply_editor_live_transform(
    session_id: String,
    editor_scene_session_id: String,
    entity_id: String,
    transform: EditorTransform2Dto,
    live_sessions: State<'_, EditorLiveSceneSessions>,
) -> Result<EditorLiveCommandResultDto, String> {
    live_sessions.apply_transform(&session_id, &editor_scene_session_id, entity_id, transform)
}

pub fn commit_editor_scene_session(
    session_id: String,
    editor_scene_session_id: String,
    live_sessions: State<'_, EditorLiveSceneSessions>,
) -> Result<EditorLiveCommandResultDto, String> {
    live_sessions.commit(&session_id, &editor_scene_session_id)
}

pub fn discard_editor_scene_session(
    session_id: String,
    editor_scene_session_id: String,
    live_sessions: State<'_, EditorLiveSceneSessions>,
) -> Result<EditorLiveCommandResultDto, String> {
    live_sessions.discard(&session_id, &editor_scene_session_id)
}
