use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginSceneComponentId(pub String);

impl PluginSceneComponentId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Display for PluginSceneComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSceneComponentDescriptor {
    pub id: PluginSceneComponentId,
    pub domain: String,
    pub label: String,
}

impl PluginSceneComponentDescriptor {
    pub fn new(
        id: impl Into<String>,
        domain: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: PluginSceneComponentId::new(id),
            domain: domain.into(),
            label: label.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
            && !self.domain.trim().is_empty()
            && !self.label.trim().is_empty()
    }
}

