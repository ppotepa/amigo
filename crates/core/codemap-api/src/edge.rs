use crate::node::CodeMapNodeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodeMapEdgeKind {
    Provides,
    Requires,
    ImplementsSlot,
    Replaces,
    ReadsTarget,
    WritesTarget,
    ContributesTarget,
    EmitsContribution,
    ConsumesContribution,
    ResolvesCandidate,
    ConsumesTarget,
    ProducesDiagnostic,
    CoveredByTest,
    DocumentedBy,
    Owns,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeMapEdge {
    pub from: CodeMapNodeId,
    pub to: CodeMapNodeId,
    pub kind: CodeMapEdgeKind,
    pub label: Option<String>,
}

impl CodeMapEdge {
    pub fn new(from: CodeMapNodeId, to: CodeMapNodeId, kind: CodeMapEdgeKind) -> Self {
        Self {
            from,
            to,
            kind,
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}
