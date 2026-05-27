use std::collections::HashMap;

use amigo_plugin_api::{PluginId, PluginManifest};

#[derive(Clone, Debug, Default)]
pub struct PluginIndex {
    manifests: HashMap<PluginId, PluginManifest>,
}

impl PluginIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_manifests(manifests: impl IntoIterator<Item = PluginManifest>) -> Self {
        let mut index = Self::new();

        for manifest in manifests {
            index.insert(manifest);
        }

        index
    }

    pub fn insert(&mut self, manifest: PluginManifest) {
        self.manifests.insert(manifest.id.clone(), manifest);
    }

    pub fn get(&self, id: &PluginId) -> Option<&PluginManifest> {
        self.manifests.get(id)
    }

    pub fn manifests(&self) -> impl Iterator<Item = &PluginManifest> {
        self.manifests.values()
    }

    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }
}
