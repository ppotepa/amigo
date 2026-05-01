use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::AssetCatalog;

pub struct AssetsPlugin;

impl RuntimePlugin for AssetsPlugin {
    fn name(&self) -> &'static str {
        "amigo-assets"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(AssetCatalog::default())
    }
}
