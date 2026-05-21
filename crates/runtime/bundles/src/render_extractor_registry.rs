use std::collections::BTreeMap;
use std::sync::RwLock;

use amigo_render_api::RuntimeRenderExtractorIdRegistry;
use amigo_runtime::Runtime;

use crate::render_extractor_bridges::{self, WgpuRenderExtractorRegistry};
pub(crate) type WgpuBridgeInstaller = fn(&mut WgpuRenderExtractorRegistry);

#[derive(Default)]
pub(crate) struct WgpuRenderExtractorBridgeRegistry {
    installers: RwLock<BTreeMap<String, WgpuBridgeInstaller>>,
}

impl WgpuRenderExtractorBridgeRegistry {
    pub(crate) fn register(&self, id: impl Into<String>, installer: WgpuBridgeInstaller) {
        self.installers
            .write()
            .expect("render extractor bridge registry poisoned")
            .insert(id.into(), installer);
    }

    fn install(&self, id: &str, registry: &mut WgpuRenderExtractorRegistry) {
        if let Some(install) = self
            .installers
            .read()
            .expect("render extractor bridge registry poisoned")
            .get(id)
            .copied()
        {
            install(registry);
        }
    }
}

pub fn default_wgpu_render_extractor_registry_for_runtime(
    runtime: &Runtime,
) -> WgpuRenderExtractorRegistry {
    let mut registry = WgpuRenderExtractorRegistry::new();
    render_extractor_bridges::register_world_2d_builtin_render_extractors(&mut registry);

    if let (Some(ids), Some(bridges)) = (
        runtime.resolve::<RuntimeRenderExtractorIdRegistry>(),
        runtime.resolve::<WgpuRenderExtractorBridgeRegistry>(),
    ) {
        for id in ids.registered_ids() {
            bridges.install(&id, &mut registry);
        }
    }

    render_extractor_bridges::register_world_3d_render_extractors(&mut registry);
    render_extractor_bridges::register_host_overlay_render_extractors(&mut registry);
    registry
}
