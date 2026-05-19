use amigo_editor_api::{InspectRequest, InspectRequestService, InspectSubject};
use amigo_editor_authoring::{
    AuthoringNode, AuthoringNodeKind, AuthoringSceneGraph, AuthoringSceneGraphService,
    build_property_panel_for_node,
};
use amigo_runtime::Runtime;

use crate::state::{
    EditorSelection, IngameEditorState, InspectTarget, InspectTargetKind, SelectionSource,
};

#[derive(Debug)]
pub(crate) enum InspectResolveError {
    NoGraph,
    NoSelection,
    UnknownTarget(String),
    AmbiguousTarget {
        selector: String,
        candidates: Vec<String>,
    },
    NotInspectable {
        label: String,
        reason: String,
    },
}

pub(crate) struct ResolvedInspectTarget {
    pub target: InspectTarget,
    pub selection: EditorSelection,
}

pub(crate) fn process_pending_inspect_requests(
    runtime: &Runtime,
    state: &IngameEditorState,
) -> Result<Option<InspectTarget>, InspectResolveError> {
    let Some(queue) = runtime.resolve::<InspectRequestService>() else {
        return Ok(None);
    };
    let Some(request) = queue.take_latest() else {
        return Ok(None);
    };
    let resolved = resolve_inspect_request(runtime, state, &request)?;
    let target = resolved.target.clone();
    state.open_inspector_dock(resolved.target, resolved.selection);
    Ok(Some(target))
}

pub(crate) fn resolve_inspect_request(
    runtime: &Runtime,
    state: &IngameEditorState,
    request: &InspectRequest,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    let Some(graph_service) = runtime.resolve::<AuthoringSceneGraphService>() else {
        return Err(InspectResolveError::NoGraph);
    };
    let Ok(graph) = graph_service.graph_for_current_scene(runtime) else {
        return Err(InspectResolveError::NoGraph);
    };
    resolve_subject(&graph, state, &request.subject, request.expression.clone())
}

pub(crate) fn resolve_subject(
    graph: &AuthoringSceneGraph,
    state: &IngameEditorState,
    subject: &InspectSubject,
    expression: Option<String>,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    match subject {
        InspectSubject::Selected => resolve_selected(graph, state, expression),
        InspectSubject::AuthoringNode { node_id } => {
            let Some(node) = graph.find_node(node_id) else {
                return Err(InspectResolveError::UnknownTarget(node_id.clone()));
            };
            resolved_from_node(
                node,
                InspectTargetKind::AuthoringNode,
                node.label.clone(),
                node_id.clone(),
                expression,
            )
        }
        InspectSubject::Entity { name } => resolve_entity(graph, name, expression),
        InspectSubject::PostFxFrameItem { index, label } => {
            resolve_postfx_frame_item(graph, *index, label.clone(), expression)
        }
        InspectSubject::RenderLayer { id } => resolve_render_layer(graph, id, expression),
    }
}

fn all_nodes(graph: &AuthoringSceneGraph) -> Vec<&AuthoringNode> {
    fn walk<'a>(node: &'a AuthoringNode, out: &mut Vec<&'a AuthoringNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    for root in &graph.nodes {
        walk(root, &mut out);
    }
    out
}

fn node_has_editable_properties(node: &AuthoringNode) -> bool {
    let panel = build_property_panel_for_node(node);
    panel.groups.iter().any(|group| {
        group.properties.iter().any(|property| {
            property.binding.is_some()
                && !property.read_only
                && !matches!(
                    property.editor,
                    amigo_editor_authoring::AuthoringPropertyEditor::ReadOnly
                )
        })
    })
}

fn first_inspectable_descendant(node: &AuthoringNode) -> Option<&AuthoringNode> {
    if node_has_editable_properties(node) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = first_inspectable_descendant(child) {
            return Some(found);
        }
    }
    None
}

