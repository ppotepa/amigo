use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

use amigo_runtime::{RuntimePlugin, ServiceRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneDocumentWatch {
    pub source_mod: String,
    pub scene_id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetWatch {
    pub asset_key: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotReloadWatchKind {
    SceneDocument {
        source_mod: String,
        scene_id: String,
    },
    Asset {
        asset_key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotReloadWatchDescriptor {
    pub id: String,
    pub kind: HotReloadWatchKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotReloadChange {
    pub watch: HotReloadWatchDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileStamp {
    Missing,
    Present {
        modified_millis: Option<u128>,
        byte_len: u64,
    },
}

#[derive(Debug, Clone)]
struct HotReloadWatchRecord {
    descriptor: HotReloadWatchDescriptor,
    stamp: FileStamp,
}

#[derive(Debug, Default)]
struct HotReloadState {
    watches: BTreeMap<String, HotReloadWatchRecord>,
}

#[derive(Debug, Default)]
pub struct HotReloadService {
    state: Mutex<HotReloadState>,
}

impl HotReloadService {
    pub fn sync_scene_document(&self, watch: Option<SceneDocumentWatch>) {
        let mut state = self
            .state
            .lock()
            .expect("hot reload state mutex should not be poisoned");

        let active_scene_watch_id = watch.as_ref().map(scene_watch_id);
        state.watches.retain(|watch_id, record| {
            !matches!(
                record.descriptor.kind,
                HotReloadWatchKind::SceneDocument { .. }
            ) || active_scene_watch_id
                .as_ref()
                .is_some_and(|active| active == watch_id)
        });

        if let Some(watch) = watch {
            upsert_watch(
                &mut state,
                scene_watch_id(&watch),
                HotReloadWatchKind::SceneDocument {
                    source_mod: watch.source_mod,
                    scene_id: watch.scene_id,
                },
                watch.path,
            );
        }
    }

    pub fn sync_assets(&self, watches: Vec<AssetWatch>) {
        let mut state = self
            .state
            .lock()
            .expect("hot reload state mutex should not be poisoned");
        let active_watch_ids = watches.iter().map(asset_watch_id).collect::<BTreeSet<_>>();

        state.watches.retain(|watch_id, record| {
            !matches!(record.descriptor.kind, HotReloadWatchKind::Asset { .. })
                || active_watch_ids.contains(watch_id)
        });

        for watch in watches {
            upsert_watch(
                &mut state,
                asset_watch_id(&watch),
                HotReloadWatchKind::Asset {
                    asset_key: watch.asset_key,
                },
                watch.path,
            );
        }
    }

    pub fn watched_targets(&self) -> Vec<HotReloadWatchDescriptor> {
        let state = self
            .state
            .lock()
            .expect("hot reload state mutex should not be poisoned");
        state
            .watches
            .values()
            .map(|record| record.descriptor.clone())
            .collect()
    }

    pub fn poll_changes(&self) -> Vec<HotReloadChange> {
        let mut state = self
            .state
            .lock()
            .expect("hot reload state mutex should not be poisoned");
        let mut changes = Vec::new();

        for record in state.watches.values_mut() {
            let current_stamp = file_stamp(&record.descriptor.path);
            if current_stamp != record.stamp {
                record.stamp = current_stamp;
                changes.push(HotReloadChange {
                    watch: record.descriptor.clone(),
                });
            }
        }

        changes
    }

    pub fn changes_for_paths(&self, paths: &[PathBuf]) -> Vec<HotReloadChange> {
        let mut state = self
            .state
            .lock()
            .expect("hot reload state mutex should not be poisoned");
        let desired = paths.iter().cloned().collect::<BTreeSet<_>>();
        let mut changes = Vec::new();

        for record in state.watches.values_mut() {
            if desired.contains(&record.descriptor.path) {
                record.stamp = file_stamp(&record.descriptor.path);
                changes.push(HotReloadChange {
                    watch: record.descriptor.clone(),
                });
            }
        }

        changes
    }
}

fn upsert_watch(
    state: &mut HotReloadState,
    watch_id: String,
    kind: HotReloadWatchKind,
    path: PathBuf,
) {
    if let Some(existing) = state.watches.get_mut(&watch_id) {
        if existing.descriptor.path != path {
            existing.descriptor.path = path.clone();
            existing.stamp = file_stamp(&path);
        }
        existing.descriptor.kind = kind;
        return;
    }

    state.watches.insert(
        watch_id.clone(),
        HotReloadWatchRecord {
            descriptor: HotReloadWatchDescriptor {
                id: watch_id,
                kind,
                path: path.clone(),
            },
            stamp: file_stamp(&path),
        },
    );
}

fn scene_watch_id(watch: &SceneDocumentWatch) -> String {
    format!("scene:{}:{}", watch.source_mod, watch.scene_id)
}

fn asset_watch_id(watch: &AssetWatch) -> String {
    format!("asset:{}", watch.asset_key)
}

fn file_stamp(path: &Path) -> FileStamp {
    match fs::metadata(path) {
        Ok(metadata) => {
            let modified_millis = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis());
            FileStamp::Present {
                modified_millis,
                byte_len: metadata.len(),
            }
        }
        Err(_) => FileStamp::Missing,
    }
}

pub struct HotReloadPlugin;

impl RuntimePlugin for HotReloadPlugin {
    fn name(&self) -> &'static str {
        "amigo-hot-reload"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(HotReloadService::default())
    }
}

#[cfg(test)]
include!("tests.rs");
