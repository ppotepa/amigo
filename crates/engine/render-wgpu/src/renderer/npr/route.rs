#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NprMeshRenderRoute {
    GpuRealtime,
    CpuReference,
}

pub(crate) fn npr_mesh_render_route(
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprMeshRenderRoute {
    match settings.render_strategy {
        amigo_render_api::NprRenderStrategy3d::GpuRealtime => NprMeshRenderRoute::GpuRealtime,
        amigo_render_api::NprRenderStrategy3d::CpuReference => NprMeshRenderRoute::CpuReference,
    }
}
