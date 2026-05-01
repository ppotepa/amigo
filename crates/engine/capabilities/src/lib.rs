use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use amigo_core::AmigoResult;
use amigo_runtime::ServiceRegistry;

pub const DEFAULT_CAPABILITY_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy)]
pub struct CapabilityDescriptor {
    pub id: &'static str,
    pub provider: &'static str,
    pub version: &'static str,
    pub depends_on: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct PluginDescriptor {
    pub name: &'static str,
    pub provider: &'static str,
    pub version: &'static str,
    pub capabilities: &'static [&'static str],
    pub depends_on: &'static [&'static str],
}

#[derive(Default)]
struct CapabilityRegistryState {
    capabilities: BTreeMap<&'static str, CapabilityDescriptor>,
    plugins: BTreeMap<&'static str, PluginDescriptor>,
}

#[derive(Clone, Default)]
pub struct CapabilityRegistry {
    state: Arc<RwLock<CapabilityRegistryState>>,
}

impl CapabilityRegistry {
    fn with_write_lock(&self) -> std::sync::RwLockWriteGuard<'_, CapabilityRegistryState> {
        self.state
            .write()
            .expect("capability registry lock should be writable")
    }

    fn with_read_lock(&self) -> std::sync::RwLockReadGuard<'_, CapabilityRegistryState> {
        self.state
            .read()
            .expect("capability registry lock should be readable")
    }

    pub fn register_plugin(&self, plugin: PluginDescriptor) {
        let mut state = self.with_write_lock();

        for &capability_id in plugin.capabilities {
            state.capabilities.insert(
                capability_id,
                CapabilityDescriptor {
                    id: capability_id,
                    provider: plugin.provider,
                    version: plugin.version,
                    depends_on: plugin.depends_on,
                },
            );
        }

        state.plugins.insert(plugin.name, plugin);
    }

    pub fn capability_names(&self) -> Vec<String> {
        let state = self.with_read_lock();
        state
            .capabilities
            .keys()
            .map(|capability| (*capability).to_owned())
            .collect()
    }

    pub fn plugin_names(&self) -> Vec<String> {
        let state = self.with_read_lock();
        state
            .plugins
            .keys()
            .map(|plugin| (*plugin).to_owned())
            .collect()
    }

    pub fn plugins(&self) -> Vec<PluginDescriptor> {
        let state = self.with_read_lock();
        state.plugins.values().copied().collect()
    }
}

pub fn register_domain_plugin(
    registry: &mut ServiceRegistry,
    name: &'static str,
    capabilities: &'static [&'static str],
    depends_on: &'static [&'static str],
    version: &'static str,
) -> AmigoResult<()> {
    let plugin_descriptor = PluginDescriptor {
        name,
        provider: name,
        version,
        capabilities,
        depends_on,
    };

    register_plugin(registry, plugin_descriptor)
}

pub fn register_plugin(registry: &mut ServiceRegistry, plugin: PluginDescriptor) -> AmigoResult<()> {
    if !registry.has::<CapabilityRegistry>() {
        registry.register(CapabilityRegistry::default())?;
    };

    let registry_handle = registry
        .resolve::<CapabilityRegistry>()
        .expect("capability registry should be available after registration");

    registry_handle.register_plugin(plugin);
    Ok(())
}
