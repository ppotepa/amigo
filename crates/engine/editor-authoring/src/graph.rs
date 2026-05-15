use std::path::PathBuf;

use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoringNodeKind {
    File,
    Use,
    Scene,
    Visual2d,
    RenderLayers,
    RenderLayer,
    LightGroups,
    LightGroup,
    LightRoutes,
    LightRoute,
    PostFx,
    PostFxItem,
    Entities,
    Entity,
    Components,
    Component,
    PrefabRef,
    PrefabOverrides,
    Mapping,
    Sequence,
    Scalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoringNodeOrigin {
    Root,
    UseRef,
    PrefabRef,
    Inline,
    Resolved,
}

#[derive(Debug, Clone)]
pub struct AuthoringNode {
    pub id: String,
    pub label: String,
    pub kind: AuthoringNodeKind,
    pub origin: AuthoringNodeOrigin,
    pub source_file: PathBuf,
    pub yaml_pointer: String,
    pub editable: bool,
    pub value: Value,
    pub value_preview: String,
    pub semantic: AuthoringNodeSemantic,
    pub children: Vec<AuthoringNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringNodeSummary {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub origin: String,
    pub source_file: String,
    pub yaml_pointer: String,
    pub editable: bool,
    pub value_preview: String,
    pub owner_entity_name: Option<String>,
    pub scene_object_id: Option<String>,
    pub component_type: Option<String>,
    pub render_layer_id: Option<String>,
    pub post_fx_id: Option<String>,
    pub post_fx_type: Option<String>,
    pub post_fx_scope: Option<String>,
    pub light_group_id: Option<String>,
    pub light_route_receiver_layer: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthoringNodeSemantic {
    pub parent_id: Option<String>,
    pub owner_entity_name: Option<String>,
    pub scene_object_id: Option<String>,
    pub component_type: Option<String>,
    pub render_layer_id: Option<String>,
    pub post_fx_id: Option<String>,
    pub post_fx_type: Option<String>,
    pub post_fx_scope: Option<String>,
    pub light_group_id: Option<String>,
    pub light_route_receiver_layer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthoringSceneGraph {
    pub source_mod: String,
    pub scene_id: String,
    pub root_file: PathBuf,
    pub source_files: Vec<PathBuf>,
    pub nodes: Vec<AuthoringNode>,
}

impl AuthoringNode {
    pub fn summary(&self) -> AuthoringNodeSummary {
        AuthoringNodeSummary {
            id: self.id.clone(),
            label: self.label.clone(),
            kind: format!("{:?}", self.kind),
            origin: format!("{:?}", self.origin),
            source_file: self.source_file.display().to_string(),
            yaml_pointer: self.yaml_pointer.clone(),
            editable: self.editable,
            value_preview: self.value_preview.clone(),
            owner_entity_name: self.semantic.owner_entity_name.clone(),
            scene_object_id: self.semantic.scene_object_id.clone(),
            component_type: self.semantic.component_type.clone(),
            render_layer_id: self.semantic.render_layer_id.clone(),
            post_fx_id: self.semantic.post_fx_id.clone(),
            post_fx_type: self.semantic.post_fx_type.clone(),
            post_fx_scope: self.semantic.post_fx_scope.clone(),
            light_group_id: self.semantic.light_group_id.clone(),
            light_route_receiver_layer: self.semantic.light_route_receiver_layer.clone(),
        }
    }
}

impl AuthoringNodeSummary {
    pub fn to_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("id: {}", self.id),
            format!("label: {}", self.label),
            format!("kind: {}", self.kind),
            format!("origin: {}", self.origin),
            format!("source: {}", self.source_file),
            format!("pointer: {}", self.yaml_pointer),
            format!("editable: {}", self.editable),
            format!("preview: {}", self.value_preview),
        ];
        if let Some(value) = &self.owner_entity_name {
            lines.push(format!("semantic.owner_entity: {value}"));
        }
        if let Some(value) = &self.scene_object_id {
            lines.push(format!("semantic.scene_object_id: {value}"));
        }
        if let Some(value) = &self.component_type {
            lines.push(format!("semantic.component_type: {value}"));
        }
        if let Some(value) = &self.render_layer_id {
            lines.push(format!("semantic.render_layer_id: {value}"));
        }
        if let Some(value) = &self.post_fx_id {
            lines.push(format!("semantic.post_fx_id: {value}"));
        }
        if let Some(value) = &self.post_fx_type {
            lines.push(format!("semantic.post_fx_type: {value}"));
        }
        if let Some(value) = &self.post_fx_scope {
            lines.push(format!("semantic.post_fx_scope: {value}"));
        }
        if let Some(value) = &self.light_group_id {
            lines.push(format!("semantic.light_group_id: {value}"));
        }
        if let Some(value) = &self.light_route_receiver_layer {
            lines.push(format!("semantic.light_route_receiver_layer: {value}"));
        }
        lines
    }
}

impl AuthoringSceneGraph {
    pub fn find_node(&self, id: &str) -> Option<&AuthoringNode> {
        fn walk<'a>(nodes: &'a [AuthoringNode], id: &str) -> Option<&'a AuthoringNode> {
            for node in nodes {
                if node.id == id {
                    return Some(node);
                }
                if let Some(found) = walk(&node.children, id) {
                    return Some(found);
                }
            }
            None
        }

        walk(&self.nodes, id)
    }

    pub fn first_editable_node_id(&self) -> Option<String> {
        fn walk(nodes: &[AuthoringNode]) -> Option<String> {
            for node in nodes {
                if node.editable {
                    return Some(node.id.clone());
                }
                if let Some(found) = walk(&node.children) {
                    return Some(found);
                }
            }
            None
        }

        walk(&self.nodes)
    }

    pub fn breadcrumb_for_node(&self, id: &str) -> Vec<String> {
        fn walk(nodes: &[AuthoringNode], id: &str, path: &mut Vec<String>) -> bool {
            for node in nodes {
                path.push(node.label.clone());
                if node.id == id {
                    return true;
                }
                if walk(&node.children, id, path) {
                    return true;
                }
                path.pop();
            }
            false
        }
        let mut path = Vec::new();
        if walk(&self.nodes, id, &mut path) {
            path
        } else {
            Vec::new()
        }
    }
}

pub fn value_preview(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Sequence(items) => format!("{} items", items.len()),
        Value::Mapping(mapping) => format!("{} fields", mapping.len()),
        Value::Tagged(tagged) => value_preview(&tagged.value),
    }
}
