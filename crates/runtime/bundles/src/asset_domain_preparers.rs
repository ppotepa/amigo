use std::sync::{Arc, RwLock};

use amigo_assets::{AssetCatalog, AssetKey};

pub trait LoadedAssetDomainPreparer: Send + Sync {
    fn name(&self) -> &'static str;

    fn prepare(&self, asset_catalog: &AssetCatalog, asset_key: &AssetKey);

    /// Polls asynchronous domain work without waiting on the host thread.
    /// Synchronous preparers keep the default no-op implementation.
    fn poll(&self, _asset_catalog: &AssetCatalog) {}
}

#[derive(Default)]
pub struct LoadedAssetDomainPreparerRegistry {
    preparers: RwLock<Vec<Arc<dyn LoadedAssetDomainPreparer>>>,
}

impl LoadedAssetDomainPreparerRegistry {
    pub fn register(&self, preparer: Arc<dyn LoadedAssetDomainPreparer>) {
        let mut preparers = self
            .preparers
            .write()
            .expect("loaded asset domain preparer registry lock poisoned");
        if preparers
            .iter()
            .any(|registered| registered.name() == preparer.name())
        {
            return;
        }
        preparers.push(preparer);
    }

    pub fn prepare_all(&self, asset_catalog: &AssetCatalog, asset_key: &AssetKey) {
        let preparers = self
            .preparers
            .read()
            .expect("loaded asset domain preparer registry lock poisoned");
        for preparer in preparers.iter() {
            preparer.prepare(asset_catalog, asset_key);
        }
    }

    pub fn poll_all(&self, asset_catalog: &AssetCatalog) {
        let preparers = self
            .preparers
            .read()
            .expect("loaded asset domain preparer registry lock poisoned");
        for preparer in preparers.iter() {
            preparer.poll(asset_catalog);
        }
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.preparers
            .read()
            .expect("loaded asset domain preparer registry lock poisoned")
            .iter()
            .map(|preparer| preparer.name())
            .collect()
    }
}
