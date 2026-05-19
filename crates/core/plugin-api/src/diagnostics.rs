use crate::ids::{DiagnosticChannelId, PluginId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticChannelRef {
    pub id: DiagnosticChannelId,
    pub owner: PluginId,
}

impl DiagnosticChannelRef {
    pub fn is_empty(&self) -> bool {
        self.id.0.trim().is_empty() || self.owner.0.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticTrace {
    pub channel: DiagnosticChannelId,
    pub summary: String,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticManifest {
    pub channels: Vec<DiagnosticChannelRef>,
}
