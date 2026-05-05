use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::editor_mode::document_commands::apply_document_transform_2d;
use crate::editor_mode::document_snapshot::document_editor_snapshot;
use crate::editor_mode::dto::{
    EditorLiveCommandResultDto, EditorLiveSceneSessionDto, EditorLiveSceneSessionStatusDto,
    EditorSceneSnapshotDto, EditorSceneSnapshotLayoutSourceDto, EditorTransform2Dto,
    OpenEditorLiveSceneSessionResultDto,
};

#[derive(Debug, Clone)]
pub struct EditorLiveSceneSession {
    pub dto: EditorLiveSceneSessionDto,
    pub mod_id: String,
    pub root_path: String,
    pub snapshot: EditorSceneSnapshotDto,
    pub pending_transforms: HashMap<String, EditorTransform2Dto>,
}

#[derive(Debug, Default)]
pub struct EditorLiveSceneSessions {
    state: Mutex<EditorLiveSceneSessionsState>,
}

#[derive(Debug, Default)]
struct EditorLiveSceneSessionsState {
    next_id: u64,
    sessions: HashMap<String, EditorLiveSceneSession>,
}

impl EditorLiveSceneSessions {
    pub fn open_session(
        &self,
        editor_session_id: String,
        mod_id: String,
        root_path: String,
        scene_id: String,
    ) -> Result<OpenEditorLiveSceneSessionResultDto, String> {
        let mut snapshot = document_editor_snapshot(mod_id.clone(), &root_path, scene_id.clone())?;
        snapshot.layout_source = EditorSceneSnapshotLayoutSourceDto::Runtime;

        let mut state = self
            .state
            .lock()
            .map_err(|_| "failed to lock live editor sessions".to_owned())?;
        state.next_id += 1;

        let editor_scene_session_id =
            format!("live:{}:{}:{}", editor_session_id, scene_id, state.next_id);
        let dto = EditorLiveSceneSessionDto {
            editor_scene_session_id: editor_scene_session_id.clone(),
            editor_session_id,
            scene_id: scene_id.clone(),
            mode: "live".to_owned(),
            status: EditorLiveSceneSessionStatusDto::Ready,
            dirty: false,
            revision: 1,
            opened_at_ms: now_ms(),
        };

        state.sessions.insert(
            editor_scene_session_id,
            EditorLiveSceneSession {
                dto: dto.clone(),
                mod_id,
                root_path,
                snapshot: snapshot.clone(),
                pending_transforms: HashMap::new(),
            },
        );

        Ok(OpenEditorLiveSceneSessionResultDto {
            session: dto,
            snapshot,
            diagnostics: Vec::new(),
        })
    }

