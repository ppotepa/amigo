use crate::renderer::collect_material_candidate_2d;
use crate::{WgpuRenderable2dAdapter, WgpuRenderable2dAdapterContext, WgpuVisualMapAdapterContext};
use amigo_render_api::{
    RenderMaterialBinding2d, RenderPrimitive2d, RenderPrimitive2dKind, TexturedQuad2dPrimitive,
    VisualMaps2dPrimitive, VisualSourceKind2d,
};

pub struct LayeredTexturedQuads2dRenderableAdapter;

impl WgpuRenderable2dAdapter for LayeredTexturedQuads2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::LayeredTexturedQuads
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::LayeredTexturedQuads(command) = &item.primitive else {
            return false;
        };

        ctx.renderer
            .append_layered_image_primitive_texture_batches_filtered(
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

    fn focus_sample_world_position(
        &self,
        item: &crate::Renderable2dItem,
    ) -> Option<amigo_math::Vec2> {
        item.primitive
            .layered_textured_quads()
            .map(|primitive| primitive.transform.translation)
    }

    fn append_visual_map_batches(
        &self,
        ctx: &mut WgpuVisualMapAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let Some(primitive) = item.primitive.layered_textured_quads() else {
            return false;
        };
        let mut appended = false;
        if let Some(asset) = visual_map_for_kind(primitive.visual_maps.as_ref(), ctx.kind) {
            append_visual_map_sprite_batch(ctx, primitive.transform, asset, primitive.size);
            appended = true;
        }
        for override_ in &primitive.layer_overrides {
            let Some(asset) = visual_map_for_kind(override_.visual_maps.as_ref(), ctx.kind) else {
                continue;
            };
            append_visual_map_sprite_batch(ctx, primitive.transform, asset, primitive.size);
            appended = true;
        }
        appended
    }
}

fn append_visual_map_sprite_batch(
    ctx: &mut WgpuVisualMapAdapterContext<'_>,
    transform: amigo_math::Transform2,
    asset: &amigo_assets::AssetKey,
    size: amigo_math::Vec2,
) {
    ctx.renderer.append_textured_quad_texture_batch(
        ctx.texture_batches,
        &ctx.target.device,
        &ctx.target.queue,
        ctx.assets,
        ctx.viewport,
        ctx.camera,
        &TexturedQuad2dPrimitive {
            texture: asset.clone(),
            size,
            transform,
            visual_maps: None,
            sheet: None,
            frame_index: 0,
            material: RenderMaterialBinding2d::none(
                amigo_material_api::MaterialCoverageKind2d::TextureAlpha,
            ),
        },
    );
}

fn visual_map_for_kind(
    maps: Option<&VisualMaps2dPrimitive>,
    kind: VisualSourceKind2d,
) -> Option<&amigo_assets::AssetKey> {
    let maps = maps?;
    match kind {
        VisualSourceKind2d::SceneNormal => maps.normal.as_ref(),
        VisualSourceKind2d::SceneWetness => maps.wetness.as_ref(),
        VisualSourceKind2d::SceneHighlight => maps.highlight.as_ref(),
        VisualSourceKind2d::SceneEmissive => maps.emissive.as_ref(),
        _ => None,
    }
}
