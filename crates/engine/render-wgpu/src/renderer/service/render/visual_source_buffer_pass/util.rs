use super::*;

pub(super) fn take_or_create_target(
    slot: &mut Option<WgpuOffscreenTarget>,
    template: &WgpuOffscreenTarget,
    label: &'static str,
) -> WgpuOffscreenTarget {
    slot.take().unwrap_or_else(|| {
        super::super::offscreen_ops::compatible_offscreen_target(template, label)
    })
}

pub(super) fn append_visual_quad(
    color_batches: &mut Vec<ColorBatch>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    size: Vec2,
    color: ColorRgba,
) {
    crate::renderer::world_2d::append_textured_quad_debug_vertices(
        color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha),
        viewport,
        camera,
        &amigo_render_api::TexturedQuad2dPrimitive {
            texture: amigo_assets::AssetKey::new("generated://visual-source/quad"),
            size,
            transform,
            sheet: None,
            frame_index: 0,
            visual_maps: None,
            material: amigo_render_api::RenderMaterialBinding2d::none(
                amigo_material_api::MaterialCoverageKind2d::TextureAlpha,
            ),
        },
        color,
    );
}

pub(super) fn color_to_wgpu(color: ColorRgba) -> wgpu::Color {
    wgpu::Color {
        r: color.r as f64,
        g: color.g as f64,
        b: color.b as f64,
        a: color.a as f64,
    }
}
