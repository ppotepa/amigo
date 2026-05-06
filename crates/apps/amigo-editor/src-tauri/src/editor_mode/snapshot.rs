use std::collections::BTreeMap;

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::editor_mode::dto::{
    EditorCameraDto, EditorSceneCanvasKindDto, EditorSceneSnapshotDto,
    EditorSceneSnapshotLayoutSourceDto, EditorSceneSnapshotQualityDto,
};
use crate::editor_mode::gizmos::{default_selection, default_tool_state};

pub fn fallback_editor_snapshot(
    mod_id: String,
    scene_id: String,
    entity_count: usize,
) -> EditorSceneSnapshotDto {
    let width = 1280;
    let height = 720;

    EditorSceneSnapshotDto {
        mod_id,
        scene_id,
        canvas_kind: EditorSceneCanvasKindDto::TwoD,
        layout_source: EditorSceneSnapshotLayoutSourceDto::Fallback,
        width,
        height,
        camera: EditorCameraDto {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            viewport_width: width as f32,
            viewport_height: height as f32,
        },
        quality: EditorSceneSnapshotQualityDto {
            indexed_entities: entity_count,
            objects: 0,
            editable_objects: 0,
            objects_without_transform: 0,
            objects_without_bounds: 0,
            unsupported_bounds_providers: 0,
            diagnostics_by_code: BTreeMap::from([("EDITOR_MODE_LAYOUT_UNAVAILABLE".to_owned(), 1)]),
        },
        objects: Vec::new(),
        diagnostics: vec![EditorDiagnosticDto {
            level: DiagnosticLevel::Info,
            code: "EDITOR_MODE_LAYOUT_UNAVAILABLE".to_owned(),
            message: format!(
                "Editor-mode has indexed {entity_count} scene entities, but real transforms/bounds are not available yet."
            ),
            path: None,
        }],
        gizmos: Vec::new(),
        selection: default_selection(),
        tool_state: default_tool_state(),
    }
}
