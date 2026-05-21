use crate::{
    WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext,
};
use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};
use crate::renderer::{append_textured_quad_debug_vertices, collect_material_candidate_2d, sprite_color};

pub struct TexturedQuad2dRenderableAdapter;

impl WgpuRenderable2dAdapter for TexturedQuad2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::TexturedQuad
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::TexturedQuad(quad) = &item.primitive else {
            return false;
        };
        let appended = ctx.renderer.append_textured_quad_texture_batch(
            ctx.texture_batches,
            ctx.device,
            ctx.queue,
            ctx.assets,
            ctx.viewport,
            ctx.layer_camera,
            quad,
        );
        if !appended {
            let vertices = crate::renderer::color_batch_vertices(
                ctx.color_batches,
                crate::renderer::particle_blend_mode(amigo_render_api::ParticleBlendMode2dPrimitive::Alpha),
            );
            append_textured_quad_debug_vertices(
                vertices,
                ctx.viewport,
                ctx.layer_camera,
                quad,
                sprite_color(quad.texture.as_str()),
            );
        }
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
