use amigo_editor_authoring::{AuthoringNode, AuthoringNodeKind, AuthoringSceneGraph};
use serde_yaml::Value;

use crate::layout::{GAME_VIEWPORT_LOGICAL_H, GAME_VIEWPORT_LOGICAL_W};
use crate::state::{EditorRect, IngameEditorState};

pub fn select_viewport_target(
    state: &IngameEditorState,
    graph: &AuthoringSceneGraph,
    logical_x: f32,
    logical_y: f32,
) -> bool {
    let Some(candidate) = topmost_candidate_at(graph, logical_x, logical_y) else {
        state.set_status(format!("viewport miss @ {logical_x:.1},{logical_y:.1}"));
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

#[derive(Debug, Clone)]
struct ViewportPickCandidate {
    node_id: String,
    source_path: Option<String>,
    yaml_pointer: Option<String>,
    entity_name: Option<String>,
    component_type: Option<String>,
    bounds: EditorRect,
    order: f32,
}

fn topmost_candidate_at(
    graph: &AuthoringSceneGraph,
    logical_x: f32,
    logical_y: f32,
) -> Option<ViewportPickCandidate> {
    let mut candidates = Vec::new();
    collect_pick_candidates(&graph.nodes, graph, &mut candidates);
    candidates
        .into_iter()
        .filter(|candidate| candidate.bounds.contains(logical_x, logical_y))
        .max_by(|left, right| {
            left.order
                .partial_cmp(&right.order)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn collect_pick_candidates(
    nodes: &[AuthoringNode],
    graph: &AuthoringSceneGraph,
    out: &mut Vec<ViewportPickCandidate>,
) {
    for node in nodes {
        if let Some(candidate) = pick_candidate_for_node(node, graph) {
            out.push(candidate);
        }
        collect_pick_candidates(&node.children, graph, out);
    }
}

fn pick_candidate_for_node(
    node: &AuthoringNode,
    graph: &AuthoringSceneGraph,
) -> Option<ViewportPickCandidate> {
    if node.kind != AuthoringNodeKind::Component {
        return None;
    }

    let component_type = node.semantic.component_type.as_deref()?;
    let bounds = match component_type {
        "LayeredImage2D" => layered_image_bounds(node, graph)?,
        "ParticleEmitter2D" => particle_emitter_bounds(node, graph)?,
        _ => return None,
    };

    Some(ViewportPickCandidate {
        node_id: node.id.clone(),
        source_path: Some(node.source_file.display().to_string()),
        yaml_pointer: Some(node.yaml_pointer.clone()),
        entity_name: node.semantic.owner_entity_name.clone(),
        component_type: node.semantic.component_type.clone(),
        bounds,
        order: render_order_hint(node),
    })
}

fn layered_image_bounds(node: &AuthoringNode, graph: &AuthoringSceneGraph) -> Option<EditorRect> {
    let (width, height) = vec2_field(&node.value, "size")
        .unwrap_or((GAME_VIEWPORT_LOGICAL_W, GAME_VIEWPORT_LOGICAL_H));
    let (entity_x, entity_y) = entity_translation(node, graph).unwrap_or((0.0, 0.0));
    Some(EditorRect {
        x: entity_x,
        y: entity_y,
        width,
        height,
    })
}

fn particle_emitter_bounds(
    node: &AuthoringNode,
    graph: &AuthoringSceneGraph,
) -> Option<EditorRect> {
    let (width, height) = mapping_get(&node.value, "spawn_area")
        .and_then(|spawn_area| vec2_field(spawn_area, "size"))
        .unwrap_or((128.0, 128.0));
    let (entity_x, entity_y) = entity_translation(node, graph).unwrap_or((0.0, 0.0));
    Some(EditorRect {
        x: entity_x - width * 0.5,
        y: entity_y - height * 0.5,
        width,
        height,
    })
}

fn render_order_hint(node: &AuthoringNode) -> f32 {
    number_field(&node.value, "z_index").unwrap_or(0.0)
}

fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

fn number_field(value: &Value, key: &str) -> Option<f32> {
    mapping_get(value, key)?.as_f64().map(|value| value as f32)
}

fn vec2_field(value: &Value, key: &str) -> Option<(f32, f32)> {
    let value = mapping_get(value, key)?;
    Some((number_field(value, "x")?, number_field(value, "y")?))
}

fn entity_translation(node: &AuthoringNode, graph: &AuthoringSceneGraph) -> Option<(f32, f32)> {
    let owner = node.semantic.owner_entity_name.as_deref()?;
    let entity = find_entity_node_by_owner_name(&graph.nodes, owner)?;
    let transform = mapping_get(&entity.value, "transform2")?;
    let translation = mapping_get(transform, "translation")?;
    Some((
        number_field(translation, "x")?,
        number_field(translation, "y")?,
    ))
}

fn find_entity_node_by_owner_name<'a>(
    nodes: &'a [AuthoringNode],
    owner: &str,
) -> Option<&'a AuthoringNode> {
    for node in nodes {
        if node.kind == AuthoringNodeKind::Entity {
            let name = mapping_get(&node.value, "name").and_then(Value::as_str);
            let id = mapping_get(&node.value, "id").and_then(Value::as_str);
            if name == Some(owner) || id == Some(owner) {
                return Some(node);
            }
        }
        if let Some(found) = find_entity_node_by_owner_name(&node.children, owner) {
            return Some(found);
        }
    }
    None
}
