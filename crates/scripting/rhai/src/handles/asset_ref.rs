use std::sync::Arc;

use amigo_assets::AssetCatalog;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::assets::{
    asset_exists, asset_failure_reason, asset_format, asset_kind, asset_label, asset_path,
    asset_source, asset_state, asset_tags,
};
use crate::bindings::commands::queue_asset_reload;

#[derive(Clone)]
pub struct AssetRef {
    asset_catalog: Option<Arc<AssetCatalog>>,
    command_queue: Option<Arc<ScriptCommandQueue>>,
    asset_key: String,
}

impl AssetRef {
    pub fn new(
        asset_catalog: Option<Arc<AssetCatalog>>,
        command_queue: Option<Arc<ScriptCommandQueue>>,
        asset_key: impl Into<String>,
    ) -> Self {
        Self {
            asset_catalog,
            command_queue,
            asset_key: asset_key.into(),
        }
    }

    pub fn key(&mut self) -> String {
        self.asset_key.clone()
    }

    pub fn exists(&mut self) -> bool {
        asset_exists(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn state(&mut self) -> String {
        asset_state(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn source(&mut self) -> String {
        asset_source(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn path(&mut self) -> String {
        asset_path(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn kind(&mut self) -> String {
        asset_kind(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn label(&mut self) -> String {
        asset_label(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn format(&mut self) -> String {
        asset_format(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn tags(&mut self) -> rhai::Array {
        asset_tags(self.asset_catalog.as_ref(), &self.asset_key)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn reason(&mut self) -> String {
        asset_failure_reason(self.asset_catalog.as_ref(), &self.asset_key)
    }

    pub fn reload(&mut self) -> bool {
        queue_asset_reload(self.command_queue.as_ref(), &self.asset_key)
    }
}
