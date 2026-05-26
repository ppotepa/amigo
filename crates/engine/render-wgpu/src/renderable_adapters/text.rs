use amigo_math::Vec2;
use amigo_render_api::{GlyphRun2dBlendMode, RenderPrimitive2d, RenderPrimitive2dKind};

use crate::renderer::collect_material_candidate_2d;
use crate::{
    WgpuRefractiveMaskAdapterContext, WgpuRefractiveMaskAppendOutcome, WgpuRenderable2dAdapter,
    WgpuRenderable2dAdapterContext,
};

pub struct Text2dRenderableAdapter;

impl WgpuRenderable2dAdapter for Text2dRenderableAdapter {
    fn kind(&self) -> RenderPrimitive2dKind {
        RenderPrimitive2dKind::GlyphRun
    }

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &crate::Renderable2dItem,
    ) -> bool {
        let RenderPrimitive2d::GlyphRun(command) = &item.primitive else {
            return false;
        };
        if command.color.a <= 0.001 {
            return true;
        }

        if let Some(glow) = command.glow {
            let passes = glow.passes.max(1);
            let step = glow.radius.max(0.0) / passes as f32;
            for pass in 1..=passes {
                let radius = pass as f32 * step;
                let alpha = glow.intensity.max(0.0) / pass as f32;
                for (dx, dy) in text2d_effect_offsets(radius) {
                    let glow_transform = translated_transform2(ctx.transform, Vec2::new(dx, dy));
                    let color = color_with_alpha_mul(glow.color, alpha);
                    let _ = ctx.renderer.append_text2d_ttf_font_texture_batch(
                        ctx.texture_batches,
                        ctx.device,
                        ctx.queue,
                        ctx.assets,
                        ctx.viewport,
                        ctx.layer_camera,
                        &command.font,
                        &command.text,
                        glow_transform,
                        command.bounds,
                        command.font_size,
                        color,
                    );
                }
            }
        }

        if let Some(outline) = command.outline {
            let width = outline.width.max(0.0);
            if width > 0.0 {
                for (dx, dy) in text2d_effect_offsets(width) {
                    let outline_transform = translated_transform2(ctx.transform, Vec2::new(dx, dy));
                    let _ = ctx.renderer.append_text2d_ttf_font_texture_batch(
                        ctx.texture_batches,
                        ctx.device,
                        ctx.queue,
                        ctx.assets,
                        ctx.viewport,
                        ctx.layer_camera,
                        &command.font,
                        &command.text,
                        outline_transform,
                        command.bounds,
                        command.font_size,
                        outline.color,
                    );
                }
            }
        }

        if let Some(shadow) = command.shadow {
            let shadow_transform = translated_transform2(ctx.transform, shadow.offset);
            let _ = ctx.renderer.append_text2d_ttf_font_texture_batch(
                ctx.texture_batches,
                ctx.device,
                ctx.queue,
                ctx.assets,
                ctx.viewport,
                ctx.layer_camera,
                &command.font,
                &command.text,
                shadow_transform,
                command.bounds,
                command.font_size,
                shadow.color,
            );
        }

        let _blend = match command.blend {
            GlyphRun2dBlendMode::Alpha => GlyphRun2dBlendMode::Alpha,
            GlyphRun2dBlendMode::Additive => GlyphRun2dBlendMode::Additive,
            GlyphRun2dBlendMode::Multiply => GlyphRun2dBlendMode::Multiply,
            GlyphRun2dBlendMode::Screen => GlyphRun2dBlendMode::Screen,
        };

        let _ = ctx.renderer.append_text2d_ttf_font_texture_batch(
            ctx.texture_batches,
            ctx.device,
            ctx.queue,
            ctx.assets,
            ctx.viewport,
            ctx.layer_camera,
            &command.font,
            &command.text,
            ctx.transform,
            command.bounds,
            command.font_size,
            command.color,
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

    fn focus_sample_world_position(&self, item: &crate::Renderable2dItem) -> Option<Vec2> {
        item.primitive
            .glyph_run()
            .map(|primitive| primitive.transform.translation)
    }

    fn append_refractive_mask_batches(
        &self,
        ctx: &mut WgpuRefractiveMaskAdapterContext<'_>,
        item: &crate::Renderable2dItem,
        layer_opacity: f32,
    ) -> WgpuRefractiveMaskAppendOutcome {
        let Some(glyph) = item.primitive.glyph_run() else {
            return WgpuRefractiveMaskAppendOutcome::none();
        };
        let alpha = (glyph.color.a * layer_opacity).clamp(0.0, 1.0);
        if ctx.renderer.append_text2d_ttf_font_texture_batch(
            ctx.texture_batches,
            &ctx.target.device,
            &ctx.target.queue,
            ctx.assets,
            ctx.viewport,
            ctx.camera,
            &glyph.font,
            &glyph.text,
            glyph.transform,
            glyph.bounds,
            glyph.font_size,
            amigo_math::ColorRgba::new(1.0, 1.0, 1.0, alpha),
        ) {
            return WgpuRefractiveMaskAppendOutcome::appended("ttf_font", false);
        }

        let vertices = crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(
                amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
            ),
        );
        crate::renderer::append_text_2d_vertices(
            vertices,
            ctx.viewport,
            ctx.camera,
            &glyph.text,
            glyph.transform,
            glyph.bounds,
            amigo_math::ColorRgba::new(1.0, 1.0, 1.0, alpha),
        );
        WgpuRefractiveMaskAppendOutcome::appended("generated_geometry", true)
    }
}

fn translated_transform2(
    transform: amigo_math::Transform2,
    offset: Vec2,
) -> amigo_math::Transform2 {
    amigo_math::Transform2 {
        translation: Vec2::new(
            transform.translation.x + offset.x,
            transform.translation.y + offset.y,
        ),
        ..transform
    }
}

fn color_with_alpha_mul(color: amigo_math::ColorRgba, alpha_mul: f32) -> amigo_math::ColorRgba {
    let mut color = color;
    color.a = (color.a * alpha_mul).clamp(0.0, 1.0);
    color
}

fn text2d_effect_offsets(radius: f32) -> Vec<(f32, f32)> {
    let radius = radius.max(0.0);
    if radius <= 0.001 {
        return vec![(0.0, 0.0)];
    }

    let diagonal = radius * 0.70710677;
    vec![
        (-radius, 0.0),
        (radius, 0.0),
        (0.0, -radius),
        (0.0, radius),
        (-diagonal, -diagonal),
        (-diagonal, diagonal),
        (diagonal, -diagonal),
        (diagonal, diagonal),
    ]
}
