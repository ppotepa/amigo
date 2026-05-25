use std::collections::{BTreeMap, BTreeSet};

use amigo_math::Vec2;

use crate::renderer::service::pipeline_registry::WgpuPipelineRegistry;
use crate::renderer::service::{CachedFontAtlas, WgpuEmergencyOverlayLine};
use crate::renderer::{CachedLightMap2dImage, CachedTextureResource};

pub(crate) const POST_FX_EXECUTOR_BLUR: &str = "screen_space.blur";
pub(crate) const POST_FX_EXECUTOR_CAMERA_EXPOSURE: &str = "screen_space.camera_exposure";
pub(crate) const POST_FX_EXECUTOR_CAMERA_OPTICS: &str = "screen_space.camera_optics";
pub(crate) const POST_FX_EXECUTOR_COLOR_QUANTIZE: &str = "screen_space.color_quantize";
pub(crate) const POST_FX_EXECUTOR_COLOR_RAMP: &str = "screen_space.color_ramp";
pub(crate) const POST_FX_EXECUTOR_CRT: &str = "screen_space.crt";
pub(crate) const POST_FX_EXECUTOR_DIRTY_BLOOM: &str = "screen_space.dirty_bloom";
pub(crate) const POST_FX_EXECUTOR_DOWNSCALE: &str = "screen_space.downscale";
pub(crate) const POST_FX_EXECUTOR_EMBOSSED_EDGES: &str = "screen_space.embossed_edges";
pub(crate) const POST_FX_EXECUTOR_FILM_EMULSION: &str = "screen_space.film_emulsion";
pub(crate) const POST_FX_EXECUTOR_FILM_NOISE: &str = "screen_space.film_noise";
pub(crate) const POST_FX_EXECUTOR_FOCUS_BLUR: &str = "screen_space.focus_blur";
pub(crate) const POST_FX_EXECUTOR_LENS_DROPLETS: &str = "screen_space.lens_droplets";
pub(crate) const POST_FX_EXECUTOR_RAIN_GLASS: &str = "screen_space.rain_glass";
pub(crate) const POST_FX_EXECUTOR_SCAN_OUTPUT: &str = "screen_space.scan_output";
pub(crate) const POST_FX_EXECUTOR_SHUTTER_BLUR: &str = "screen_space.shutter_blur";
pub(crate) const POST_FX_EXECUTOR_WET_REFLECTIONS: &str = "screen_space.wet_reflections";
pub(crate) const POST_FX_AUX_HIGHLIGHT_EXTRACT: &str = "aux.highlight_extract";
pub(crate) const POST_FX_AUX_PLATE_RELIGHT: &str = "aux.plate_relight";
pub(crate) const POST_FX_AUX_REFRACTIVE_MATERIAL: &str = "aux.refractive_material";

pub struct WgpuSceneRenderer {
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) camera_visual_source_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) focus_blur_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) shutter_blur_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) wet_reflections_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) wet_reflections_uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) pipelines: WgpuPipelineRegistry,
    pub(crate) post_fx_executors: crate::renderer::service::post_fx::WgpuPostFxExecutorRegistry,
    pub(crate) shutter_blur_runtimes: BTreeMap<
        crate::renderer::service::post_fx::runtime_key::PostFxRuntimeKey,
        crate::renderer::service::post_fx::shutter_blur::ShutterBlurRuntime,
    >,
    pub(crate) rain_glass_runtimes: BTreeMap<
        crate::renderer::service::post_fx::runtime_key::PostFxRuntimeKey,
        crate::renderer::service::post_fx::rain_glass::RainGlassRenderRuntime,
    >,
    pub(crate) texture_cache: BTreeMap<String, CachedTextureResource>,
    pub(crate) lightmap_2d_image_cache: BTreeMap<String, CachedLightMap2dImage>,
    pub(crate) font_atlas_cache: BTreeMap<String, CachedFontAtlas>,
    pub(crate) font_missing_glyph_warnings: BTreeSet<String>,
    pub(crate) frame_graph_executor: crate::renderer::graph::WgpuFrameGraphExecutor,
    pub(crate) emergency_overlay_lines: Vec<WgpuEmergencyOverlayLine>,
    pub(crate) visual_source_targets_2d: crate::renderer::service::WgpuVisualSourceTargets2d,
    pub(crate) visual_source_previous_positions_2d: BTreeMap<String, Vec2>,
    pub(crate) plate_relight_last_summary: String,
    pub(crate) render_materials_last_summary: String,
    pub(crate) frame_diagnostics: Vec<amigo_render_api::RenderFrameDiagnostic>,
}

impl WgpuSceneRenderer {
    pub(crate) fn pipeline(&self, id: &str) -> &wgpu::RenderPipeline {
        self.pipelines.required(id)
    }

    pub(crate) fn post_fx_pipeline(&self, id: &str) -> &wgpu::RenderPipeline {
        self.pipeline(id)
    }
}
