//! Capability registry describing which plugins and domains are active.
//! It lets bootstrap code and diagnostics expose the engine feature surface at runtime.
//!
//! Manifest-declared capabilities are canonical for domain plugins. The older
//! static runtime descriptors remain available for engine/runtime features that
//! do not have a `plugin.toml` manifest.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use amigo_core::AmigoResult;
use amigo_plugin_api::{CapabilityId, CapabilityRef, PluginId, PluginManifest};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestCapabilityDescriptor {
    pub id: CapabilityId,
    pub provider: PluginId,
    pub version: u32,
}

#[derive(Default)]
struct CapabilityRegistryState {
    runtime_capabilities: BTreeMap<&'static str, CapabilityDescriptor>,
    runtime_plugins: BTreeMap<&'static str, PluginDescriptor>,
    manifest_capabilities: BTreeMap<(String, u32), ManifestCapabilityDescriptor>,
    manifest_plugins: BTreeMap<String, Vec<CapabilityRef>>,
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

    pub fn register_manifest(&self, manifest: &PluginManifest) {
        let mut state = self.with_write_lock();
        let provided = manifest.capabilities.provides.clone();
        for capability in &provided {
            state.manifest_capabilities.insert(
                (capability.id.0.clone(), capability.version),
                ManifestCapabilityDescriptor {
                    id: capability.id.clone(),
                    provider: manifest.id.clone(),
                    version: capability.version,
                },
            );
        }
        state
            .manifest_plugins
            .insert(manifest.id.0.clone(), provided);
    }

    pub fn register_plugin(&self, plugin: PluginDescriptor) {
        let mut state = self.with_write_lock();

        for &capability_id in plugin.capabilities {
            state.runtime_capabilities.insert(
                capability_id,
                CapabilityDescriptor {
                    id: capability_id,
                    provider: plugin.provider,
                    version: plugin.version,
                    depends_on: plugin.depends_on,
                },
            );
        }

        state.runtime_plugins.insert(plugin.name, plugin);
    }

    pub fn capability_names(&self) -> Vec<String> {
        let state = self.with_read_lock();
        let mut names = state
            .manifest_capabilities
            .values()
            .map(|capability| capability.id.0.clone())
            .chain(
                state
                    .runtime_capabilities
                    .keys()
                    .map(|capability| (*capability).to_owned()),
            )
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    pub fn manifest_capabilities(&self) -> Vec<ManifestCapabilityDescriptor> {
        self.with_read_lock()
            .manifest_capabilities
            .values()
            .cloned()
            .collect()
    }

    pub fn plugin_names(&self) -> Vec<String> {
        let state = self.with_read_lock();
        let mut names = state
            .manifest_plugins
            .keys()
            .cloned()
            .chain(state.runtime_plugins.keys().map(|plugin| (*plugin).to_owned()))
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    pub fn plugins(&self) -> Vec<PluginDescriptor> {
        let state = self.with_read_lock();
        state.runtime_plugins.values().copied().collect()
    }
}

fn capability_registry(registry: &mut ServiceRegistry) -> AmigoResult<Arc<CapabilityRegistry>> {
    if !registry.has::<CapabilityRegistry>() {
        registry.register(CapabilityRegistry::default())?;
    }

    Ok(registry
        .resolve::<CapabilityRegistry>()
        .expect("capability registry should be available after registration"))
}

pub fn register_plugin_manifest(
    registry: &mut ServiceRegistry,
    manifest: &PluginManifest,
) -> AmigoResult<()> {
    capability_registry(registry)?.register_manifest(manifest);
    Ok(())
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

pub fn register_plugin(
    registry: &mut ServiceRegistry,
    plugin: PluginDescriptor,
) -> AmigoResult<()> {
    capability_registry(registry)?.register_plugin(plugin);
    Ok(())
}
