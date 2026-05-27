use crate::renderer::{
    append_textured_quad_debug_vertices, collect_material_candidate_2d, sprite_color,
};
use crate::{
    WgpuRefractiveMaskAdapterContext, WgpuRefractiveMaskAppendOutcome, WgpuRenderable2dAdapter,
    WgpuRenderable2dAdapterContext, WgpuVisualMapAdapterContext,
};
use amigo_render_api::{
    RenderMaterialBinding2d, RenderPrimitive2d, RenderPrimitive2dKind, TexturedQuad2dPrimitive,
    VisualMaps2dPrimitive, VisualSourceKind2d,
};

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
                crate::renderer::particle_blend_mode(
                    amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
                ),
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

    fn focus_sample_world_position(
        &self,
        item: &crate::Renderable2dItem,
    ) -> Option<amigo_math::Vec2> {
        item.primitive
            .textured_quad()
            .map(|primitive| primitive.transform.translation)
    }

    fn append_visual_map_batches(
        &self,
        ctx: &mut WgpuVisualMapAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let Some(primitive) = item.primitive.textured_quad() else {
            return false;
        };
        let Some(asset) = visual_map_for_kind(primitive.visual_maps.as_ref(), ctx.kind) else {
            return false;
        };
        append_visual_map_sprite_batch(ctx, primitive.transform, asset, primitive.size);
        true
    }

    fn append_refractive_mask_batches(
        &self,
        ctx: &mut WgpuRefractiveMaskAdapterContext<'_>,
        item: &crate::Renderable2dItem,
        _alpha: f32,
    ) -> WgpuRefractiveMaskAppendOutcome {
        let Some(quad) = item.primitive.textured_quad() else {
            return WgpuRefractiveMaskAppendOutcome::none();
        };
        let _ = ctx.renderer.append_textured_quad_texture_batch(
            ctx.texture_batches,
            &ctx.target.device,
            &ctx.target.queue,
            ctx.assets,
            ctx.viewport,
            ctx.camera,
            quad,
        );
        WgpuRefractiveMaskAppendOutcome::appended("texture_alpha", false)
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