    pub fn close_session(
        &self,
        editor_session_id: &str,
        editor_scene_session_id: &str,
    ) -> Result<EditorLiveCommandResultDto, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "failed to lock live editor sessions".to_owned())?;
        let Some(mut session) = state.sessions.remove(editor_scene_session_id) else {
            return Ok(live_error(
                "LIVE_SESSION_NOT_FOUND",
                "Live editor session was not found.",
            ));
        };

        ensure_owner(&session, editor_session_id)?;
        session.dto.status = EditorLiveSceneSessionStatusDto::Closed;

        Ok(EditorLiveCommandResultDto {
            ok: true,
            session: Some(session.dto),
            snapshot: None,
            diagnostics: Vec::new(),
            message: Some("Live editor session closed.".to_owned()),
        })
    }

    pub fn snapshot(
        &self,
        editor_session_id: &str,
        editor_scene_session_id: &str,
    ) -> Result<EditorSceneSnapshotDto, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "failed to lock live editor sessions".to_owned())?;
        let session = state.sessions.get(editor_scene_session_id).ok_or_else(|| {
            "LIVE_SESSION_NOT_FOUND: Live editor session was not found.".to_owned()
        })?;

        ensure_owner(session, editor_session_id)?;
        Ok(session.snapshot.clone())
    }

    pub fn apply_transform(
        &self,
        editor_session_id: &str,
        editor_scene_session_id: &str,
        entity_id: String,
        transform: EditorTransform2Dto,
    ) -> Result<EditorLiveCommandResultDto, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "failed to lock live editor sessions".to_owned())?;
        let session = state
            .sessions
            .get_mut(editor_scene_session_id)
            .ok_or_else(|| {
                "LIVE_SESSION_NOT_FOUND: Live editor session was not found.".to_owned()
            })?;

        ensure_owner(session, editor_session_id)?;

        let Some(object) = session
            .snapshot
            .objects
            .iter_mut()
            .find(|object| object.entity_id == entity_id)
        else {
            return Ok(live_error(
                "ENTITY_NOT_FOUND",
                format!("Entity `{entity_id}` was not found in live snapshot."),
            ));
        };

        if object.transform_2.is_none() {
            return Ok(live_error(
                "ENTITY_NO_TRANSFORM2",
                format!("Entity `{entity_id}` has no editable transform2."),
            ));
        }

        if let Some(bounds) = object.bounds_2.as_mut() {
            bounds.x = transform.x - bounds.width / 2.0;
            bounds.y = transform.y - bounds.height / 2.0;
        }

        object.transform_2 = Some(transform.clone());
        session.pending_transforms.insert(entity_id, transform);
        session.dto.dirty = true;
        session.dto.status = EditorLiveSceneSessionStatusDto::Dirty;
        session.dto.revision += 1;
        session.snapshot.layout_source = EditorSceneSnapshotLayoutSourceDto::Runtime;

        Ok(EditorLiveCommandResultDto {
            ok: true,
            session: Some(session.dto.clone()),
            snapshot: Some(session.snapshot.clone()),
            diagnostics: Vec::new(),
            message: Some("Live transform applied in editor session.".to_owned()),
        })
    }

    pub fn commit(
        &self,
        editor_session_id: &str,
        editor_scene_session_id: &str,
    ) -> Result<EditorLiveCommandResultDto, String> {
        let session_snapshot = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| "failed to lock live editor sessions".to_owned())?;
            let session = state
                .sessions
                .get_mut(editor_scene_session_id)
                .ok_or_else(|| {
                    "LIVE_SESSION_NOT_FOUND: Live editor session was not found.".to_owned()
                })?;

            ensure_owner(session, editor_session_id)?;
            session.dto.status = EditorLiveSceneSessionStatusDto::Saving;
            session.clone()
        };

        let mut diagnostics = Vec::new();
        for (entity_id, transform) in session_snapshot.pending_transforms.iter() {
            let result = apply_document_transform_2d(
                session_snapshot.mod_id.clone(),
                &session_snapshot.root_path,
                session_snapshot.dto.scene_id.clone(),
                entity_id.clone(),
                transform.clone(),
            )?;
            diagnostics.extend(result.diagnostics);
            if !result.ok {
                return Ok(EditorLiveCommandResultDto {
                    ok: false,
                    session: Some(session_snapshot.dto),
                    snapshot: None,
                    diagnostics,
                    message: result.message,
                });
            }
        }

        let mut fresh_snapshot = document_editor_snapshot(
            session_snapshot.mod_id.clone(),
            &session_snapshot.root_path,
            session_snapshot.dto.scene_id.clone(),
        )?;
        fresh_snapshot.layout_source = EditorSceneSnapshotLayoutSourceDto::Runtime;
        diagnostics.extend(fresh_snapshot.diagnostics.clone());

        let mut state = self
            .state
            .lock()
            .map_err(|_| "failed to lock live editor sessions".to_owned())?;
        let session = state
            .sessions
            .get_mut(editor_scene_session_id)
            .ok_or_else(|| {
                "LIVE_SESSION_NOT_FOUND: Live editor session was not found.".to_owned()
            })?;

        ensure_owner(session, editor_session_id)?;
        session.pending_transforms.clear();
        session.snapshot = fresh_snapshot.clone();
        session.dto.dirty = false;
        session.dto.status = EditorLiveSceneSessionStatusDto::Ready;
        session.dto.revision += 1;

        Ok(EditorLiveCommandResultDto {
            ok: true,
            session: Some(session.dto.clone()),
            snapshot: Some(fresh_snapshot),
            diagnostics,
            message: Some("Live editor session committed to scene document.".to_owned()),
        })
    }

    pub fn discard(
        &self,
        editor_session_id: &str,
        editor_scene_session_id: &str,
    ) -> Result<EditorLiveCommandResultDto, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "failed to lock live editor sessions".to_owned())?;
        let session = state
            .sessions
            .get_mut(editor_scene_session_id)
            .ok_or_else(|| {
                "LIVE_SESSION_NOT_FOUND: Live editor session was not found.".to_owned()
            })?;

        ensure_owner(session, editor_session_id)?;

        let mut snapshot = document_editor_snapshot(
            session.mod_id.clone(),
            &session.root_path,
            session.dto.scene_id.clone(),
        )?;
        snapshot.layout_source = EditorSceneSnapshotLayoutSourceDto::Runtime;

        session.pending_transforms.clear();
        session.snapshot = snapshot.clone();
        session.dto.dirty = false;
        session.dto.status = EditorLiveSceneSessionStatusDto::Ready;
        session.dto.revision += 1;

        Ok(EditorLiveCommandResultDto {
            ok: true,
            session: Some(session.dto.clone()),
            snapshot: Some(snapshot),
            diagnostics: Vec::new(),
            message: Some("Live editor session changes discarded.".to_owned()),
        })
    }
}

fn ensure_owner(session: &EditorLiveSceneSession, editor_session_id: &str) -> Result<(), String> {
    if session.dto.editor_session_id == editor_session_id {
        return Ok(());
    }
    Err(
        "LIVE_SESSION_OWNER_MISMATCH: Live editor session belongs to another editor session."
            .to_owned(),
    )
}

fn live_error(code: &str, message: impl Into<String>) -> EditorLiveCommandResultDto {
    let message = message.into();
    EditorLiveCommandResultDto {
        ok: false,
        session: None,
        snapshot: None,
        diagnostics: vec![EditorDiagnosticDto {
            level: DiagnosticLevel::Error,
            code: code.to_owned(),
            message: message.clone(),
            path: None,
        }],
        message: Some(message),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
