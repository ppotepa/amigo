use amigo_editor_authoring::{AuthoringNode, AuthoringNodeKind, AuthoringSceneGraph};
use amigo_scene::{BoundsPolicy, ComponentRegistry, default_component_registry};
use serde_yaml::Value;

use crate::state::EditorRect;

pub trait BoundsProvider {
    fn bounds_for_component(
        &self,
        node: &AuthoringNode,
        graph: &AuthoringSceneGraph,
        component_type: &str,
    ) -> Option<EditorRect>;
}

pub struct DescriptorBoundsProvider<'a> {
    pub registry: &'a ComponentRegistry,
}

#[derive(Debug, Clone)]
pub struct PickCandidate {
    pub node_id: String,
    pub source_path: Option<String>,
    pub yaml_pointer: Option<String>,
    pub entity_name: Option<String>,
    pub component_type: Option<String>,
    pub bounds: EditorRect,
    pub order: f32,
}

#[allow(dead_code)]
pub fn topmost_candidate_at(
    graph: &AuthoringSceneGraph,
    logical_x: f32,
    logical_y: f32,
) -> Option<PickCandidate> {
    let registry = default_component_registry();
    topmost_candidate_at_with_registry(graph, &registry, logical_x, logical_y)
}

pub fn topmost_candidate_at_with_registry(
    graph: &AuthoringSceneGraph,
    registry: &ComponentRegistry,
    logical_x: f32,
    logical_y: f32,
) -> Option<PickCandidate> {
    let mut candidates = Vec::new();
    let provider = DescriptorBoundsProvider { registry };
    collect_pick_candidates(graph, &provider, &mut candidates);
    candidates
        .into_iter()
        .filter(|candidate| candidate.bounds.contains(logical_x, logical_y))
        .max_by(|left, right| {
            left.order
                .partial_cmp(&right.order)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn collect_pick_candidates(
    graph: &AuthoringSceneGraph,
    provider: &impl BoundsProvider,
    out: &mut Vec<PickCandidate>,
) {
    collect_pick_candidates_in_nodes(&graph.nodes, graph, provider, out);
}

#[allow(dead_code)]
pub fn bounds_for_node(graph: &AuthoringSceneGraph, node_id: &str) -> Option<EditorRect> {
    let registry = default_component_registry();
    bounds_for_node_with_registry(graph, &registry, node_id)
}

pub fn bounds_for_node_with_registry(
    graph: &AuthoringSceneGraph,
    registry: &ComponentRegistry,
    node_id: &str,
) -> Option<EditorRect> {
    let provider = DescriptorBoundsProvider { registry };
    find_node_by_id(&graph.nodes, node_id)
        .and_then(|node| bounds_for_authoring_node(node, graph, &provider))
}
#[allow(dead_code)]
pub fn pick_candidate_for_node_id(
    graph: &AuthoringSceneGraph,
    node_id: &str,
) -> Option<PickCandidate> {
    let registry = default_component_registry();
    let provider = DescriptorBoundsProvider {
        registry: &registry,
    };
    let node = find_node_by_id(&graph.nodes, node_id)?;
    pick_candidate_for_node(node, graph, &provider)
}

fn collect_pick_candidates_in_nodes(
    nodes: &[AuthoringNode],
    graph: &AuthoringSceneGraph,
    provider: &impl BoundsProvider,
    out: &mut Vec<PickCandidate>,
) {
    for node in nodes {
        if let Some(candidate) = pick_candidate_for_node(node, graph, provider) {
            out.push(candidate);
        }
        collect_pick_candidates_in_nodes(&node.children, graph, provider, out);
    }
}

fn pick_candidate_for_node(
    node: &AuthoringNode,
    graph: &AuthoringSceneGraph,
    provider: &impl BoundsProvider,
) -> Option<PickCandidate> {
    if node.kind != AuthoringNodeKind::Component {
        return None;
    }

    let component_type = node.semantic.component_type.as_deref()?;
    let bounds = provider.bounds_for_component(node, graph, component_type)?;

    Some(PickCandidate {
        node_id: node.id.clone(),
        source_path: Some(node.source_file.display().to_string()),
        yaml_pointer: Some(node.yaml_pointer.clone()),
        entity_name: node.semantic.owner_entity_name.clone(),
        component_type: node.semantic.component_type.clone(),
        bounds,
        order: render_order_hint(node),
    })
}

fn bounds_for_authoring_node(
    node: &AuthoringNode,
    graph: &AuthoringSceneGraph,
    provider: &impl BoundsProvider,
) -> Option<EditorRect> {
    match node.kind {
        AuthoringNodeKind::Component => {
            let component_type = node.semantic.component_type.as_deref()?;
            provider.bounds_for_component(node, graph, component_type)
        }
        AuthoringNodeKind::Entity => node
            .children
            .iter()
            .filter_map(|child| bounds_for_authoring_node(child, graph, provider))
            .reduce(union_rects),
        _ => None,
    }
}

impl BoundsProvider for DescriptorBoundsProvider<'_> {
    fn bounds_for_component(
        &self,
        node: &AuthoringNode,
        graph: &AuthoringSceneGraph,
        component_type: &str,
    ) -> Option<EditorRect> {
        let descriptor = self.registry.descriptor_by_type_name(component_type)?;

        match descriptor.bounds_policy {
            BoundsPolicy::ComponentBounds2D { field } => component_bounds_2d(node, graph, field),
            BoundsPolicy::SpawnArea2D {
                field,
                size_field,
                default_width,
                default_height,
            } => particle_emitter_bounds(
                node,
                graph,
                field,
                size_field,
                default_width,
                default_height,
            ),
            BoundsPolicy::EntityTransformPoint => entity_point_bounds(node, graph),
            _ => None,
        }
    }
}

fn component_bounds_2d(
    node: &AuthoringNode,
    graph: &AuthoringSceneGraph,
    field: &str,
) -> Option<EditorRect> {
    let (width, height) = vec2_field(&node.value, field)?;
    let (entity_x, entity_y) = entity_translation(node, graph).unwrap_or((0.0, 0.0));
    let (offset_x, offset_y) = vec2_field(&node.value, "local_offset").unwrap_or((0.0, 0.0));
    Some(EditorRect {
        x: entity_x + offset_x,
        y: entity_y + offset_y,
        width,
        height,
    })
}

fn particle_emitter_bounds(
    node: &AuthoringNode,
    graph: &AuthoringSceneGraph,
    field: &str,
    size_field: &str,
    default_width: u32,
    default_height: u32,
) -> Option<EditorRect> {
    let (width, height) = mapping_get(&node.value, field)
        .and_then(|spawn_area| vec2_field(spawn_area, size_field))
        .unwrap_or((default_width as f32, default_height as f32));
    let (entity_x, entity_y) = entity_translation(node, graph).unwrap_or((0.0, 0.0));
    Some(EditorRect {
        x: entity_x - width * 0.5,
        y: entity_y - height * 0.5,
        width,
        height,
    })
}

fn entity_point_bounds(node: &AuthoringNode, graph: &AuthoringSceneGraph) -> Option<EditorRect> {
    let (entity_x, entity_y) = entity_translation(node, graph).unwrap_or((0.0, 0.0));
    Some(EditorRect {
        x: entity_x - 8.0,
        y: entity_y - 8.0,
        width: 16.0,
        height: 16.0,
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

fn union_rects(a: EditorRect, b: EditorRect) -> EditorRect {
    let min_x = a.x.min(b.x);
    let min_y = a.y.min(b.y);
    let max_x = (a.x + a.width).max(b.x + b.width);
    let max_y = (a.y + a.height).max(b.y + b.height);
    EditorRect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(0.0),
        height: (max_y - min_y).max(0.0),
    }
}
