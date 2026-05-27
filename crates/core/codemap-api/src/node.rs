use amigo_plugin_api::{CapabilityId, DiagnosticChannelId, DomainId, PluginId, SlotId, TargetId};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CodeMapNodeId {
    Plugin(PluginId),
    Domain(DomainId),
    Capability(CapabilityId),
    Slot(SlotId),
    Target(TargetId),
    DiagnosticChannel(DiagnosticChannelId),
    Contribution(String),
    Candidate(String),
    Consumer(String),
    Test(String),
    Doc(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodeMapNodeKind {
    Plugin,
    Domain,
    Capability,
    Slot,
    Target,
    DiagnosticChannel,
    Contribution,
    Candidate,
    Consumer,
    Test,
    Doc,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeMapNode {
    pub id: CodeMapNodeId,
    pub kind: CodeMapNodeKind,
    pub label: String,
    pub path: Option<String>,
}

impl CodeMapNode {
    pub fn new(id: CodeMapNodeId, kind: CodeMapNodeKind, label: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            label: label.into(),
            path: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}
