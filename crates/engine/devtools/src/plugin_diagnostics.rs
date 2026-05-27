use amigo_plugin_api::{DiagnosticChannelId, PluginId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDiagnosticProviderDescriptor {
    pub owner: PluginId,
    pub channels: Vec<DiagnosticChannelId>,
}

impl PluginDiagnosticProviderDescriptor {
    pub fn new(owner: impl Into<String>) -> Self {
        Self {
            owner: PluginId(owner.into()),
            channels: Vec::new(),
        }
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channels.push(DiagnosticChannelId(channel.into()));
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.owner.0.trim().is_empty()
            && !self.channels.is_empty()
            && self
                .channels
                .iter()
                .all(|channel| !channel.0.trim().is_empty())
    }
}