fn resolve_selected(
    graph: &AuthoringSceneGraph,
    state: &IngameEditorState,
    expression: Option<String>,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    let snapshot = state.snapshot();
    let Some(selection) = snapshot.selection.clone() else {
        return Err(InspectResolveError::NoSelection);
    };
    let Some(node) = graph.find_node(&selection.node_id) else {
        return Err(InspectResolveError::UnknownTarget(
            selection.node_id.clone(),
        ));
    };
    if !node_has_editable_properties(node) {
        return Err(InspectResolveError::NotInspectable {
            label: selection
                .label
                .clone()
                .unwrap_or_else(|| selection.node_id.clone()),
            reason: "selected node has no editable properties".to_owned(),
        });
    }
    Ok(ResolvedInspectTarget {
        target: InspectTarget {
            kind: InspectTargetKind::Selected,
            label: selection
                .label
                .clone()
                .unwrap_or_else(|| selection.node_id.clone()),
            subject: "selected".to_owned(),
            node_id: selection.node_id.clone(),
            expression,
        },
        selection,
    })
}

fn resolve_entity(
    graph: &AuthoringSceneGraph,
    name: &str,
    expression: Option<String>,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    let candidates: Vec<&AuthoringNode> = all_nodes(graph)
        .into_iter()
        .filter(|node| matches!(node.kind, AuthoringNodeKind::Entity))
        .filter(|node| {
            node.semantic.scene_object_id.as_deref() == Some(name)
                || node.semantic.owner_entity_name.as_deref() == Some(name)
                || node.id == name
                || node.label == name
                || node
                    .label
                    .strip_prefix("entity: ")
                    .unwrap_or(node.label.as_str())
                    == name
        })
        .collect();

    if candidates.len() > 1 {
        return Err(InspectResolveError::AmbiguousTarget {
            selector: name.to_owned(),
            candidates: candidates.iter().map(|node| node.label.clone()).collect(),
        });
    }

    let Some(entity_node) = candidates.first().copied() else {
        return Err(InspectResolveError::UnknownTarget(format!(
            "entity({name})"
        )));
    };
    let Some(inspect_node) = first_inspectable_descendant(entity_node) else {
        return Err(InspectResolveError::NotInspectable {
            label: entity_node.label.clone(),
            reason: "entity has no editable inspector properties".to_owned(),
        });
    };
    resolved_from_node(
        inspect_node,
        InspectTargetKind::Entity,
        format!("Entity: {name}"),
        format!("entity:{name}"),
        expression,
    )
}

fn resolve_postfx_frame_item(
    graph: &AuthoringSceneGraph,
    index: usize,
    label: Option<String>,
    expression: Option<String>,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    let nodes: Vec<&AuthoringNode> = all_nodes(graph)
        .into_iter()
        .filter(|node| matches!(node.kind, AuthoringNodeKind::PostFxItem))
        .collect();
    let Some(node) = nodes.get(index).copied() else {
        return Err(InspectResolveError::UnknownTarget(format!(
            "postfx.item({index})"
        )));
    };
    if !node_has_editable_properties(node) {
        return Err(InspectResolveError::NotInspectable {
            label: node.label.clone(),
            reason: "post-fx item has no editable live bindings".to_owned(),
        });
    }
    resolved_from_node(
        node,
        InspectTargetKind::PostFxFrameItem,
        label.unwrap_or_else(|| node.label.clone()),
        format!("postfx:{index}"),
        expression,
    )
}

fn resolve_render_layer(
    graph: &AuthoringSceneGraph,
    id: &str,
    expression: Option<String>,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    let candidates: Vec<&AuthoringNode> = all_nodes(graph)
        .into_iter()
        .filter(|node| matches!(node.kind, AuthoringNodeKind::RenderLayer))
        .filter(|node| {
            node.semantic.render_layer_id.as_deref() == Some(id)
                || node.id == id
                || node.label == id
        })
        .collect();
    if candidates.len() > 1 {
        return Err(InspectResolveError::AmbiguousTarget {
            selector: id.to_owned(),
            candidates: candidates.iter().map(|node| node.label.clone()).collect(),
        });
    }
    let Some(node) = candidates.first().copied() else {
        return Err(InspectResolveError::UnknownTarget(format!(
            "render.layer({id})"
        )));
    };
    if !node_has_editable_properties(node) {
        return Err(InspectResolveError::NotInspectable {
            label: node.label.clone(),
            reason: "render layer has no editable properties".to_owned(),
        });
    }
    resolved_from_node(
        node,
        InspectTargetKind::RenderLayer,
        format!("Render Layer: {id}"),
        format!("layer:{id}"),
        expression,
    )
}

