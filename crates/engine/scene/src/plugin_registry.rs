use std::collections::BTreeMap;

pub type ScenePluginComponentDescriptor = amigo_plugin_api::PluginSceneComponentDescriptor;
pub type ScenePluginComponentId = amigo_plugin_api::PluginSceneComponentId;

pub trait ScenePluginDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry);
}

pub trait ScenePluginMetadataProvider: Send + Sync {
    fn register_component_metadata(&self, registry: &mut crate::ComponentRegistry);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScenePluginDescriptorRegistry {
    descriptors: BTreeMap<ScenePluginComponentId, ScenePluginComponentDescriptor>,
}

impl ScenePluginDescriptorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        descriptor: ScenePluginComponentDescriptor,
    ) -> Option<ScenePluginComponentDescriptor> {
        self.descriptors.insert(descriptor.id.clone(), descriptor)
    }

    pub fn get(&self, id: &ScenePluginComponentId) -> Option<&ScenePluginComponentDescriptor> {
        self.descriptors.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScenePluginComponentDescriptor> {
        self.descriptors.values()
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn invalid_descriptors(&self) -> Vec<&ScenePluginComponentDescriptor> {
        self.descriptors
            .values()
            .filter(|descriptor| !descriptor.is_valid())
            .collect()
    }

    pub fn register_provider(&mut self, provider: &impl ScenePluginDescriptorProvider) {
        provider.register_scene_descriptors(self);
    }
}
