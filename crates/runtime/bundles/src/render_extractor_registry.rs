use std::collections::BTreeMap;
use std::sync::RwLock;

use amigo_render_api::RuntimeRenderExtractorIdRegistry;
use amigo_runtime::Runtime;

use crate::render_extractor_bridges::{self, WgpuRenderExtractorRegistry};
pub(crate) type WgpuBridgeInstaller = fn(&mut WgpuRenderExtractorRegistry);

pub(crate) struct WgpuRenderExtractorBridgeInstaller {
    pub(crate) extractor_id: &'static str,
    pub(crate) install: WgpuBridgeInstaller,
}

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

    pub(crate) fn register_installer(&self, installer: WgpuRenderExtractorBridgeInstaller) {
        self.register(installer.extractor_id, installer.install);
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

    if let (Some(ids), Some(bridges)) = (
        runtime.resolve::<RuntimeRenderExtractorIdRegistry>(),
        runtime.resolve::<WgpuRenderExtractorBridgeRegistry>(),
    ) {
        for id in ids.registered_ids() {
            bridges.install(&id, &mut registry);
        }
    }

    render_extractor_bridges::register_world_2d_builtin_render_extractors(&mut registry);
    render_extractor_bridges::register_world_3d_render_extractors(&mut registry);
    render_extractor_bridges::register_host_overlay_render_extractors(&mut registry);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_render_api::RenderFrameExtractor;
    use amigo_render_wgpu::WgpuRenderFramePacket;
    use amigo_runtime::RuntimeBuilder;

    struct TestExtractor;

    impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for TestExtractor {
        fn name(&self) -> &'static str {
            "test.render.extractor"
        }

        fn extract(&self, _context: &Runtime, _packet: &mut WgpuRenderFramePacket) {}
    }

    fn register_test_extractor(registry: &mut WgpuRenderExtractorRegistry) {
        registry.register(TestExtractor);
    }

    #[test]
    fn runtime_registry_installs_plugin_extractors_from_registered_ids() {
        let baseline_runtime = RuntimeBuilder::default()
            .with_service(RuntimeRenderExtractorIdRegistry::default())
            .expect("should register extractor id registry")
            .with_service(WgpuRenderExtractorBridgeRegistry::default())
            .expect("should register bridge registry")
            .build();
        let baseline_len =
            default_wgpu_render_extractor_registry_for_runtime(&baseline_runtime).len();

        let ids = RuntimeRenderExtractorIdRegistry::default();
        ids.register("test.extractor");
        let bridges = WgpuRenderExtractorBridgeRegistry::default();
        bridges.register("test.extractor", register_test_extractor);

        let runtime = RuntimeBuilder::default()
            .with_service(ids)
            .expect("should register extractor id registry")
            .with_service(bridges)
            .expect("should register bridge registry")
            .build();

        let registry = default_wgpu_render_extractor_registry_for_runtime(&runtime);
        assert_eq!(registry.len(), baseline_len + 1);
    }
}
