use std::collections::BTreeMap;

use super::{
    SceneGraphDiagnostic, SceneGraphNode, SceneGraphNodeId, SceneGraphNodeKind, SceneReferenceEdge,
};

#[derive(Debug, Clone)]
pub struct SemanticSceneGraph {
    pub root: SceneGraphNodeId,
    pub nodes: BTreeMap<SceneGraphNodeId, SceneGraphNode>,
    pub references: Vec<SceneReferenceEdge>,
    pub diagnostics: Vec<SceneGraphDiagnostic>,
}

impl SemanticSceneGraph {
    pub fn new(scene_id: &str) -> Self {
        let root = SceneGraphNodeId::new(format!("scene:{scene_id}"));
        let root_node = SceneGraphNode::new(root.clone(), scene_id, SceneGraphNodeKind::Root);

        let mut nodes = BTreeMap::new();
        nodes.insert(root.clone(), root_node);

        Self {
            root,
            nodes,
            references: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn add_child(
        &mut self,
        parent: &SceneGraphNodeId,
        mut node: SceneGraphNode,
    ) -> SceneGraphNodeId {
        let id = node.id.clone();

        if self.nodes.contains_key(&id) {
            self.diagnostics.push(SceneGraphDiagnostic::error(
                "duplicate_scene_graph_node",
                format!("duplicate scene graph node `{id}`"),
                Some(id.clone()),
            ));
            return id;
        }

        node.parent = Some(parent.clone());

        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.children.push(id.clone());
        } else {
            self.diagnostics.push(SceneGraphDiagnostic::error(
                "missing_scene_graph_parent",
                format!("missing scene graph parent `{parent}` for `{id}`"),
                Some(id.clone()),
            ));
        }

        self.nodes.insert(id.clone(), node);
        id
    }

    pub fn add_reference(&mut self, edge: SceneReferenceEdge) {
        self.references.push(edge);
    }

    pub fn add_diagnostic(&mut self, diagnostic: SceneGraphDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn node(&self, id: &SceneGraphNodeId) -> Option<&SceneGraphNode> {
        self.nodes.get(id)
    }

    pub fn nodes_of_kind(&self, kind: SceneGraphNodeKind) -> Vec<&SceneGraphNode> {
        self.nodes
            .values()
            .filter(|node| node.kind == kind)
            .collect()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == super::SceneGraphDiagnosticSeverity::Error)
    }
}
