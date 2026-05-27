use crate::graph::CodeMapGraph;
use crate::node::CodeMapNodeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodeMapGraphValidationError {
    EdgeFromMissingNode(String),
    EdgeToMissingNode(String),
    EmptyNodeLabel(String),
}

pub type CodeMapGraphValidationResult = Result<(), Vec<CodeMapGraphValidationError>>;

pub fn validate_codemap_graph(graph: &CodeMapGraph) -> CodeMapGraphValidationResult {
    let mut errors = Vec::new();

    for node in graph.nodes.values() {
        if node.label.trim().is_empty() {
            errors.push(CodeMapGraphValidationError::EmptyNodeLabel(format_node_id(
                &node.id,
            )));
        }
    }

    for edge in &graph.edges {
        if !graph.contains_node(&edge.from) {
            errors.push(CodeMapGraphValidationError::EdgeFromMissingNode(
                format_node_id(&edge.from),
            ));
        }

        if !graph.contains_node(&edge.to) {
            errors.push(CodeMapGraphValidationError::EdgeToMissingNode(
                format_node_id(&edge.to),
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn format_node_id(id: &CodeMapNodeId) -> String {
    format!("{id:?}")
}
