use amigo_plugin_api::PluginId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RhaiPluginBindingProviderDescriptor {
    pub owner: PluginId,
    pub namespace: String,
    pub bindings: Vec<String>,
}

impl RhaiPluginBindingProviderDescriptor {
    pub fn new(owner: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            owner: PluginId(owner.into()),
            namespace: namespace.into(),
            bindings: Vec::new(),
        }
    }

    pub fn with_binding(mut self, binding: impl Into<String>) -> Self {
        self.bindings.push(binding.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.owner.0.trim().is_empty()
            && !self.namespace.trim().is_empty()
            && !self.bindings.is_empty()
            && self
                .bindings
                .iter()
                .all(|binding| !binding.trim().is_empty())
    }
}
