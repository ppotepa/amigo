use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

use serde_yaml::{Mapping, Value};

use amigo_core::{AmigoError, AmigoResult};

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
        self.try_register_descriptor(descriptor)
            .expect("duplicate component schema descriptor");
    }

    pub fn try_register_descriptor(
        &self,
        descriptor: ScenePluginComponentDescriptor,
    ) -> AmigoResult<()> {
        let mut descriptors = self
            .descriptors
            .write()
            .expect("component schema registry poisoned");
        let id = descriptor.id.as_str().to_owned();
        if descriptors.contains_key(&id) {
            return Err(AmigoError::Message(format!(
                "duplicate component schema descriptor `{id}`"
            )));
        }
        if descriptors.contains_key(&descriptor.label) {
            return Err(AmigoError::Message(format!(
                "duplicate component schema descriptor label `{}`",
                descriptor.label
            )));
        }

        descriptors.insert(descriptor.id.as_str().to_owned(), descriptor.clone());
        descriptors.insert(descriptor.label.clone(), descriptor);
        Ok(())
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
        self.try_register_schema_provider(provider)
            .expect("duplicate component schema provider");
    }

    pub fn try_register_schema_provider<P>(&self, provider: P) -> AmigoResult<()>
    where
        P: SceneComponentSchemaProvider + 'static,
    {
        let provider = Arc::new(provider);
        let mut schema_providers = self
            .schema_providers
            .write()
            .expect("component schema registry poisoned");
        let component_type = provider.component_type().to_owned();
        if schema_providers.contains_key(&component_type) {
            return Err(AmigoError::Message(format!(
                "duplicate component schema provider `{component_type}`"
            )));
        }
        for alias in provider.aliases() {
            if schema_providers.contains_key(*alias) {
                return Err(AmigoError::Message(format!(
                    "duplicate component schema provider alias `{alias}`"
                )));
            }
        }

        schema_providers.insert(component_type, provider.clone());
        for alias in provider.aliases() {
            schema_providers.insert((*alias).to_owned(), provider.clone());
        }
        Ok(())
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

    pub fn parse_plugin_payload_with_canonical_type(
        &self,
        component_type: &str,
        payload: Mapping,
    ) -> Option<Result<(String, Value), serde_yaml::Error>> {
        self.schema_providers
            .read()
            .expect("component schema registry poisoned")
            .get(component_type)
            .cloned()
            .map(|provider| {
                provider
                    .parse_yaml(payload)
                    .map(|payload| (provider.component_type().to_owned(), payload))
            })
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
