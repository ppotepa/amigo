use amigo_math::{ColorRgba, Vec2};
use amigo_render_api::{ParticleBlendMode2dPrimitive, ParticleLineAnchor2dPrimitive};

use crate::renderer::vertices::{ColorVertex, TextureVertex};

pub(crate) type ParticleBlendMode2d = ParticleBlendMode2dPrimitive;
pub(crate) type ParticleLineAnchor2d = ParticleLineAnchor2dPrimitive;

#[derive(Clone, Copy)]
pub(crate) struct ProjectedPoint {
    pub(crate) position: Vec2,
    pub(crate) depth: f32,
}

#[derive(Clone)]
pub(crate) struct ProjectedTriangle {
    pub(crate) points: [Vec2; 3],
    pub(crate) color: ColorRgba,
    pub(crate) depth: f32,
    pub(crate) render_order: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct TextureUvRect {
    pub(crate) u0: f32,
    pub(crate) v0: f32,
    pub(crate) u1: f32,
    pub(crate) v1: f32,
}

#[derive(Clone)]
pub(crate) struct TextureBatch {
    pub(crate) blend_mode: TextureBlendMode,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) _owned_sampler: Option<wgpu::Sampler>,
    pub(crate) vertices: Vec<TextureVertex>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureBlendMode {
    Opaque,
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

#[derive(Clone)]
pub(crate) struct ColorBatch {
    pub(crate) blend_mode: ParticleBlendMode2d,
    pub(crate) vertices: Vec<ColorVertex>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SpriteSheet {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
    pub fps: f32,
    pub looping: bool,
}

impl SpriteSheet {
    pub(crate) fn visible_frame_count(&self) -> u32 {
        self.frame_count
            .max(1)
            .min(self.columns.max(1).saturating_mul(self.rows.max(1)))
    }
}
