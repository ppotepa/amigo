use super::builders::gizmos_for_selection;
use super::hit_test::hit_test_snapshot_entity;
use super::state::{default_selection, default_tool_state};
use crate::editor_mode::controls::EditorControlBuildContext;
use crate::editor_mode::dto::{
    EditorBounds2Dto, EditorCameraDto, EditorObjectEditCommandKindDto,
    EditorObjectPlacementKindDto, EditorSceneCanvasKindDto, EditorSceneObjectDto,
    EditorSceneSnapshotDto, EditorSceneSnapshotLayoutSourceDto, EditorToolDto, EditorTransform2Dto,
};

#[test]
fn move_tool_generates_selection_and_move_gizmo() {
    let object = test_object();
    let gizmos = gizmos_for_selection(
        &[object],
        &["player".to_owned()],
        EditorToolDto::Move,
        &EditorControlBuildContext::default(),
    );

    assert_eq!(gizmos.len(), 2);
    assert_eq!(
        gizmos[0].kind,
        crate::editor_mode::dto::EditorGizmoKindDto::SelectionBounds2D
    );
    assert_eq!(
        gizmos[1].kind,
        crate::editor_mode::dto::EditorGizmoKindDto::Move2D
    );
    assert!(gizmos[1].handles.iter().any(|handle| handle.id == "axis-x"));
    assert!(gizmos[1].handles.iter().any(|handle| handle.id == "axis-y"));
    assert!(
        gizmos[1]
            .handles
            .iter()
            .any(|handle| handle.id == "plane-xy")
    );
}

#[test]
fn rotate_tool_generates_rotation_ring() {
    let object = test_object();
    let gizmos = gizmos_for_selection(
        &[object],
        &["player".to_owned()],
        EditorToolDto::Rotate,
        &EditorControlBuildContext::default(),
    );

    assert_eq!(gizmos.len(), 2);
    assert_eq!(
        gizmos[1].kind,
        crate::editor_mode::dto::EditorGizmoKindDto::Rotate2D
    );
    assert!(
        gizmos[1]
            .handles
            .iter()
            .any(|handle| handle.id == "rotation-ring")
    );
}

#[test]
fn hit_test_uses_topmost_selectable_bounds() {
    let mut bottom = test_object();
    bottom.entity_id = "bottom".to_owned();
    let top = test_object();

    let hit = hit_test_snapshot_entity(
        &EditorSceneSnapshotDto {
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
            quality: Default::default(),
            objects: vec![bottom, top],
            ui_nodes: Vec::new(),
            diagnostics: Vec::new(),
            gizmos: Vec::new(),
            selection: default_selection(),
            tool_state: default_tool_state(),
        },
        100.0,
        120.0,
    );

    assert_eq!(hit.as_deref(), Some("player"));
}

fn test_object() -> EditorSceneObjectDto {
    EditorSceneObjectDto {
        entity_id: "player".to_owned(),
        name: "Player".to_owned(),
        visible: true,
        selectable: true,
        locked: false,
        movable: true,
        locked_reason: None,
        category: "sprite".to_owned(),
        component_types: vec!["Sprite".to_owned()],
        placement_kind: EditorObjectPlacementKindDto::Transform2,
        edit_command_kind: EditorObjectEditCommandKindDto::SetTransform2,
        transform_2: Some(EditorTransform2Dto {
            x: 100.0,
            y: 120.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            z_index: Some(1),
        }),
        transform_3: None,
        bounds_2: Some(EditorBounds2Dto {
            x: 84.0,
            y: 104.0,
            width: 32.0,
            height: 32.0,
        }),
        render_bounds_2: None,
        selection_bounds_2: None,
        prefab_instance: None,
    }
}
