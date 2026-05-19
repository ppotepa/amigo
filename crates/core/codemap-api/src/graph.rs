use std::collections::HashMap;

use crate::edge::CodeMapEdge;
use crate::node::{CodeMapNode, CodeMapNodeId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeMapGraph {
    pub nodes: HashMap<CodeMapNodeId, CodeMapNode>,
    pub edges: Vec<CodeMapEdge>,
}

impl CodeMapGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: CodeMapNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: CodeMapEdge) {
        self.edges.push(edge);
    }

    pub fn node(&self, id: &CodeMapNodeId) -> Option<&CodeMapNode> {
        self.nodes.get(id)
    }

    pub fn outgoing(&self, id: &CodeMapNodeId) -> Vec<&CodeMapEdge> {
        self.edges.iter().filter(|edge| &edge.from == id).collect()
    }

    pub fn incoming(&self, id: &CodeMapNodeId) -> Vec<&CodeMapEdge> {
        self.edges.iter().filter(|edge| &edge.to == id).collect()
    }

    pub fn contains_node(&self, id: &CodeMapNodeId) -> bool {
        self.nodes.contains_key(id)
    }
}
