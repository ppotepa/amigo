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
    let sprite = amigo_sprite_2d_plugin::Sprite {
        texture: amigo_assets::AssetKey::new("generated://visual-source/quad"),
        size,
        sheet: None,
        sheet_is_explicit: false,
        animation_override: None,
        visual_maps: None,
        frame_index: 0,
        frame_elapsed: 0.0,
    };
    crate::renderer::world_2d::append_sprite_vertices(
        color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha),
        viewport,
        camera,
        transform,
        &sprite,
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

pub(super) fn tilemap_draw_size(tilemap: &amigo_tilemap_2d_plugin::TileMap2d) -> Vec2 {
    let width = tilemap
        .grid
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(1) as f32
        * tilemap.tile_size.x.max(1.0);
    let height = tilemap.grid.len().max(1) as f32 * tilemap.tile_size.y.max(1.0);
    Vec2::new(width, height)
}
