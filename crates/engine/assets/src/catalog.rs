use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use crate::{
    AssetEvent, AssetKey, AssetLoadRequest, AssetManifest, AssetSourceKind, FailedAsset,
    LoadedAsset, PreparedAsset,
};

#[derive(Debug, Default)]
struct AssetCatalogState {
    manifests: BTreeMap<AssetKey, AssetManifest>,
    pending_loads: BTreeMap<AssetKey, AssetLoadRequest>,
    external_loads: BTreeSet<AssetKey>,
    loaded_assets: BTreeMap<AssetKey, LoadedAsset>,
    prepared_assets: BTreeMap<AssetKey, PreparedAsset>,
    failed_assets: BTreeMap<AssetKey, FailedAsset>,
    events: Vec<AssetEvent>,
}

#[derive(Debug, Default)]
pub struct AssetCatalog {
    state: Mutex<AssetCatalogState>,
}

/// Deduplicated catalog progress for a loading contribution. A key is counted
/// once even when it is instanced by many scene entities.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssetCatalogLoadingSummary {
    pub total_assets: usize,
    pub completed_assets: usize,
    pub pending_assets: usize,
    pub external_loading_assets: usize,
    pub failed_assets: usize,
    pub current_asset: Option<AssetKey>,
}

impl AssetCatalog {
    /// Returns the current catalog lifecycle as unique-asset progress. Domains
    /// can feed this into their `RuntimeLoadingJob` without duplicating work
    /// for repeated GLB instances.
    pub fn loading_summary(&self) -> AssetCatalogLoadingSummary {
        let state = self.state.lock().expect("asset catalog mutex poisoned");
        let mut keys = BTreeSet::new();
        keys.extend(state.pending_loads.keys().cloned());
        keys.extend(state.external_loads.iter().cloned());
        keys.extend(state.loaded_assets.keys().cloned());
        keys.extend(state.prepared_assets.keys().cloned());
        keys.extend(state.failed_assets.keys().cloned());
        let completed_assets = keys.iter().filter(|key| {
            state.loaded_assets.contains_key(*key) || state.prepared_assets.contains_key(*key)
        }).count();
        AssetCatalogLoadingSummary {
            total_assets: keys.len(),
            completed_assets,
            pending_assets: state.pending_loads.len(),
            external_loading_assets: state.external_loads.len(),
            failed_assets: state.failed_assets.len(),
            current_asset: state.pending_loads.keys().next().cloned()
                .or_else(|| state.external_loads.iter().next().cloned()),
        }
    }
    /// Records work owned by a domain loader without queuing it for the file loader.
    pub fn begin_external_load(&self, key: AssetKey) {
        let mut state = self.state.lock().expect("asset catalog mutex poisoned");
        state.failed_assets.remove(&key);
        state.prepared_assets.remove(&key);
        state.external_loads.insert(key);
    }

    /// Marks domain-owned preparation as in flight while retaining the parsed
    /// asset metadata. This is used when a loader has already produced a
    /// `PreparedAsset` record but still needs to build a heavy runtime cache.
    pub fn begin_external_prepare(&self, key: AssetKey) {
        let mut state = self.state.lock().expect("asset catalog mutex poisoned");
        state.failed_assets.remove(&key);
        state.external_loads.insert(key);
    }

    pub fn external_loading_keys(&self) -> BTreeSet<AssetKey> {
        self.state.lock().expect("asset catalog mutex poisoned").external_loads.clone()
    }

    pub fn register(&self, key: AssetKey) -> bool {
        self.register_manifest(AssetManifest::engine(key))
    }

    pub fn register_manifest(&self, manifest: AssetManifest) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let key = manifest.key.clone();
        let inserted = state.manifests.insert(key.clone(), manifest).is_none();

        if inserted {
            state.events.push(AssetEvent::ManifestRegistered(key));
        }

