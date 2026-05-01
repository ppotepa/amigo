use std::collections::BTreeMap;

use crate::renderer::CachedTextureResource;

pub struct WgpuSceneRenderer {
    pub(crate) color_alpha_pipeline: wgpu::RenderPipeline,
    pub(crate) color_additive_pipeline: wgpu::RenderPipeline,
    pub(crate) color_multiply_pipeline: wgpu::RenderPipeline,
    pub(crate) color_screen_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) texture_cache: BTreeMap<String, CachedTextureResource>,
}
