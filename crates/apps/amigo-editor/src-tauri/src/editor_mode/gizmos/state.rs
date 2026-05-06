use crate::editor_mode::controls::EditorControlBuildContext;
use crate::editor_mode::dto::{
    EditorSceneSnapshotDto, EditorSelectionDto, EditorSnapSettingsDto, EditorToolDto,
    EditorToolSpaceDto, EditorToolStateDto, EditorUiNodeSelectionDto,
};

use super::builders::gizmos_for_selection;

pub fn default_selection() -> EditorSelectionDto {
    EditorSelectionDto {
        selected_entity_ids: Vec::new(),
        selected_ui_node: None,
    }
}

pub fn default_tool_state() -> EditorToolStateDto {
    EditorToolStateDto {
        active_tool: EditorToolDto::Select,
        space: EditorToolSpaceDto::World,
        snap: EditorSnapSettingsDto {
            enabled: true,
            grid_size: 1.0,
            angle_step_deg: 15.0,
            scale_step: 0.1,
        },
    }
}

pub fn enrich_snapshot_with_editor_state(
    snapshot: EditorSceneSnapshotDto,
    selected_entity_id: Option<String>,
    active_tool: EditorToolDto,
) -> EditorSceneSnapshotDto {
    enrich_snapshot_with_editor_control_state(
        snapshot,
        selected_entity_id,
        None,
        active_tool,
        EditorControlBuildContext::default(),
    )
}

pub fn enrich_snapshot_with_editor_control_state(
    mut snapshot: EditorSceneSnapshotDto,
    selected_entity_id: Option<String>,
    selected_ui_node: Option<EditorUiNodeSelectionDto>,
    active_tool: EditorToolDto,
    control_context: EditorControlBuildContext,
) -> EditorSceneSnapshotDto {
    let selected_entity_ids = selected_entity_id.into_iter().collect::<Vec<_>>();
    snapshot.selection = EditorSelectionDto {
        selected_entity_ids: selected_entity_ids.clone(),
        selected_ui_node,
    };
    snapshot.tool_state.active_tool = active_tool;
    snapshot.gizmos = gizmos_for_selection(
        &snapshot.objects,
        &selected_entity_ids,
        active_tool,
        &control_context,
    );

    if let Some(selection) = snapshot.selection.selected_ui_node.clone() {
        if let Some(gizmo) =
            super::builders::ui_node_selection_gizmo(&snapshot.ui_nodes, &selection)
        {
            snapshot.gizmos.push(gizmo);
        }
    }
    snapshot
}
