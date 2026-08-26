use std::sync::RwLock;

use crate::component_metadata::{ComponentRegistry, ComponentTypeDescriptor};

pub trait ComponentMetadataProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn register_component_metadata(&self, registry: &mut ComponentRegistry);
}

#[derive(Default)]
pub struct ComponentMetadataProviderRegistry {
    providers: RwLock<Vec<Box<dyn ComponentMetadataProvider>>>,
}

impl ComponentMetadataProviderRegistry {
    pub fn register<P>(&self, provider: P)
    where
        P: ComponentMetadataProvider + 'static,
    {
        self.providers
            .write()
            .expect("component metadata provider registry poisoned")
            .push(Box::new(provider));
    }

    pub fn apply_all(&self, registry: &mut ComponentRegistry) {
        let providers = self
            .providers
            .read()
            .expect("component metadata provider registry poisoned");
        for provider in providers.iter() {
            provider.register_component_metadata(registry);
        }
    }

    /// Compose the engine's base descriptors with all domain-owned providers.
    /// New domain metadata should enter through providers rather than growing the
    /// central built-in metadata module.
    pub fn compose(
        &self,
        base: impl IntoIterator<Item = ComponentTypeDescriptor>,
    ) -> ComponentRegistry {
        let mut registry = ComponentRegistry::new(base);
        self.apply_all(&mut registry);
        registry
    }

    pub fn provider_ids(&self) -> Vec<&'static str> {
        self.providers
            .read()
            .expect("component metadata provider registry poisoned")
            .iter()
            .map(|provider| provider.provider_id())
            .collect()
    }
}
