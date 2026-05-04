//! Dependency-injection runtime and plugin composition for Amigo.
//! It owns service registration, plugin bundles, and phased system dispatch used by the app host.

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};

mod bundle;
mod handler_registry;
mod schedule;

pub use bundle::PluginBundle;
pub use handler_registry::HandlerDispatcher;
pub use handler_registry::HandlerRegistry;
pub use handler_registry::RoutedHandler;
pub use handler_registry::RoutedHandlerRegistry;
pub use handler_registry::register_routed_handler;
pub use schedule::RuntimeSystem;
pub use schedule::SystemPhase;
pub use schedule::SystemRegistry;

pub trait RuntimePlugin {
    fn name(&self) -> &'static str;
    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()>;
}

#[derive(Clone)]
struct ServiceEntry {
    name: &'static str,
    value: Arc<dyn Any + Send + Sync>,
}

#[derive(Default)]
pub struct ServiceRegistry {
    services: HashMap<TypeId, ServiceEntry>,
}

impl ServiceRegistry {
    pub fn register<T>(&mut self, service: T) -> AmigoResult<()>
    where
        T: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        if self.services.contains_key(&type_id) {
            return Err(AmigoError::DuplicateService(type_name));
        }

        self.services.insert(
            type_id,
            ServiceEntry {
                name: type_name,
                value: Arc::new(service),
            },
        );

        Ok(())
    }

    pub fn resolve<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.value.clone().downcast::<T>().ok())
    }

    pub fn has<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.services.contains_key(&TypeId::of::<T>())
    }

    pub fn registered_names(&self) -> Vec<&'static str> {
        let mut names = self
            .services
            .values()
            .map(|entry| entry.name)
            .collect::<Vec<_>>();
        names.sort_unstable();
        names
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeReport {
    pub plugin_names: Vec<&'static str>,
    pub service_names: Vec<&'static str>,
}

pub struct Runtime {
    registry: ServiceRegistry,
    plugin_names: Vec<&'static str>,
}

impl Runtime {
    pub fn resolve<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.registry.resolve::<T>()
    }

    pub fn has<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.registry.has::<T>()
    }

    pub fn report(&self) -> RuntimeReport {
        RuntimeReport {
            plugin_names: self.plugin_names.clone(),
            service_names: self.registry.registered_names(),
        }
    }
}

#[derive(Default)]
pub struct RuntimeBuilder {
    registry: ServiceRegistry,
    plugin_names: Vec<&'static str>,
}

impl RuntimeBuilder {
    pub fn with_service<T>(mut self, service: T) -> AmigoResult<Self>
    where
        T: Send + Sync + 'static,
    {
        self.registry.register(service)?;
        Ok(self)
    }

    pub fn with_plugin<P>(mut self, plugin: P) -> AmigoResult<Self>
    where
        P: RuntimePlugin,
    {
        plugin.register(&mut self.registry)?;
        self.plugin_names.push(plugin.name());
        Ok(self)
    }

    pub fn with_bundle<B>(self, bundle: B) -> AmigoResult<Self>
    where
        B: PluginBundle,
    {
        bundle.register(self)
    }

    pub fn build(self) -> Runtime {
        Runtime {
            registry: self.registry,
            plugin_names: self.plugin_names,
        }
    }
}
