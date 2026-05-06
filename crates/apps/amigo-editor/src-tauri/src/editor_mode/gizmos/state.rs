use crate::editor_mode::controls::EditorControlBuildContext;
use crate::editor_mode::dto::{
    EditorSceneSnapshotDto, EditorSelectionDto, EditorSnapSettingsDto, EditorToolDto,
    EditorToolSpaceDto, EditorToolStateDto,
};

use super::builders::gizmos_for_selection;

pub fn default_selection() -> EditorSelectionDto {
    EditorSelectionDto {
        selected_entity_ids: Vec::new(),
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
        active_tool,
        EditorControlBuildContext::default(),
    )
}

pub fn enrich_snapshot_with_editor_control_state(
    mut snapshot: EditorSceneSnapshotDto,
    selected_entity_id: Option<String>,
    active_tool: EditorToolDto,
    control_context: EditorControlBuildContext,
) -> EditorSceneSnapshotDto {
    let selected_entity_ids = selected_entity_id.into_iter().collect::<Vec<_>>();
    snapshot.selection = EditorSelectionDto {
        selected_entity_ids: selected_entity_ids.clone(),
    };
    snapshot.tool_state.active_tool = active_tool;
    snapshot.gizmos = gizmos_for_selection(
        &snapshot.objects,
        &selected_entity_ids,
        active_tool,
        &control_context,
    );
    snapshot
}
