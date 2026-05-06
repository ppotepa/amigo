use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dto::EditorDiagnosticDto;

use super::dto::{
    EditorCursorDto, EditorFrameTransportKindDto, EditorModeDto, EditorModeSessionDto,
    EditorRenderTransportPreferenceDto, EditorSceneSnapshotDto, EditorToolDto, EditorTransform2Dto,
    EditorUiNodeSelectionDto, EditorViewportDto,
};
use super::transaction::EditorTransactionLog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorActiveInteractionKind {
    SelectEntity,
    MoveEntity2D,
    MoveAxisX,
    MoveAxisY,
    PanViewport,
}

#[derive(Debug, Clone)]
pub struct EditorActiveInteraction {
    pub kind: EditorActiveInteractionKind,
    pub entity_id: Option<String>,
    pub start_pointer_x: f32,
    pub start_pointer_y: f32,
    pub start_transform_2: Option<EditorTransform2Dto>,
    pub changed_entities: Vec<String>,
}

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
    pub selected_ui_node: Option<EditorUiNodeSelectionDto>,
    pub active_interaction: Option<EditorActiveInteraction>,
    pub hovered_control_id: Option<String>,
    pub hovered_handle_id: Option<String>,
    pub hovered_entity_id: Option<String>,
    pub hovered_ui_node: Option<EditorUiNodeSelectionDto>,
    pub active_control_id: Option<String>,
    pub active_handle_id: Option<String>,
    pub cursor: EditorCursorDto,
    pub last_pointer_scene_x: Option<f32>,
    pub last_pointer_scene_y: Option<f32>,
    pub last_pointer_frame_x: Option<f32>,
    pub last_pointer_frame_y: Option<f32>,
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
            can_undo: self.transactions.can_undo(),
            can_redo: self.transactions.can_redo(),
            revision: self.revision,
            transport: self.transport,
            cursor: self.cursor.clone(),
            hovered_control_id: self.hovered_control_id.clone(),
            hovered_handle_id: self.hovered_handle_id.clone(),
            active_control_id: self.active_control_id.clone(),
            active_handle_id: self.active_handle_id.clone(),
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
        EditorRenderTransportPreferenceDto::Auto | EditorRenderTransportPreferenceDto::ImageUrl => {
            EditorFrameTransportKindDto::ImageUrl
        }
    }
}
