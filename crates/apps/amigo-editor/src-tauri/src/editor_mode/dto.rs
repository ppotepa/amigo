use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::dto::EditorDiagnosticDto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorSceneCanvasKindDto {
    #[serde(rename = "2d")]
    TwoD,
    #[serde(rename = "2.5d")]
    TwoHalfD,
    #[serde(rename = "3d")]
    ThreeD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorSceneSnapshotLayoutSourceDto {
    Runtime,
    Document,
    Fallback,
    Unavailable,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneSnapshotQualityDto {
    pub indexed_entities: usize,
    pub objects: usize,
    pub editable_objects: usize,
    pub objects_without_transform: usize,
    pub objects_without_bounds: usize,
    pub unsupported_bounds_providers: usize,
    pub diagnostics_by_code: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCameraDto {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTransform2Dto {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorTransform3Dto {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorBounds2Dto {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneObjectDto {
    pub entity_id: String,
    pub name: String,
    pub visible: bool,
    pub selectable: bool,
    pub locked: bool,
    pub category: String,
    pub component_types: Vec<String>,
    #[serde(rename = "transform2")]
    pub transform_2: Option<EditorTransform2Dto>,
    #[serde(rename = "transform3")]
    pub transform_3: Option<EditorTransform3Dto>,
    #[serde(rename = "bounds2")]
    pub bounds_2: Option<EditorBounds2Dto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSceneSnapshotDto {
    pub mod_id: String,
    pub scene_id: String,
    pub canvas_kind: EditorSceneCanvasKindDto,
    pub layout_source: EditorSceneSnapshotLayoutSourceDto,
    pub width: u32,
    pub height: u32,
    pub camera: EditorCameraDto,
    pub quality: EditorSceneSnapshotQualityDto,
    pub objects: Vec<EditorSceneObjectDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorViewportPointDto {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorHitTestCandidateDto {
    pub entity_id: String,
    pub name: String,
    pub depth: i32,
    #[serde(rename = "bounds2")]
    pub bounds_2: Option<EditorBounds2Dto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorHitTestResultDto {
    pub hit: bool,
    pub entity_id: Option<String>,
    pub candidates: Vec<EditorHitTestCandidateDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditorCommandDto {
    #[serde(rename = "SelectEntity", rename_all = "camelCase")]
    SelectEntity { scene_id: String, entity_id: String },
    #[serde(rename = "SetEntityTransform2D", rename_all = "camelCase")]
    SetEntityTransform2D {
        scene_id: String,
        entity_id: String,
        transform: EditorTransform2Dto,
    },
    #[serde(rename = "MoveEntity2D", rename_all = "camelCase")]
    MoveEntity2D {
        scene_id: String,
        entity_id: String,
        dx: f32,
        dy: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCommandResultDto {
    pub ok: bool,
    pub scene_dirty: bool,
    pub changed_entities: Vec<String>,
    pub snapshot: Option<EditorSceneSnapshotDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorLiveSceneSessionStatusDto {
    Opening,
    Ready,
    Dirty,
    Saving,
    Closed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLiveSceneSessionDto {
    pub editor_scene_session_id: String,
    pub editor_session_id: String,
    pub scene_id: String,
    pub mode: String,
    pub status: EditorLiveSceneSessionStatusDto,
    pub dirty: bool,
    pub revision: u64,
    pub opened_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenEditorLiveSceneSessionResultDto {
    pub session: EditorLiveSceneSessionDto,
    pub snapshot: EditorSceneSnapshotDto,
    pub diagnostics: Vec<EditorDiagnosticDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLiveCommandResultDto {
    pub ok: bool,
    pub session: Option<EditorLiveSceneSessionDto>,
    pub snapshot: Option<EditorSceneSnapshotDto>,
    pub diagnostics: Vec<EditorDiagnosticDto>,
    pub message: Option<String>,
}
