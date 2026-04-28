use std::sync::Arc;

use amigo_assets::{AssetCatalog, AssetKey};
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::queue_asset_reload;
use crate::bindings::common::string_array;
use crate::handles::AssetRef;

#[derive(Clone)]
pub struct AssetsApi {
    pub(crate) asset_catalog: Option<Arc<AssetCatalog>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl AssetsApi {
    pub fn get(&mut self, asset_key: &str) -> AssetRef {
        AssetRef::new(
            self.asset_catalog.clone(),
            self.command_queue.clone(),
            asset_key,
        )
    }

    pub fn has(&mut self, asset_key: &str) -> bool {
        asset_exists(self.asset_catalog.as_ref(), asset_key)
    }

    pub fn registered(&mut self) -> rhai::Array {
        string_array(registered_asset_keys(self.asset_catalog.as_ref()))
    }

    pub fn by_mod(&mut self, mod_id: &str) -> rhai::Array {
        string_array(asset_keys_for_mod(self.asset_catalog.as_ref(), mod_id))
    }

    pub fn reload(&mut self, asset_key: &str) -> bool {
        queue_asset_reload(self.command_queue.as_ref(), asset_key)
    }

    pub fn pending(&mut self) -> rhai::Array {
        string_array(pending_asset_keys(self.asset_catalog.as_ref()))
    }

    pub fn loaded(&mut self) -> rhai::Array {
        string_array(loaded_asset_keys(self.asset_catalog.as_ref()))
    }

    pub fn prepared(&mut self) -> rhai::Array {
        string_array(prepared_asset_keys(self.asset_catalog.as_ref()))
    }

    pub fn failed(&mut self) -> rhai::Array {
        string_array(failed_asset_keys(self.asset_catalog.as_ref()))
    }
}

pub fn asset_exists(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> bool {
    asset_catalog
        .map(|asset_catalog| asset_catalog.contains(&AssetKey::new(asset_key)))
        .unwrap_or(false)
}

pub fn registered_asset_keys(asset_catalog: Option<&Arc<AssetCatalog>>) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| {
            asset_catalog
                .registered_keys()
                .into_iter()
                .map(|key| key.as_str().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn asset_keys_for_mod(asset_catalog: Option<&Arc<AssetCatalog>>, mod_id: &str) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| {
            asset_catalog
                .manifests_for_mod(mod_id)
                .into_iter()
                .map(|manifest| manifest.key.as_str().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn pending_asset_keys(asset_catalog: Option<&Arc<AssetCatalog>>) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| {
            asset_catalog
                .pending_loads()
                .into_iter()
                .map(|request| request.key.as_str().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn loaded_asset_keys(asset_catalog: Option<&Arc<AssetCatalog>>) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| {
            asset_catalog
                .loaded_assets()
                .into_iter()
                .map(|asset| asset.key.as_str().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn prepared_asset_keys(asset_catalog: Option<&Arc<AssetCatalog>>) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| {
            asset_catalog
                .prepared_assets()
                .into_iter()
                .map(|asset| asset.key.as_str().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn failed_asset_keys(asset_catalog: Option<&Arc<AssetCatalog>>) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| {
            asset_catalog
                .failed_assets()
                .into_iter()
                .map(|asset| asset.key.as_str().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn asset_tags(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> Vec<String> {
    asset_catalog
        .map(|asset_catalog| asset_catalog.tags_for(&AssetKey::new(asset_key)))
        .unwrap_or_default()
}

pub fn asset_source(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    asset_catalog
        .and_then(|asset_catalog| asset_catalog.manifest(&AssetKey::new(asset_key)))
        .map(|manifest| manifest.source.label())
        .unwrap_or_default()
}

pub fn asset_state(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    let Some(asset_catalog) = asset_catalog else {
        return String::new();
    };
    let asset_key = AssetKey::new(asset_key);

    if asset_catalog.is_prepared(&asset_key) {
        "prepared".to_owned()
    } else if asset_catalog.is_loaded(&asset_key) {
        "loaded".to_owned()
    } else if asset_catalog.is_failed(&asset_key) {
        "failed".to_owned()
    } else if asset_catalog
        .pending_loads()
        .iter()
        .any(|request| request.key == asset_key)
    {
        "pending".to_owned()
    } else if asset_catalog.contains(&asset_key) {
        "registered".to_owned()
    } else {
        String::new()
    }
}

pub fn asset_path(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    asset_catalog
        .and_then(|asset_catalog| asset_catalog.loaded_asset(&AssetKey::new(asset_key)))
        .map(|asset| asset.resolved_path.display().to_string())
        .unwrap_or_default()
}

pub fn asset_kind(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    asset_catalog
        .and_then(|asset_catalog| asset_catalog.prepared_asset(&AssetKey::new(asset_key)))
        .map(|asset| asset.kind.as_str().to_owned())
        .unwrap_or_default()
}

pub fn asset_label(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    asset_catalog
        .and_then(|asset_catalog| asset_catalog.prepared_asset(&AssetKey::new(asset_key)))
        .and_then(|asset| asset.label)
        .unwrap_or_default()
}

pub fn asset_format(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    asset_catalog
        .and_then(|asset_catalog| asset_catalog.prepared_asset(&AssetKey::new(asset_key)))
        .and_then(|asset| asset.format)
        .unwrap_or_default()
}

pub fn asset_failure_reason(asset_catalog: Option<&Arc<AssetCatalog>>, asset_key: &str) -> String {
    asset_catalog
        .and_then(|asset_catalog| asset_catalog.failed_asset(&AssetKey::new(asset_key)))
        .map(|asset| asset.reason)
        .unwrap_or_default()
}
