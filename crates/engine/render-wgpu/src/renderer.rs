use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use amigo_2d_composition::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
use amigo_render_api::{
    LayeredImageAssetSource, ParticleBlendMode2dPrimitive, ParticleLineAnchor2dPrimitive,
    PostFx2d, PostFx2dCacheKey,
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

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) struct ParticleRenderLight {
    position: Vec2,
    color: ColorRgba,
    radius: f32,
    intensity: f32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct LightMap2dImageData {
    width: u32,
    height: u32,
    pixels: Arc<Vec<[f32; 4]>>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct LightMap2dLayer {
    image: LightMap2dImageData,
    opacity: f32,
}

#[allow(dead_code)]
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
mod shaders;
mod text;
mod vertices;
mod world_2d;
mod world_3d;

use assets::*;
use buffers::*;
use glyphs::*;
use math::*;
use pipelines::*;
use scene::*;
use shaders::*;
use text::*;
use vertices::*;
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
