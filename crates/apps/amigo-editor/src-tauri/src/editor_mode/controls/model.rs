use crate::editor_mode::dto::{
    EditorControlDto, EditorControlHandleDto, EditorControlStateDto, EditorCursorDto,
    EditorGizmoDto, EditorGizmoHandleKindDto,
};

#[derive(Debug, Clone, Default)]
pub struct EditorControlBuildContext {
    pub hovered_control_id: Option<String>,
    pub hovered_handle_id: Option<String>,
    pub active_control_id: Option<String>,
    pub active_handle_id: Option<String>,
}

impl EditorControlBuildContext {
    pub fn is_hovered(&self, control_id: &str, handle_id: &str) -> bool {
        self.hovered_control_id.as_deref() == Some(control_id)
            && self.hovered_handle_id.as_deref() == Some(handle_id)
    }

    pub fn is_active(&self, control_id: &str, handle_id: &str) -> bool {
        self.active_control_id.as_deref() == Some(control_id)
            && self.active_handle_id.as_deref() == Some(handle_id)
    }

    #[allow(dead_code)]
    pub fn state_for(&self, control_id: &str, handle_id: &str) -> EditorControlStateDto {
        if self.is_active(control_id, handle_id) {
            EditorControlStateDto::Active
        } else if self.is_hovered(control_id, handle_id) {
            EditorControlStateDto::Hovered
        } else {
            EditorControlStateDto::Idle
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EditorControlHit {
    pub control_id: String,
    pub entity_id: Option<String>,
    pub handle_id: String,
    pub handle_kind: EditorGizmoHandleKindDto,
    pub cursor: Option<EditorCursorDto>,
}

#[allow(dead_code)]
pub fn flatten_control_gizmos(controls: &[EditorControlDto]) -> Vec<EditorGizmoDto> {
    controls
        .iter()
        .flat_map(|control| control.gizmos.iter().cloned())
        .collect()
}

#[allow(dead_code)]
pub fn flatten_control_handles(controls: &[EditorControlDto]) -> Vec<EditorControlHandleDto> {
    controls
        .iter()
        .flat_map(|control| control.handles.iter().cloned())
        .collect()
}
