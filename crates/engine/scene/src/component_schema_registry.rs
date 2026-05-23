use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

use serde_yaml::{Mapping, Value};

use crate::{
    SceneDocumentError, SceneDocumentResult, ScenePluginComponentDescriptor,
    ScenePluginDescriptorProvider,
};

pub trait SceneComponentPayload: Send + Sync {
    fn component_type(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

pub trait SceneComponentSchemaProvider: Send + Sync {
    fn component_type(&self) -> &'static str;

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn parse_yaml(&self, payload: Mapping) -> Result<Value, serde_yaml::Error>;

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        let _ = payload;
        Err(SceneDocumentError::Compile {
            path: None,
            message: format!(
                "typed plugin payload parsing is not implemented for `{}`",
                self.component_type()
            ),
        })
    }
}

#[derive(Default)]
pub struct ComponentSchemaRegistry {
    descriptors: RwLock<BTreeMap<String, ScenePluginComponentDescriptor>>,
    schema_providers: RwLock<BTreeMap<String, Arc<dyn SceneComponentSchemaProvider>>>,
}

impl ComponentSchemaRegistry {
    pub fn register_descriptor(&self, descriptor: ScenePluginComponentDescriptor) {
        let mut descriptors = self
            .descriptors
            .write()
            .expect("component schema registry poisoned");
        descriptors.insert(descriptor.id.as_str().to_owned(), descriptor.clone());
        descriptors.insert(descriptor.label.clone(), descriptor);
    }

    pub fn register_provider(&self, provider: &impl ScenePluginDescriptorProvider) {
        let mut registry = crate::ScenePluginDescriptorRegistry::new();
        registry.register_provider(provider);
        for descriptor in registry.iter() {
            self.register_descriptor(descriptor.clone());
        }
    }

    pub fn register_schema_provider<P>(&self, provider: P)
    where
        P: SceneComponentSchemaProvider + 'static,
    {
        let provider = Arc::new(provider);
        let mut schema_providers = self
            .schema_providers
            .write()
            .expect("component schema registry poisoned");
        schema_providers.insert(provider.component_type().to_owned(), provider.clone());
        for alias in provider.aliases() {
            schema_providers.insert((*alias).to_owned(), provider.clone());
        }
    }

    pub fn get(&self, component_type: &str) -> Option<ScenePluginComponentDescriptor> {
        self.descriptors
            .read()
            .expect("component schema registry poisoned")
            .get(component_type)
            .cloned()
    }

    pub fn parse_plugin_payload(
        &self,
        component_type: &str,
        payload: Mapping,
    ) -> Option<Result<Value, serde_yaml::Error>> {
        self.schema_providers
            .read()
            .expect("component schema registry poisoned")
            .get(component_type)
            .cloned()
            .map(|provider| provider.parse_yaml(payload))
    }

    pub fn parse_typed_plugin_payload(
        &self,
        component_type: &str,
        payload: &Value,
    ) -> Option<SceneDocumentResult<Box<dyn SceneComponentPayload>>> {
        self.schema_providers
            .read()
            .expect("component schema registry poisoned")
            .get(component_type)
            .cloned()
            .map(|provider| provider.parse_payload_value(payload))
    }

    pub fn known_component_types(&self) -> Vec<String> {
        self.descriptors
            .read()
            .expect("component schema registry poisoned")
            .keys()
            .cloned()
            .collect()
    }
}