fn resolved_from_node(
    node: &AuthoringNode,
    kind: InspectTargetKind,
    label: String,
    subject: String,
    expression: Option<String>,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    if !node_has_editable_properties(node) {
        return Err(InspectResolveError::NotInspectable {
            label: node.label.clone(),
            reason: "node has no editable inspector properties".to_owned(),
        });
    }
    let selection = EditorSelection {
        node_id: node.id.clone(),
        source: SelectionSource::Command,
        source_path: Some(node.source_file.display().to_string()),
        yaml_pointer: Some(node.yaml_pointer.clone()),
        label: Some(node.label.clone()),
        logical_x: None,
        logical_y: None,
        logical_bounds: None,
    };
    Ok(ResolvedInspectTarget {
        target: InspectTarget {
            kind,
            label,
            subject,
            node_id: node.id.clone(),
            expression,
        },
        selection,
    })
}

pub(crate) fn resolve_text_inspect_selector(
    runtime: &Runtime,
    state: &IngameEditorState,
    selector: &str,
) -> Result<ResolvedInspectTarget, InspectResolveError> {
    let Some(graph_service) = runtime.resolve::<AuthoringSceneGraphService>() else {
        return Err(InspectResolveError::NoGraph);
    };
    let Ok(graph) = graph_service.graph_for_current_scene(runtime) else {
        return Err(InspectResolveError::NoGraph);
    };

    if selector == "selected" {
        return resolve_subject(
            &graph,
            state,
            &InspectSubject::Selected,
            Some(selector.to_owned()),
        );
    }
    if let Some(name) = selector.strip_prefix("entity:") {
        return resolve_subject(
            &graph,
            state,
            &InspectSubject::Entity {
                name: name.to_owned(),
            },
            Some(selector.to_owned()),
        );
    }
    if let Some(id) = selector
        .strip_prefix("layer:")
        .or_else(|| selector.strip_prefix("render-layer:"))
    {
        return resolve_subject(
            &graph,
            state,
            &InspectSubject::RenderLayer { id: id.to_owned() },
            Some(selector.to_owned()),
        );
    }
    if let Some(node_id) = selector.strip_prefix("node:") {
        return resolve_subject(
            &graph,
            state,
            &InspectSubject::AuthoringNode {
                node_id: node_id.to_owned(),
            },
            Some(selector.to_owned()),
        );
    }
    if let Some(raw) = selector.strip_prefix("postfx:") {
        if let Ok(index) = raw.parse::<usize>() {
            return resolve_subject(
                &graph,
                state,
                &InspectSubject::PostFxFrameItem { index, label: None },
                Some(selector.to_owned()),
            );
        }
    }
    Err(InspectResolveError::UnknownTarget(selector.to_owned()))
}

pub(crate) fn format_inspect_error(error: InspectResolveError) -> String {
    match error {
        InspectResolveError::NoGraph => "inspect: authoring scene graph unavailable".to_owned(),
        InspectResolveError::NoSelection => "inspect: no current selection".to_owned(),
        InspectResolveError::UnknownTarget(target) => format!("inspect: unknown target `{target}`"),
        InspectResolveError::AmbiguousTarget {
            selector,
            candidates,
        } => format!(
            "inspect: ambiguous target `{}`; candidates: {}",
            selector,
            candidates.join(", ")
        ),
        InspectResolveError::NotInspectable { label, reason } => {
            format!("inspect: `{label}` is not inspectable: {reason}")
        }
    }
}
