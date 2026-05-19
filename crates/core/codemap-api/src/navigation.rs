use crate::edge::{CodeMapEdge, CodeMapEdgeKind};
use crate::graph::CodeMapGraph;
use crate::node::CodeMapNodeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeMapNavigationResult {
    pub origin: CodeMapNodeId,
    pub edges: Vec<CodeMapEdge>,
}

pub fn navigate_by_edge_kind(
    graph: &CodeMapGraph,
    origin: &CodeMapNodeId,
    kind: CodeMapEdgeKind,
) -> CodeMapNavigationResult {
    let edges = graph
        .outgoing(origin)
        .into_iter()
        .filter(|edge| edge.kind == kind)
        .cloned()
        .collect();

    CodeMapNavigationResult {
        origin: origin.clone(),
        edges,
    }
}

pub fn navigate_to_targets(
    graph: &CodeMapGraph,
    origin: &CodeMapNodeId,
) -> CodeMapNavigationResult {
    let edges = graph
        .outgoing(origin)
        .into_iter()
        .filter(|edge| {
            matches!(
                edge.kind,
                CodeMapEdgeKind::ReadsTarget
                    | CodeMapEdgeKind::WritesTarget
                    | CodeMapEdgeKind::ContributesTarget
                    | CodeMapEdgeKind::ConsumesTarget
            )
        })
        .cloned()
        .collect();

    CodeMapNavigationResult {
        origin: origin.clone(),
        edges,
    }
}

pub fn navigate_to_diagnostics(
    graph: &CodeMapGraph,
    origin: &CodeMapNodeId,
) -> CodeMapNavigationResult {
    navigate_by_edge_kind(graph, origin, CodeMapEdgeKind::ProducesDiagnostic)
}
