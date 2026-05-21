use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};

use crate::{
    WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext,
};

pub struct Beacon2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Beacon2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::BeaconLight
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::BeaconLight(primitive) = &item.primitive else {
            return false;
        };

        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(amigo_render_api::ParticleBlendMode2dPrimitive::Additive),
        );
        crate::renderer::append_beacon_vfx_primitive_vertices(
            vertices,
            ctx.viewport,
            ctx.layer_camera,
            primitive,
        );
        true
    }
}
