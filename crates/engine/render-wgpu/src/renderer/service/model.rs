use std::collections::{BTreeMap, BTreeSet};

use crate::renderer::service::{CachedFontAtlas, WgpuEmergencyOverlayLine};
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
    pub(crate) focus_blur_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) shutter_blur_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) wet_reflections_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) wet_reflections_uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) wet_reflections_pipeline: wgpu::RenderPipeline,
    pub(crate) dirty_bloom_pipeline: wgpu::RenderPipeline,
    pub(crate) color_quantize_pipeline: wgpu::RenderPipeline,
    pub(crate) downscale_pipeline: wgpu::RenderPipeline,
    pub(crate) camera_exposure_pipeline: wgpu::RenderPipeline,
    pub(crate) shutter_blur_pipeline: wgpu::RenderPipeline,
    pub(crate) shutter_blur_runtimes: BTreeMap<
        crate::renderer::service::post_fx::runtime_key::PostFxRuntimeKey,
        crate::renderer::service::post_fx::shutter_blur::ShutterBlurRuntime,
    >,
    pub(crate) rain_glass_runtimes: BTreeMap<
        crate::renderer::service::post_fx::runtime_key::PostFxRuntimeKey,
        crate::renderer::service::post_fx::rain_glass::RainGlassRenderRuntime,
    >,
    pub(crate) camera_optics_pipeline: wgpu::RenderPipeline,
    pub(crate) focus_blur_pipeline: wgpu::RenderPipeline,
    pub(crate) film_emulsion_pipeline: wgpu::RenderPipeline,
    pub(crate) film_noise_pipeline: wgpu::RenderPipeline,
    pub(crate) scan_output_pipeline: wgpu::RenderPipeline,
    pub(crate) crt_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_cache: BTreeMap<String, CachedTextureResource>,
    pub(crate) lightmap_2d_image_cache: BTreeMap<String, CachedLightMap2dImage>,
    pub(crate) font_atlas_cache: BTreeMap<String, CachedFontAtlas>,
    pub(crate) font_fallback_warnings: BTreeSet<String>,
    pub(crate) frame_graph_executor: crate::renderer::graph::WgpuFrameGraphExecutor,
    pub(crate) emergency_overlay_lines: Vec<WgpuEmergencyOverlayLine>,
}
