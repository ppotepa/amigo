use crate::renderer::collect_material_candidate_2d;
use crate::{WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext};
use amigo_render_api::{
    ParticleBlendMode2dPrimitive, ParticleLightMode2dPrimitive, RenderPrimitive2d,
    RenderPrimitive2dKind,
};

pub struct ParticleBatch2dRenderableAdapter;

impl WgpuRenderable2dAdapter for ParticleBatch2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::ParticleBatch
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::ParticleBatch(primitive) = &item.primitive else {
            return false;
        };

        if primitive.light.is_some_and(|light| {
            light.glow && matches!(light.mode, ParticleLightMode2dPrimitive::Particle)
        }) {
            let vertices = crate::renderer::color_batch_vertices(
                ctx.color_batches,
                crate::renderer::particle_blend_mode(ParticleBlendMode2dPrimitive::Additive),
            );
            crate::renderer::append_particle_light_primitive_vertices(
                vertices,
                ctx.viewport,
                ctx.layer_camera,
                primitive,
            );
        }

        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(primitive.blend_mode),
        );
        crate::renderer::append_particle_primitive_vertices(
            vertices,
            ctx.viewport,
            ctx.layer_camera,
            primitive,
            ctx.particle_lights,
            ctx.lightmap_samplers,
            ctx.light_sources,
            ctx.light_routes,
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
