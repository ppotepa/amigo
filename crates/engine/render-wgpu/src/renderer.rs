use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::mem::size_of;
use std::path::PathBuf;
use std::time::SystemTime;

use amigo_2d_particles::{
    Particle2dDrawCommand, ParticleBlendMode2d, ParticleLightMode2d, ParticleLineAnchor2d,
    ParticleShape2d,
};
use amigo_2d_sprite::{Sprite, SpriteSceneService, SpriteSheet};
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::{TileMap2d, TileMap2dSceneService};
use amigo_2d_vector::{VectorSceneService, VectorShape2d, VectorShapeKind2d, VectorStyle2d};
use amigo_3d_material::{MaterialDrawCommand, MaterialSceneService};
use amigo_3d_mesh::{MeshDrawCommand, MeshSceneService};
use amigo_3d_text::{Text3dDrawCommand, Text3dSceneService};
use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
use amigo_scene::SceneService;
use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::{WgpuOffscreenTarget, WgpuSurfaceState};
use crate::ui_overlay::{
    UiDrawPrimitive, UiOverlayDocument, UiViewportSize, build_ui_overlay_primitives,
};

const COLOR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

const TEXTURE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(color_texture, color_sampler, input.uv) * input.color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ColorVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl ColorVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4
    ];

    pub(crate) fn new(position: Vec2, color: ColorRgba) -> Self {
        Self {
            position: [position.x, position.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TextureVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl TextureVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4
    ];

    pub(crate) fn new(position: Vec2, uv: Vec2, color: ColorRgba) -> Self {
        Self {
            position: [position.x, position.y],
            uv: [uv.x, uv.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }

    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<TextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Viewport {
    half_width: f32,
    half_height: f32,
    aspect: f32,
}

impl Viewport {
    pub(crate) fn from_surface(surface: &WgpuSurfaceState) -> Self {
        let width = surface.config.width.max(1) as f32;
        let height = surface.config.height.max(1) as f32;
        Self::from_dimensions(width, height)
    }

    pub(crate) fn from_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::from_dimensions(target.width.max(1) as f32, target.height.max(1) as f32)
    }

    pub(crate) fn from_dimensions(width: f32, height: f32) -> Self {
        Self {
            half_width: width * 0.5,
            half_height: height * 0.5,
            aspect: width / height,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ProjectedPoint {
    position: Vec2,
    depth: f32,
}

#[derive(Clone)]
pub(crate) struct ProjectedTriangle {
    points: [Vec2; 3],
    color: ColorRgba,
    depth: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct TextureUvRect {
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

#[derive(Clone)]
pub(crate) struct TextureBatch {
    bind_group: wgpu::BindGroup,
    vertices: Vec<TextureVertex>,
}

#[derive(Clone)]
pub(crate) struct ColorBatch {
    blend_mode: ParticleBlendMode2d,
    vertices: Vec<ColorVertex>,
}

#[derive(Clone, Copy)]
pub(crate) struct ParticleRenderLight {
    position: Vec2,
    color: ColorRgba,
    radius: f32,
    intensity: f32,
}

#[derive(Clone)]
pub(crate) enum World2dItem {
    TileMap(amigo_2d_tilemap::TileMap2dDrawCommand),
    Vector(amigo_2d_vector::VectorShape2dDrawCommand),
    Sprite(amigo_2d_sprite::SpriteDrawCommand),
    Particle(Particle2dDrawCommand),
}

pub(crate) struct CachedTextureResource {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    image_path: PathBuf,
    modified_at: Option<SystemTime>,
    width: u32,
    height: u32,
}

impl CachedTextureResource {
    fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

mod assets;
mod buffers;
mod math;
mod particles;
mod pipelines;
mod scene;
mod service;
mod text;
mod world_2d;
mod world_3d;
mod glyphs;

use assets::*;
use buffers::*;
use glyphs::*;
use math::*;
use particles::*;
use pipelines::*;
use scene::*;
use text::*;
use world_2d::*;
use world_3d::*;

pub use service::WgpuSceneRenderer;

#[cfg(test)]
mod tests;
