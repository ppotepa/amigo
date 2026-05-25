use crate::renderer::append_tilemap_primitive_color_vertices;
use crate::{WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext};
use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};

pub struct TileBatch2dRenderableAdapter;

impl WgpuRenderable2dAdapter for TileBatch2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::TileBatch
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::TileBatch(command) = &item.primitive else {
            return false;
        };

        if ctx.renderer.append_tilemap_primitive_texture_batch(
            ctx.texture_batches,
            ctx.device,
            ctx.queue,
            ctx.assets,
            ctx.viewport,
            ctx.layer_camera,
            ctx.transform,
            command,
        ) {
            return true;
        }

        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(
                amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
            ),
        );
        append_tilemap_primitive_color_vertices(
            vertices,
            ctx.viewport,
            ctx.layer_camera,
            ctx.transform,
            command,
        );
        true
    }
}
