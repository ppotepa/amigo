use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};

use crate::renderer::collect_material_candidate_2d;
use crate::{
    motion_vector_color,
    WgpuRefractiveMaskAdapterContext, WgpuRefractiveMaskAppendOutcome, WgpuRenderable2dAdapter,
    WgpuRenderable2dAdapterContext, WgpuMotionAdapterContext,
};

pub struct VectorMesh2dRenderableAdapter;

impl WgpuRenderable2dAdapter for VectorMesh2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::VectorMesh
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::VectorMesh(command) = &item.primitive else {
            return false;
        };

        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(
                amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
            ),
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

    fn append_refractive_mask_batches(
        &self,
        ctx: &mut WgpuRefractiveMaskAdapterContext<'_>,
        item: &crate::Renderable2dItem,
        _alpha: f32,
    ) -> WgpuRefractiveMaskAppendOutcome {
        let Some(vector) = item.primitive.vector_mesh() else {
            return WgpuRefractiveMaskAppendOutcome::none();
        };
        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(
                amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
            ),
        );
        crate::renderer::append_vector_primitive_vertices(
            vertices,
            ctx.viewport,
            ctx.camera,
            vector,
            None,
            None,
            None,
        );
        WgpuRefractiveMaskAppendOutcome::appended("vector_coverage", false)
    }

    fn append_motion_batches(
        &self,
        ctx: &mut WgpuMotionAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let Some(vector) = item.primitive.vector_mesh() else {
            return false;
        };
        let transform = crate::renderer::vector_primitive_viewport_fit_transform(
            ctx.viewport,
            vector,
        );
        let key = item.source_id().as_str().to_owned();
        ctx.current_positions
            .insert(key.clone(), transform.translation);
        let color = motion_vector_color(
            ctx.previous_positions.get(&key).copied(),
            transform.translation,
            ctx.target_size,
        );
        crate::renderer::append_vector_primitive_vertices(
            crate::renderer::color_batch_vertices(
                ctx.color_batches,
                crate::renderer::particle_blend_mode(
                    amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
                ),
            ),
            ctx.viewport,
            ctx.camera,
            vector,
            Some(transform),
            Some(color),
            Some(color),
        );
        true
    }
}