        inserted
    }

    pub fn request_load(&self, request: AssetLoadRequest) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        if state.loaded_assets.contains_key(&request.key) {
            return;
        }
        match state.pending_loads.get_mut(&request.key) {
            Some(existing) if request.priority > existing.priority => {
                existing.priority = request.priority;
            }
            Some(_) => {}
            None => {
                state
                    .pending_loads
                    .insert(request.key.clone(), request.clone());
            }
        }
        state.events.push(AssetEvent::LoadRequested(request));
    }

    pub fn request_reload(&self, request: AssetLoadRequest) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.remove(&request.key);
        state.prepared_assets.remove(&request.key);
        state.failed_assets.remove(&request.key);
        match state.pending_loads.get_mut(&request.key) {
            Some(existing) if request.priority > existing.priority => {
                existing.priority = request.priority;
            }
            Some(_) => {}
            None => {
                state
                    .pending_loads
                    .insert(request.key.clone(), request.clone());
            }
        }
        state.events.push(AssetEvent::ReloadRequested(request));
    }

    pub fn mark_loaded(&self, asset: LoadedAsset) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let key = asset.key.clone();
        state.pending_loads.remove(&key);
        state.external_loads.remove(&key);
        state.failed_assets.remove(&key);
        state.loaded_assets.insert(key.clone(), asset);
        state.events.push(AssetEvent::LoadCompleted(key));
    }

    pub fn mark_failed(&self, key: AssetKey, reason: impl Into<String>) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let reason = reason.into();
        state.external_loads.remove(&key);
        state.pending_loads.remove(&key);
        state.loaded_assets.remove(&key);
        state.prepared_assets.remove(&key);
        state.failed_assets.insert(
            key.clone(),
            FailedAsset {
                key: key.clone(),
                reason: reason.clone(),
            },
        );
        state.events.push(AssetEvent::LoadFailed { key, reason });
    }

    pub fn mark_prepared(&self, asset: PreparedAsset) {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let key = asset.key.clone();
        state.external_loads.remove(&key);
        state.failed_assets.remove(&key);
        state.prepared_assets.insert(key.clone(), asset);
        state.events.push(AssetEvent::Prepared(key));
    }

    pub fn contains(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.contains_key(key)
    }

    pub fn manifest(&self, key: &AssetKey) -> Option<AssetManifest> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.get(key).cloned()
    }

    pub fn registered_keys(&self) -> Vec<AssetKey> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.keys().cloned().collect()
    }

    pub fn manifests(&self) -> Vec<AssetManifest> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.manifests.values().cloned().collect()
    }

    pub fn manifests_for_mod(&self, mod_id: &str) -> Vec<AssetManifest> {
        self.manifests()
            .into_iter()
            .filter(|manifest| matches!(&manifest.source, AssetSourceKind::Mod(source_mod) if source_mod == mod_id))
            .collect()
    }

    pub fn manifests_with_tag(&self, tag: &str) -> Vec<AssetManifest> {
        self.manifests()
            .into_iter()
            .filter(|manifest| manifest.tags.iter().any(|entry| entry == tag))
            .collect()
    }

    pub fn tags_for(&self, key: &AssetKey) -> Vec<String> {
        self.manifest(key)
            .map(|manifest| manifest.tags)
            .unwrap_or_default()
    }

    pub fn loaded_asset(&self, key: &AssetKey) -> Option<LoadedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.get(key).cloned()
    }

    pub fn failed_asset(&self, key: &AssetKey) -> Option<FailedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.failed_assets.get(key).cloned()
    }

    pub fn loaded_assets(&self) -> Vec<LoadedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.values().cloned().collect()
    }

    pub fn prepared_asset(&self, key: &AssetKey) -> Option<PreparedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.prepared_assets.get(key).cloned()
    }

    pub fn prepared_assets(&self) -> Vec<PreparedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.prepared_assets.values().cloned().collect()
    }

    pub fn failed_assets(&self) -> Vec<FailedAsset> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.failed_assets.values().cloned().collect()
    }

    pub fn is_loaded(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.loaded_assets.contains_key(key)
    }

    pub fn is_prepared(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.prepared_assets.contains_key(key)
    }

    pub fn is_failed(&self, key: &AssetKey) -> bool {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.failed_assets.contains_key(key)
    }

    pub fn pending_loads(&self) -> Vec<AssetLoadRequest> {
        let state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let mut requests = state.pending_loads.values().cloned().collect::<Vec<_>>();
        requests.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.key.as_str().cmp(right.key.as_str()))
        });
        requests
    }

    pub fn drain_pending_loads(&self) -> Vec<AssetLoadRequest> {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        let drained = state.pending_loads.values().cloned().collect::<Vec<_>>();
        state.pending_loads.clear();

        let mut drained = drained;
        drained.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.key.as_str().cmp(right.key.as_str()))
        });
        drained
    }

    pub fn drain_events(&self) -> Vec<AssetEvent> {
        let mut state = self
            .state
            .lock()
            .expect("asset catalog mutex should not be poisoned");
        state.events.drain(..).collect()
    }
}
