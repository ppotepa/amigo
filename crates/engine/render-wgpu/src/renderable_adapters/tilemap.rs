use crate::{
    WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext,
};
use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};
use crate::renderer::append_tilemap_primitive_fallback_vertices;

pub struct TileMap2dRenderableAdapter;

impl WgpuRenderable2dAdapter for TileMap2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::TileMap
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::TileMap(command) = &item.primitive else {
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
            crate::renderer::particle_blend_mode(amigo_render_api::ParticleBlendMode2dPrimitive::Alpha),
        );
        append_tilemap_primitive_fallback_vertices(
            vertices,
            ctx.viewport,
            ctx.layer_camera,
            ctx.transform,
            command,
        );
        true
    }
}
