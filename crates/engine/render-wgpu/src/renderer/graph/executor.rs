use amigo_render_api::{FrameGraph, FrameResourceKind};

use crate::renderer::graph::WgpuFrameResourceAllocator;
use crate::renderer::service::WgpuFrameRenderRequest;

#[derive(Default)]
pub(crate) struct WgpuFrameGraphExecutor {
    resources: WgpuFrameResourceAllocator,
}

impl WgpuFrameGraphExecutor {
    pub(crate) fn prepare_transient_resources(
        &mut self,
        graph: &FrameGraph,
        request: &WgpuFrameRenderRequest<'_>,
    ) {
        self.resources.clear();

        for resource in &graph.resources {
            if let FrameResourceKind::TextureColor {
                width,
                height,
                transient: true,
            } = resource.kind
            {
                self.resources.create_color_texture(
                    &request.surface.device,
                    resource.id,
                    &format!("amigo-framegraph-{}", resource.label),
                    width,
                    height,
                    request.surface.config.format,
                );
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resources(&self) -> &WgpuFrameResourceAllocator {
        &self.resources
    }
}
