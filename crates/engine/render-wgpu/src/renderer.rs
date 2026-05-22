use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::mem::size_of;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use amigo_2d_composition::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_assets::{AssetCatalog, AssetKey, PreparedAsset, PreparedAssetKind};
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
use amigo_render_api::{
    ParticleBlendMode2dPrimitive, ParticleLineAnchor2dPrimitive, PostFx2d,
    PostFx2dCacheKey, PostFx2dStack, cached_image_post_fx_stack_from_flat_metadata,
};
use amigo_scene::SceneService;
use image::{GenericImageView, RgbaImage};
use wgpu::util::DeviceExt;

use crate::ui_overlay::{
    UiDrawPrimitive, UiOverlayDocument, UiViewportSize, build_ui_overlay_primitives,
};
use crate::Renderable2dItem;
use crate::{WgpuOffscreenTarget, WgpuSurfaceState};

type ParticleBlendMode2d = ParticleBlendMode2dPrimitive;
type ParticleLineAnchor2d = ParticleLineAnchor2dPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayeredImageBlendMode2d {
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

impl LayeredImageBlendMode2d {
    pub(crate) fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "alpha" => Self::Alpha,
            "add" | "additive" => Self::Additive,
            "screen" => Self::Screen,
            "multiply" => Self::Multiply,
            "lighten" => Self::Lighten,
            _ => Self::Alpha,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayeredImageViewportFit2d {
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LayeredImageLayer {
    pub id: String,
    pub label: String,
    pub image: String,
    pub blend_mode: LayeredImageBlendMode2d,
    pub opacity: f32,
    pub color: Option<ColorRgba>,
    pub animation_hint: Option<String>,
    pub post_fx: Option<PostFx2dStack>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LayeredImageAsset {
    pub key: AssetKey,
    pub label: Option<String>,
    pub canvas_size: Vec2,
    pub base_image: String,
    pub layers: Vec<LayeredImageLayer>,
    pub preview_image: Option<String>,
}

pub(crate) trait LayeredImageAssetSource {
    fn layered_image_asset(&self, key: &AssetKey) -> Option<LayeredImageAsset>;
}

impl LayeredImageAssetSource for AssetCatalog {
    fn layered_image_asset(&self, key: &AssetKey) -> Option<LayeredImageAsset> {
        self.prepared_asset(key)
            .and_then(|prepared| infer_layered_image_asset_from_prepared(&prepared))
    }
}

fn infer_layered_image_asset_from_prepared(prepared: &PreparedAsset) -> Option<LayeredImageAsset> {
    if !matches!(prepared.kind, PreparedAssetKind::LayeredImage2d) {
        return None;
    }

    let canvas_width = metadata_f32(prepared, "canvas.width")?;
    let canvas_height = metadata_f32(prepared, "canvas.height")?;
    let base_image = metadata_string(prepared, "base.image")
        .or_else(|| metadata_string(prepared, "base.file"))
        .or_else(|| metadata_string(prepared, "base"))?;

    let mut layers = Vec::new();
    for index in 0..infer_indexed_count(prepared, "layers") {
        let prefix = format!("layers.{index}");
        let id = metadata_string(prepared, &format!("{prefix}.id"))
            .unwrap_or_else(|| format!("layer_{index:03}"));
        let Some(image) = metadata_string(prepared, &format!("{prefix}.image")) else {
            continue;
        };
        let label = metadata_string(prepared, &format!("{prefix}.label"))
            .unwrap_or_else(|| id.clone());
        let blend_mode = metadata_string(prepared, &format!("{prefix}.blend"))
            .map(|value| LayeredImageBlendMode2d::parse(&value))
            .unwrap_or(LayeredImageBlendMode2d::Additive);
        let opacity = metadata_f32(prepared, &format!("{prefix}.default_opacity"))
            .unwrap_or(1.0)
            .clamp(0.0, 4.0);

        layers.push(LayeredImageLayer {
            id,
            label,
            image,
            blend_mode,
            opacity,
            color: metadata_string(prepared, &format!("{prefix}.color"))
                .and_then(|value| parse_hex_rgba(&value)),
            animation_hint: metadata_string(prepared, &format!("{prefix}.animation_hint")),
            post_fx: cached_image_post_fx_stack_from_flat_metadata(
                &prepared.metadata,
                &format!("{prefix}.post_fx"),
            ),
            enabled: metadata_bool(prepared, &format!("{prefix}.enabled")).unwrap_or(true),
        });
    }

    Some(LayeredImageAsset {
        key: prepared.key.clone(),
        label: prepared.label.clone(),
        canvas_size: Vec2::new(canvas_width, canvas_height),
        base_image,
        layers,
        preview_image: metadata_string(prepared, "preview.image"),
    })
}

fn infer_indexed_count(prepared: &PreparedAsset, prefix: &str) -> usize {
    let prefix = format!("{prefix}.");
    prepared
        .metadata
        .keys()
        .filter_map(|key| key.strip_prefix(&prefix))
        .filter_map(|rest| rest.split_once('.').map(|(index, _)| index))
        .filter_map(|index| index.parse::<usize>().ok())
        .max()
        .map_or(0, |index| index + 1)
}

fn metadata_string(prepared: &PreparedAsset, key: &str) -> Option<String> {
    prepared
        .metadata
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_bool(prepared: &PreparedAsset, key: &str) -> Option<bool> {
    prepared.metadata.get(key)?.parse::<bool>().ok()
}

fn parse_hex_rgba(value: &str) -> Option<ColorRgba> {
    let hex = value.trim().trim_start_matches('#');
    let parse = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&hex[range], 16)
            .ok()
            .map(|v| v as f32 / 255.0)
    };

    match hex.len() {
        6 => Some(ColorRgba::new(
            parse(0..2)?,
            parse(2..4)?,
            parse(4..6)?,
            1.0,
        )),
        8 => Some(ColorRgba::new(
            parse(0..2)?,
            parse(2..4)?,
            parse(4..6)?,
            parse(6..8)?,
        )),
        _ => None,
    }
}

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

    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.half_width * 2.0, self.half_height * 2.0)
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
    blend_mode: TextureBlendMode,
    bind_group: wgpu::BindGroup,
    _owned_sampler: Option<wgpu::Sampler>,
    vertices: Vec<TextureVertex>,
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
    blend_mode: ParticleBlendMode2d,
    vertices: Vec<ColorVertex>,
}

impl WgpuSceneRenderer {
    pub fn plate_relight_last_summary(&self) -> &str {
        &self.plate_relight_last_summary
    }

    pub(crate) fn set_plate_relight_last_summary(&mut self, summary: impl Into<String>) {
        self.plate_relight_last_summary = summary.into();
    }

    pub fn render_materials_last_summary(&self) -> &str {
        &self.render_materials_last_summary
    }

    pub(crate) fn set_render_materials_last_summary(&mut self, summary: impl Into<String>) {
        self.render_materials_last_summary = summary.into();
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ParticleRenderLight {
    position: Vec2,
    color: ColorRgba,
    radius: f32,
    intensity: f32,
}

#[derive(Clone)]
pub(crate) struct LightMap2dImageData {
    width: u32,
    height: u32,
    pixels: Arc<Vec<[f32; 4]>>,
}

#[derive(Clone)]
pub(crate) struct LightMap2dLayer {
    image: LightMap2dImageData,
    opacity: f32,
}

#[derive(Clone)]
pub(crate) struct LightMap2dSampler {
    id: String,
    transform: Transform2,
    size: Vec2,
    channels: BTreeMap<String, Vec<LightMap2dLayer>>,
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

pub(crate) struct CachedLightMap2dImage {
    image_path: PathBuf,
    modified_at: Option<SystemTime>,
    data: LightMap2dImageData,
}

impl CachedTextureResource {
    fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self._view
    }
}

mod assets;
mod buffers;
mod glyphs;
mod graph;
mod lightmap2d;
mod math;
mod particles;
mod pipelines;
mod scene;
mod service;
mod text;
mod world_2d;
mod world_3d;

use assets::*;
use buffers::*;
use glyphs::*;
use math::*;
use pipelines::*;
use scene::*;
use text::*;
use world_2d::*;
use world_3d::*;

pub(crate) use particles::color_batch_vertices;
pub(crate) use particles::append_particle_light_primitive_vertices;
pub(crate) use particles::append_particle_primitive_vertices;
pub(crate) use particles::particle_blend_mode;
pub(crate) use particles::particle_render_lights_from_renderables;
pub(crate) use math::sprite_color;
pub(crate) use service::{collect_material_candidate_2d, WgpuMaterialCandidate2d};
pub(crate) use world_2d::append_textured_quad_debug_vertices;
pub(crate) use world_2d::append_tilemap_primitive_fallback_vertices;
pub(crate) use world_2d::append_beacon_vfx_primitive_vertices;
pub(crate) use world_2d::append_vector_primitive_vertices;

pub use service::{
    WgpuEmergencyOverlayLevel, WgpuEmergencyOverlayLine, WgpuFrameRenderRequest,
    WgpuFrameRenderTarget, WgpuGameViewportPlacement, WgpuSceneRenderer, WgpuSurfaceRect,
    WgpuWorld2dRenderInput, WgpuWorld3dRenderInput,
};

#[cfg(test)]
mod tests;
