use crate::ids::{DiagnosticChannelId, PluginId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticChannelRef {
    pub id: DiagnosticChannelId,
    pub owner: PluginId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticTrace {
    pub channel: DiagnosticChannelId,
    pub summary: String,
    pub reason: Option<String>,
}
