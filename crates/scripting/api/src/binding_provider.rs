use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use amigo_core::{AmigoError, AmigoResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptBindingProviderDescriptor {
    pub owner: String,
    pub namespace: String,
    pub bindings: Vec<String>,
}

impl ScriptBindingProviderDescriptor {
    pub fn new(owner: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            namespace: namespace.into(),
            bindings: Vec::new(),
        }
    }

    pub fn with_binding(mut self, binding: impl Into<String>) -> Self {
        self.bindings.push(binding.into());
        self
    }

    pub fn validate(&self) -> AmigoResult<()> {
        if self.owner.trim().is_empty() {
            return Err(AmigoError::Message(
                "script binding provider owner must not be empty".to_owned(),
            ));
        }
        if self.namespace.trim().is_empty() {
            return Err(AmigoError::Message(
                "script binding provider namespace must not be empty".to_owned(),
            ));
        }
        if self.bindings.is_empty()
            || self
                .bindings
                .iter()
                .any(|binding| binding.trim().is_empty())
        {
            return Err(AmigoError::Message(format!(
                "script binding provider `{}` must declare non-empty bindings",
                self.owner
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct ScriptBindingProviderRegistry {
    providers: Arc<RwLock<BTreeMap<String, ScriptBindingProviderDescriptor>>>,
}

impl ScriptBindingProviderRegistry {
    pub fn register(&self, descriptor: ScriptBindingProviderDescriptor) -> AmigoResult<()> {
        descriptor.validate()?;
        let mut providers = self
            .providers
            .write()
            .expect("script binding provider registry should be writable");
        if let Some(existing) = providers.get(&descriptor.namespace) {
            if existing != &descriptor {
                return Err(AmigoError::Message(format!(
                    "script binding namespace `{}` is already owned by `{}`",
                    descriptor.namespace, existing.owner
                )));
            }
            return Ok(());
        }
        providers.insert(descriptor.namespace.clone(), descriptor);
        Ok(())
    }

    pub fn provider(&self, namespace: &str) -> Option<ScriptBindingProviderDescriptor> {
        self.providers
            .read()
            .expect("script binding provider registry should be readable")
            .get(namespace)
            .cloned()
    }

    pub fn providers(&self) -> Vec<ScriptBindingProviderDescriptor> {
        self.providers
            .read()
            .expect("script binding provider registry should be readable")
            .values()
            .cloned()
            .collect()
    }

    pub fn namespaces(&self) -> Vec<String> {
        self.providers
            .read()
            .expect("script binding provider registry should be readable")
            .keys()
            .cloned()
            .collect()
    }
}
