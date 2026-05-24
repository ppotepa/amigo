use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct WgpuPipelineRegistry {
    pipelines: BTreeMap<&'static str, wgpu::RenderPipeline>,
}

impl WgpuPipelineRegistry {
    pub(crate) fn extend(&mut self, pipelines: BTreeMap<&'static str, wgpu::RenderPipeline>) {
        self.pipelines.extend(pipelines);
    }

    pub(crate) fn get(&self, id: &str) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(id)
    }

    pub(crate) fn required(&self, id: &str) -> &wgpu::RenderPipeline {
        self.get(id)
            .unwrap_or_else(|| panic!("missing WGPU pipeline: {id}"))
    }
}
