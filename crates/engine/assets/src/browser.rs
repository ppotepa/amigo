use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetSourceKind, PreparedAssetKind,
};

/// Stable identifier for an asset source exposed to UI consumers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetSourceId(String);

impl AssetSourceId {
    pub fn from_kind(kind: &AssetSourceKind) -> Self {
        Self(kind.label())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// UI-facing source metadata. It deliberately contains no filesystem handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetSourceDescriptor {
    pub id: AssetSourceId,
    pub label: String,
    pub refreshable: bool,
}

/// A browser-safe summary of one catalog asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBrowserEntry {
    pub key: AssetKey,
    pub source_id: AssetSourceId,
    pub tags: Vec<String>,
    pub kind: Option<PreparedAssetKind>,
    pub state: AssetBrowserEntryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetBrowserEntryState {
    Unloaded,
    Loading,
    Ready,
    Failed { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSourceCatalogState {
    Empty,
    Loading,
    Ready,
    Failed { failed_assets: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetSourceCatalogSnapshot {
    pub source: AssetSourceDescriptor,
    pub revision: u64,
    pub state: AssetSourceCatalogState,
    pub entries: Vec<AssetBrowserEntry>,
}

impl AssetCatalog {
    /// Lists discovered sources in deterministic label order.
    pub fn asset_sources(&self) -> Vec<AssetSourceDescriptor> {
        let mut sources = BTreeMap::new();
        for manifest in self.manifests() {
            let id = AssetSourceId::from_kind(&manifest.source);
            sources
                .entry(id.clone())
                .or_insert_with(|| AssetSourceDescriptor {
                    label: manifest.source.display_label(),
                    id,
                    refreshable: matches!(manifest.source, AssetSourceKind::FileSystemRoot(_)),
                });
        }
        sources.into_values().collect()
    }

    /// Captures one source without issuing I/O. Loading is represented by the
    /// existing catalog lifecycle and can therefore never block a UI frame.
    pub fn asset_source_snapshot(
        &self,
        source_id: &AssetSourceId,
    ) -> Option<AssetSourceCatalogSnapshot> {
        let source = self
            .asset_sources()
            .into_iter()
            .find(|source| &source.id == source_id)?;
        let pending = self
            .pending_loads()
            .into_iter()
            .map(|request| request.key)
            .chain(self.external_loading_keys())
            .collect::<BTreeSet<_>>();
        let prepared = self
            .prepared_assets()
            .into_iter()
            .map(|asset| (asset.key, asset.kind))
            .collect::<BTreeMap<_, _>>();
        let failed = self
            .failed_assets()
            .into_iter()
            .map(|asset| (asset.key, asset.reason))
            .collect::<BTreeMap<_, _>>();
        let mut entries = self
            .manifests()
            .into_iter()
            .filter_map(|manifest| {
                let id = AssetSourceId::from_kind(&manifest.source);
                (id == *source_id).then(|| {
                    let state = if let Some(message) = failed.get(&manifest.key) {
                        AssetBrowserEntryState::Failed {
                            message: message.clone(),
                        }
                    } else if prepared.contains_key(&manifest.key) {
                        AssetBrowserEntryState::Ready
                    } else if pending.contains(&manifest.key) {
                        AssetBrowserEntryState::Loading
                    } else {
                        AssetBrowserEntryState::Unloaded
                    };
                    AssetBrowserEntry {
                        key: manifest.key.clone(),
                        source_id: id,
                        tags: manifest.tags,
                        kind: prepared.get(&manifest.key).cloned(),
                        state,
                    }
                })
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.as_str().cmp(right.key.as_str()));
        let failures = entries
            .iter()
            .filter(|entry| matches!(entry.state, AssetBrowserEntryState::Failed { .. }))
            .count();
        let state = if entries.is_empty() {
            AssetSourceCatalogState::Empty
        } else if entries
            .iter()
            .any(|entry| matches!(entry.state, AssetBrowserEntryState::Loading))
        {
            AssetSourceCatalogState::Loading
        } else if failures > 0 {
            AssetSourceCatalogState::Failed {
                failed_assets: failures,
            }
        } else {
            AssetSourceCatalogState::Ready
        };
        Some(AssetSourceCatalogSnapshot {
            source,
            revision: entries.len() as u64,
            state,
            entries,
        })
    }

    /// Re-queues every asset in a source. The consumer observes Loading via a
    /// subsequent snapshot and drives actual I/O through the normal pipeline.
    pub fn refresh_asset_source(&self, source_id: &AssetSourceId) -> bool {
        let keys = self
            .manifests()
            .into_iter()
            .filter_map(|manifest| {
                (AssetSourceId::from_kind(&manifest.source) == *source_id).then_some(manifest.key)
            })
            .collect::<Vec<_>>();
        for key in &keys {
            self.request_reload(AssetLoadRequest::new(
                key.clone(),
                AssetLoadPriority::Interactive,
            ));
        }
        !keys.is_empty()
    }
}
