use super::*;
use crate::editor_mode::coordinates::{EditorCamera2D, EditorCoordinateMapper, EditorFrameSize};
use crate::editor_mode::dto::{
    EditorCameraDto, EditorFrameTransportKindDto, EditorGizmoDto, EditorGizmoKindDto,
    EditorGizmoPrimitiveDto, EditorGizmoRectDto, EditorGizmoToneDto, EditorModeDto,
    EditorSceneCanvasKindDto, EditorSceneSnapshotDto, EditorSceneSnapshotLayoutSourceDto,
    EditorSceneSnapshotQualityDto, EditorToolDto, EditorViewportDto,
};
use crate::editor_mode::gizmos::{default_selection, default_tool_state};
use crate::editor_mode::session::EditorModeSession;
use crate::editor_mode::transaction::EditorTransactionLog;
use std::path::PathBuf;

#[test]
fn returns_svg_data_url_for_debug_overlay_without_gizmos() {
    let snapshot = test_snapshot(Vec::new());
    let image_url = Some("data:image/png;base64,AAAA".to_owned());
    let result = compose_editor_overlay_image_url(
        image_url,
        1280,
        720,
        &snapshot,
        &test_session(snapshot.clone()),
        test_mapper(),
    )
    .expect("overlay url");

    assert!(result.starts_with("data:image/svg+xml;base64,"));
}

#[test]
fn returns_svg_data_url_when_gizmos_exist() {
    let snapshot = test_snapshot(vec![EditorGizmoDto {
        id: "selection:player".to_owned(),
        kind: EditorGizmoKindDto::SelectionBounds2D,
        entity_id: Some("player".to_owned()),
        primitives: vec![EditorGizmoPrimitiveDto::Rect2D {
            rect: EditorGizmoRectDto {
                x: -10.0,
                y: -20.0,
                width: 20.0,
                height: 40.0,
            },
            tone: EditorGizmoToneDto::Selection,
        }],
        handles: Vec::new(),
    }]);

    let result = compose_editor_overlay_image_url(
        Some("data:image/png;base64,AAAA".to_owned()),
        1280,
        720,
        &snapshot,
        &test_session(snapshot.clone()),
        test_mapper(),
    )
    .expect("overlay url");

    assert!(result.starts_with("data:image/svg+xml;base64,"));
}

fn test_snapshot(gizmos: Vec<EditorGizmoDto>) -> EditorSceneSnapshotDto {
    EditorSceneSnapshotDto {
        mod_id: "test".to_owned(),
        scene_id: "scene".to_owned(),
        canvas_kind: EditorSceneCanvasKindDto::TwoD,
        layout_source: EditorSceneSnapshotLayoutSourceDto::Document,
        width: 1280,
        height: 720,
        camera: EditorCameraDto {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
        },
        quality: EditorSceneSnapshotQualityDto::default(),
        objects: Vec::new(),
        ui_nodes: Vec::new(),
        diagnostics: Vec::new(),
        gizmos,
        selection: default_selection(),
        tool_state: default_tool_state(),
    }
}

fn test_mapper() -> EditorCoordinateMapper {
    EditorCoordinateMapper {
        frame: EditorFrameSize {
            width: 1280.0,
            height: 720.0,
        },
        camera: EditorCamera2D::default(),
    }
}

fn test_session(snapshot: EditorSceneSnapshotDto) -> EditorModeSession {
    EditorModeSession {
        editor_mode_session_id: "editor-mode-test".to_owned(),
        editor_session_id: "editor-session-test".to_owned(),
        mod_id: "test".to_owned(),
        root_path: PathBuf::new(),
        scene_id: "scene".to_owned(),
        document_value: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        mode: EditorModeDto::Edit,
        tool: EditorToolDto::Select,
        viewport: EditorViewportDto {
            css_width: 1280.0,
            css_height: 720.0,
            render_width: 1280,
            render_height: 720,
            device_pixel_ratio: 1.0,
            camera_x: Some(0.0),
            camera_y: Some(0.0),
            zoom: Some(1.0),
        },
        transport: EditorFrameTransportKindDto::ImageUrl,
        dirty: false,
        revision: 1,
        selected_entity_id: None,
        selected_ui_node: None,
        active_interaction: None,
        hovered_control_id: None,
        hovered_handle_id: None,
        hovered_entity_id: None,
        hovered_ui_node: None,
        active_control_id: None,
        active_handle_id: None,
        cursor: crate::editor_mode::default_editor_cursor(),
        last_pointer_scene_x: None,
        last_pointer_scene_y: None,
        last_pointer_frame_x: None,
        last_pointer_frame_y: None,
        transactions: EditorTransactionLog::default(),
        snapshot,
        diagnostics: Vec::new(),
    }
}
