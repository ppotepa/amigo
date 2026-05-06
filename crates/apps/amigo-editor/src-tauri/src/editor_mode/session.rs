use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dto::EditorDiagnosticDto;

use super::dto::{
    EditorFrameTransportKindDto, EditorModeDto, EditorModeSessionDto,
    EditorRenderTransportPreferenceDto, EditorSceneSnapshotDto, EditorToolDto, EditorViewportDto,
};
use super::transaction::EditorTransactionLog;

#[derive(Debug, Clone)]
pub struct EditorModeSession {
    pub editor_mode_session_id: String,
    pub editor_session_id: String,
    pub mod_id: String,
    pub root_path: PathBuf,
    pub scene_id: String,
    pub mode: EditorModeDto,
    pub tool: EditorToolDto,
    pub viewport: EditorViewportDto,
    pub transport: EditorFrameTransportKindDto,
    pub dirty: bool,
    pub revision: u64,
    pub selected_entity_id: Option<String>,
    pub transactions: EditorTransactionLog,
    pub snapshot: EditorSceneSnapshotDto,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

impl EditorModeSession {
    pub fn dto(&self) -> EditorModeSessionDto {
        EditorModeSessionDto {
            editor_mode_session_id: self.editor_mode_session_id.clone(),
            editor_session_id: self.editor_session_id.clone(),
            mod_id: self.mod_id.clone(),
            scene_id: self.scene_id.clone(),
            mode: self.mode,
            tool: self.tool,
            dirty: self.dirty,
            revision: self.revision,
            transport: self.transport,
        }
    }

    pub fn bump_revision(&mut self) {
        self.revision += 1;
    }
}

#[derive(Default)]
pub struct EditorModeSessionRegistry {
    sessions: Mutex<HashMap<String, EditorModeSession>>,
}

impl EditorModeSessionRegistry {
    pub fn insert(&self, session: EditorModeSession) -> Result<EditorModeSessionDto, String> {
        let dto = session.dto();
        self.sessions
            .lock()
            .map_err(|_| "EDITOR_MODE_SESSION_REGISTRY_LOCK_FAILED".to_owned())?
            .insert(session.editor_mode_session_id.clone(), session);
        Ok(dto)
    }

    pub fn get(&self, id: &str) -> Result<EditorModeSession, String> {
        self.sessions
            .lock()
            .map_err(|_| "EDITOR_MODE_SESSION_REGISTRY_LOCK_FAILED".to_owned())?
            .get(id)
            .cloned()
            .ok_or_else(|| format!("EDITOR_MODE_SESSION_NOT_FOUND: {id}"))
    }

    pub fn update(
        &self,
        id: &str,
        update: impl FnOnce(&mut EditorModeSession) -> Result<(), String>,
    ) -> Result<EditorModeSession, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "EDITOR_MODE_SESSION_REGISTRY_LOCK_FAILED".to_owned())?;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("EDITOR_MODE_SESSION_NOT_FOUND: {id}"))?;
        update(session)?;
        Ok(session.clone())
    }

    pub fn remove(&self, id: &str) -> Result<(), String> {
        self.sessions
            .lock()
            .map_err(|_| "EDITOR_MODE_SESSION_REGISTRY_LOCK_FAILED".to_owned())?
            .remove(id);
        Ok(())
    }
}

pub fn new_editor_mode_session_id(editor_session_id: &str, scene_id: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("editor-mode-{editor_session_id}-{scene_id}-{millis}")
}

pub fn resolve_transport_kind(
    preference: EditorRenderTransportPreferenceDto,
) -> EditorFrameTransportKindDto {
    match preference {
        EditorRenderTransportPreferenceDto::ImageUrl => EditorFrameTransportKindDto::ImageUrl,
        EditorRenderTransportPreferenceDto::Stream => EditorFrameTransportKindDto::Stream,
        EditorRenderTransportPreferenceDto::NativeSurface => {
            EditorFrameTransportKindDto::NativeSurface
        }
        EditorRenderTransportPreferenceDto::Auto => EditorFrameTransportKindDto::ImageUrl,
    }
}
