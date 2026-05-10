use std::collections::BTreeMap;

use crate::renderer::service::CachedFontAtlas;
use crate::renderer::{CachedLightMap2dImage, CachedTextureResource};

pub struct WgpuSceneRenderer {
    pub(crate) color_alpha_pipeline: wgpu::RenderPipeline,
    pub(crate) color_additive_pipeline: wgpu::RenderPipeline,
    pub(crate) color_multiply_pipeline: wgpu::RenderPipeline,
    pub(crate) color_screen_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_alpha_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_additive_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_multiply_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_screen_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_lighten_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) texture_cache: BTreeMap<String, CachedTextureResource>,
    pub(crate) lightmap_2d_image_cache: BTreeMap<String, CachedLightMap2dImage>,
    pub(crate) font_atlas_cache: BTreeMap<String, CachedFontAtlas>,
}
