use std::cmp::Ordering;

use serde_yaml::Value;

use crate::{AuthoringNode, AuthoringNodeKind, AuthoringSceneGraph};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoringTreeMode {
    SceneObjects,
    RenderStack,
    RawYaml,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringTreeTag {
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoringTreeIcon {
    Mod,
    Use,
    Scene,
    Visual2d,
    Entity,
    Component,
    Image,
    Particle,
    Text,
    Camera,
    Ui,
    DrawLayer,
    PostFx,
    Light,
    Route,
    Prefab,
    Override,
    Mapping,
    Sequence,
    Scalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringTreeRow {
    pub node_id: String,
    pub source_path: String,
    pub yaml_pointer: String,
    pub depth: usize,
    pub icon: AuthoringTreeIcon,
    pub label: String,
    pub tags: Vec<AuthoringTreeTag>,
    pub selectable: bool,
    pub has_children: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeProjection {
    pub rows: Vec<AuthoringTreeRow>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderStackProjection {
    pub layers: Vec<RenderStackLayerRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderStackLayerRow {
    pub node_id: String,
    pub source_path: String,
    pub yaml_pointer: String,
    pub id: String,
    pub order: f32,
    pub visible: bool,
    pub opacity: f32,
    pub entities: Vec<RenderStackEntityRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderStackEntityRow {
    pub node_id: String,
    pub source_path: String,
    pub yaml_pointer: String,
    pub icon: AuthoringTreeIcon,
    pub label: String,
    pub tags: Vec<AuthoringTreeTag>,
}

pub type SceneObjectsProjection = TreeProjection;
pub type RawYamlProjection = TreeProjection;

pub fn scene_objects_projection(graph: &AuthoringSceneGraph) -> SceneObjectsProjection {
    tree_projection(graph, AuthoringTreeMode::SceneObjects)
}

pub fn raw_yaml_projection(graph: &AuthoringSceneGraph) -> RawYamlProjection {
    tree_projection(graph, AuthoringTreeMode::RawYaml)
}

pub fn render_stack_tree_projection(graph: &AuthoringSceneGraph) -> TreeProjection {
    let stack = render_stack_projection(graph);
    let mut rows = Vec::new();
    for layer in stack.layers {
        rows.push(AuthoringTreeRow {
            node_id: layer.node_id,
            source_path: layer.source_path,
            yaml_pointer: layer.yaml_pointer,
            depth: 0,
            icon: AuthoringTreeIcon::DrawLayer,
            label: format!(
                "{}  order {:.0}  opacity {:.0}%",
                layer.id,
                layer.order,
                layer.opacity * 100.0
            ),
            tags: if layer.visible {
                vec![tag("Live")]
            } else {
                vec![tag("Hidden")]
            },
            selectable: true,
            has_children: !layer.entities.is_empty(),
        });
        for entity in layer.entities {
            rows.push(AuthoringTreeRow {
                node_id: entity.node_id,
                source_path: entity.source_path,
                yaml_pointer: entity.yaml_pointer,
                depth: 1,
                icon: entity.icon,
                label: entity.label,
                tags: minimal_status_tags(entity.tags),
                selectable: true,
                has_children: false,
            });
        }
    }
    TreeProjection { rows }
}

pub fn tree_projection(graph: &AuthoringSceneGraph, mode: AuthoringTreeMode) -> TreeProjection {
    match mode {
        AuthoringTreeMode::RenderStack => render_stack_tree_projection(graph),
        _ => {
            let mut rows = Vec::new();
            for root in &graph.nodes {
                collect_tree_rows(root, 0, mode, &mut rows);
            }
            TreeProjection { rows }
        }
    }
}

pub fn render_stack_projection(graph: &AuthoringSceneGraph) -> RenderStackProjection {
    let mut layers = Vec::new();
    for root in &graph.nodes {
        collect_render_layers(root, &mut layers);
    }
    for root in &graph.nodes {
        collect_entities_for_layers(root, &mut layers);
    }
    layers.sort_by(|a, b| a.order.partial_cmp(&b.order).unwrap_or(Ordering::Equal));
    RenderStackProjection { layers }
}

fn collect_tree_rows(
    node: &AuthoringNode,
    depth: usize,
    mode: AuthoringTreeMode,
    rows: &mut Vec<AuthoringTreeRow>,
) {
    let render_this_node = match mode {
        AuthoringTreeMode::RawYaml => true,
        AuthoringTreeMode::SceneObjects => scene_objects_renders_node(node),
        AuthoringTreeMode::RenderStack => false,
    };
    let descend = match mode {
        AuthoringTreeMode::RawYaml => true,
        AuthoringTreeMode::SceneObjects => scene_objects_descends_into(node),
        AuthoringTreeMode::RenderStack => false,
    };

    if render_this_node {
        rows.push(AuthoringTreeRow {
            node_id: node.id.clone(),
            source_path: node.source_file.display().to_string(),
            yaml_pointer: node.yaml_pointer.clone(),
            depth,
            icon: icon_for_node(node),
            label: clean_label(node),
            tags: tags_for_node(node),
            selectable: true,
            has_children: node.children.iter().any(|child| match mode {
                AuthoringTreeMode::RawYaml => true,
                AuthoringTreeMode::SceneObjects => scene_objects_renders_node(child),
                AuthoringTreeMode::RenderStack => false,
            }),
        });
    }

    if descend {
        let child_depth = if render_this_node { depth + 1 } else { depth };
        for child in &node.children {
            collect_tree_rows(child, child_depth, mode, rows);
        }
    }
}

fn scene_objects_renders_node(node: &AuthoringNode) -> bool {
    matches!(
        node.kind,
        AuthoringNodeKind::Use
            | AuthoringNodeKind::Scene
            | AuthoringNodeKind::Entities
            | AuthoringNodeKind::Entity
            | AuthoringNodeKind::Component
            | AuthoringNodeKind::PostFx
            | AuthoringNodeKind::PostFxItem
            | AuthoringNodeKind::LightGroups
            | AuthoringNodeKind::LightGroup
            | AuthoringNodeKind::LightRoutes
            | AuthoringNodeKind::LightRoute
            | AuthoringNodeKind::PrefabRef
            | AuthoringNodeKind::Use
    )
}

fn scene_objects_descends_into(node: &AuthoringNode) -> bool {
    !matches!(
        node.kind,
        AuthoringNodeKind::Scalar
            | AuthoringNodeKind::RenderLayers
            | AuthoringNodeKind::RenderLayer
    )
}

fn clean_label(node: &AuthoringNode) -> String {
    let label = node.label.as_str();
    label
        .strip_prefix("object: ")
        .or_else(|| label.strip_prefix("entity: "))
        .or_else(|| label.strip_prefix("component: "))
        .or_else(|| label.strip_prefix("draw layer: "))
        .or_else(|| label.strip_prefix("layer: "))
        .or_else(|| label.strip_prefix("route: "))
        .or_else(|| label.strip_prefix("light: "))
        .unwrap_or(label)
        .to_owned()
}

fn icon_for_node(node: &AuthoringNode) -> AuthoringTreeIcon {
    match node.kind {
        AuthoringNodeKind::File => AuthoringTreeIcon::Mod,
        AuthoringNodeKind::Use => AuthoringTreeIcon::Use,
        AuthoringNodeKind::Scene => AuthoringTreeIcon::Scene,
        AuthoringNodeKind::Visual2d => AuthoringTreeIcon::Visual2d,
        AuthoringNodeKind::Entities => AuthoringTreeIcon::Entity,
        AuthoringNodeKind::Entity => AuthoringTreeIcon::Entity,
        AuthoringNodeKind::Components => AuthoringTreeIcon::Component,
        AuthoringNodeKind::Component => component_icon(node),
        AuthoringNodeKind::RenderLayers => AuthoringTreeIcon::DrawLayer,
        AuthoringNodeKind::RenderLayer => AuthoringTreeIcon::DrawLayer,
        AuthoringNodeKind::PostFx => AuthoringTreeIcon::PostFx,
        AuthoringNodeKind::PostFxItem => AuthoringTreeIcon::PostFx,
        AuthoringNodeKind::LightGroups => AuthoringTreeIcon::Light,
        AuthoringNodeKind::LightGroup => AuthoringTreeIcon::Light,
        AuthoringNodeKind::LightRoutes => AuthoringTreeIcon::Route,
        AuthoringNodeKind::LightRoute => AuthoringTreeIcon::Route,
        AuthoringNodeKind::PrefabRef => AuthoringTreeIcon::Prefab,
        AuthoringNodeKind::PrefabOverrides => AuthoringTreeIcon::Override,
        AuthoringNodeKind::Mapping => AuthoringTreeIcon::Mapping,
        AuthoringNodeKind::Sequence => AuthoringTreeIcon::Sequence,
        AuthoringNodeKind::Scalar => AuthoringTreeIcon::Scalar,
    }
}

fn component_icon(node: &AuthoringNode) -> AuthoringTreeIcon {
    match node.semantic.component_type.as_deref() {
        Some("LayeredImage2D") | Some("Sprite2D") => AuthoringTreeIcon::Image,
        Some("ParticleEmitter2D") => AuthoringTreeIcon::Particle,
        Some("BeaconLight2D") | Some("GlobalLight2D") => AuthoringTreeIcon::Light,
        Some("Text2D") => AuthoringTreeIcon::Text,
        Some("Camera2D") => AuthoringTreeIcon::Camera,
        Some("UiDocument") => AuthoringTreeIcon::Ui,
        _ => AuthoringTreeIcon::Component,
    }
}

fn tags_for_node(node: &AuthoringNode) -> Vec<AuthoringTreeTag> {
    match node.kind {
        AuthoringNodeKind::PostFxItem => vec![tag("Readonly")],
        AuthoringNodeKind::LightGroup | AuthoringNodeKind::LightRoute => vec![tag("Readonly")],
        AuthoringNodeKind::PrefabRef | AuthoringNodeKind::Use => vec![tag("Readonly")],
        AuthoringNodeKind::Component => component_status_tag(node),
        _ => Vec::new(),
    }
}

fn component_status_tag(node: &AuthoringNode) -> Vec<AuthoringTreeTag> {
    match node.semantic.component_type.as_deref() {
        Some("LayeredImage2D") | Some("ParticleEmitter2D") | Some("BeaconLight2D") => {
            vec![tag("Live")]
        }
        Some(_) if node.editable => Vec::new(),
        _ => vec![tag("Readonly")],
    }
}

fn minimal_status_tags(tags: Vec<AuthoringTreeTag>) -> Vec<AuthoringTreeTag> {
    for wanted in ["Live", "Mock", "Readonly", "Unsupported", "Hidden"] {
        if tags.iter().any(|tag| tag.label == wanted) {
            return vec![tag(wanted)];
        }
    }
    Vec::new()
}

fn tag(label: impl Into<String>) -> AuthoringTreeTag {
    AuthoringTreeTag {
        label: label.into(),
    }
}

fn collect_render_layers(node: &AuthoringNode, out: &mut Vec<RenderStackLayerRow>) {
    if matches!(node.kind, AuthoringNodeKind::RenderLayer) {
        let id = yaml_string(&node.value, "id").unwrap_or_else(|| node.label.clone());
        out.push(RenderStackLayerRow {
            node_id: node.id.clone(),
            source_path: node.source_file.display().to_string(),
            yaml_pointer: node.yaml_pointer.clone(),
            id,
            order: yaml_number(&node.value, "order").unwrap_or(0.0),
            visible: yaml_bool(&node.value, "visible").unwrap_or(true),
            opacity: yaml_number(&node.value, "opacity").unwrap_or(1.0),
            entities: Vec::new(),
        });
    }
    for child in &node.children {
        collect_render_layers(child, out);
    }
}

fn collect_entities_for_layers(node: &AuthoringNode, layers: &mut [RenderStackLayerRow]) {
    if matches!(node.kind, AuthoringNodeKind::Entity) {
        let entity_label = yaml_string(&node.value, "name")
            .or_else(|| yaml_string(&node.value, "id"))
            .unwrap_or_else(|| clean_label(node));
        collect_entity_components_for_layers(node, node, entity_label, layers);
    }
    for child in &node.children {
        collect_entities_for_layers(child, layers);
    }
}

fn collect_entity_components_for_layers(
    entity_node: &AuthoringNode,
    node: &AuthoringNode,
    entity_label: String,
    layers: &mut [RenderStackLayerRow],
) {
    if matches!(node.kind, AuthoringNodeKind::Component) {
        if let Some(layer_id) = yaml_string(&node.value, "render_layer") {
            if let Some(layer) = layers.iter_mut().find(|layer| layer.id == layer_id) {
                layer.entities.push(RenderStackEntityRow {
                    node_id: entity_node.id.clone(),
                    source_path: entity_node.source_file.display().to_string(),
                    yaml_pointer: entity_node.yaml_pointer.clone(),
                    icon: component_icon(node),
                    label: entity_label.clone(),
                    tags: if node.editable {
                        vec![tag("Live")]
                    } else {
                        vec![tag("Readonly")]
                    },
                });
            }
        }
    }
    for child in &node.children {
        collect_entity_components_for_layers(entity_node, child, entity_label.clone(), layers);
    }
}

fn yaml_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

fn yaml_string(value: &Value, key: &str) -> Option<String> {
    yaml_get(value, key)?.as_str().map(str::to_owned)
}

fn yaml_number(value: &Value, key: &str) -> Option<f32> {
    yaml_get(value, key)?.as_f64().map(|v| v as f32)
}

fn yaml_bool(value: &Value, key: &str) -> Option<bool> {
    yaml_get(value, key)?.as_bool()
}
