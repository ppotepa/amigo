use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};

use crate::{
    WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext,
};
use crate::renderer::collect_material_candidate_2d;

pub struct Vector2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Vector2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::VectorShape
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::VectorShape(command) = &item.primitive else {
            return false;
        };

        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(amigo_render_api::ParticleBlendMode2dPrimitive::Alpha),
        );
        crate::renderer::append_vector_primitive_vertices(
            vertices,
            ctx.viewport,
            ctx.layer_camera,
            command,
            Some(ctx.transform),
            None,
            None,
        );
        collect_material_candidate_2d(
            item,
            ctx.layer_camera,
            ctx.layer_opacity,
            ctx.material_candidates,
            ctx.material_decisions,
        );
        true
    }
}
