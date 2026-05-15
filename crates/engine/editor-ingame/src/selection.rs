use amigo_editor_authoring::{AuthoringNode, AuthoringSceneGraph};

use crate::bounds::{bounds_for_node, topmost_candidate_at};
use crate::state::{EditorSelection, IngameEditorState, SelectionSource};

pub fn select_node_by_id(
    state: &IngameEditorState,
    graph: &AuthoringSceneGraph,
    node_id: String,
    source_path: Option<String>,
    yaml_pointer: Option<String>,
    source: SelectionSource,
) -> bool {
    let Some(node) = find_node_by_id(&graph.nodes, &node_id) else {
        state.set_status(format!("selection failed: node not found {node_id}"));
        return false;
    };
    let bounds = bounds_for_node(graph, &node_id);
    state.select_scene_node(EditorSelection {
        node_id,
        source,
        source_path,
        yaml_pointer,
        label: Some(selection_label(node)),
        logical_x: None,
        logical_y: None,
        logical_bounds: bounds,
    });
    true
}

pub fn select_viewport_target(
    state: &IngameEditorState,
    graph: &AuthoringSceneGraph,
    logical_x: f32,
    logical_y: f32,
) -> bool {
    let Some(candidate) = topmost_candidate_at(graph, logical_x, logical_y) else {
        state.clear_selection();
        state.set_status(format!(
            "viewport miss @ {logical_x:.1},{logical_y:.1}; picking uses logical fallback"
        ));
        return false;
    };
    state.select_scene_node(EditorSelection {
        node_id: candidate.node_id,
        source: SelectionSource::Viewport,
        source_path: candidate.source_path,
        yaml_pointer: candidate.yaml_pointer,
        label: candidate
            .entity_name
            .clone()
            .or(candidate.component_type.clone()),
        logical_x: Some(logical_x),
        logical_y: Some(logical_y),
        logical_bounds: Some(candidate.bounds),
    });
    true
}

fn selection_label(node: &AuthoringNode) -> String {
    node.label
        .strip_prefix("entity: ")
        .or_else(|| node.label.strip_prefix("component: "))
        .or_else(|| node.label.strip_prefix("layer: "))
        .unwrap_or(node.label.as_str())
        .to_owned()
}

fn find_node_by_id<'a>(nodes: &'a [AuthoringNode], node_id: &str) -> Option<&'a AuthoringNode> {
    for node in nodes {
        if node.id == node_id {
            return Some(node);
        }
        if let Some(found) = find_node_by_id(&node.children, node_id) {
            return Some(found);
        }
    }
    None
}
