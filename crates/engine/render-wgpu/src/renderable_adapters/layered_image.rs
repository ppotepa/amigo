use crate::{
    WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext,
};
use amigo_render_api::{RenderPrimitive2d, RenderPrimitive2dKind};
use crate::renderer::collect_material_candidate_2d;

pub struct LayeredImage2dRenderableAdapter;

impl WgpuRenderable2dAdapter for LayeredImage2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::LayeredImage
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::LayeredImage(command) = &item.primitive else {
            return false;
        };

        ctx.renderer.append_layered_image_primitive_texture_batches_filtered(
            ctx.texture_batches,
            ctx.device,
            ctx.queue,
            ctx.assets,
            ctx.viewport,
            ctx.layer_camera,
            command,
            ctx.included_layered_image_parts,
            ctx.excluded_layered_image_parts,
            ctx.include_base_layered_image,
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
