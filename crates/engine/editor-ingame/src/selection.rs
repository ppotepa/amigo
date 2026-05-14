use amigo_editor_authoring::AuthoringSceneGraph;

use crate::bounds::topmost_candidate_at;
use crate::state::IngameEditorState;

pub fn select_viewport_target(
    state: &IngameEditorState,
    graph: &AuthoringSceneGraph,
    logical_x: f32,
    logical_y: f32,
) -> bool {
    let Some(candidate) = topmost_candidate_at(graph, logical_x, logical_y) else {
        state.set_status(format!(
            "viewport miss @ {logical_x:.1},{logical_y:.1}; picking uses logical fallback"
        ));
        return false;
    };

    state.select_viewport_node(
        candidate.node_id,
        candidate.source_path,
        candidate.yaml_pointer,
        candidate.entity_name,
        candidate.component_type,
        logical_x,
        logical_y,
        Some(candidate.bounds),
    );
    true
}
