use std::collections::BTreeMap;

use crate::renderer::TextureVertex;
use crate::renderer::pipelines::create_color_pipeline;

mod camera_exposure;
mod camera_optics;
mod color_quantize;
mod crt;
mod dirty_bloom;
mod downscale;
mod film_emulsion;
mod film_noise;
mod focus_blur;
mod highlight_extract;
mod plate_relight;
mod refractive_material;
mod scan_output;
mod shutter_blur;
mod wet_reflections;

pub(crate) use camera_exposure::CameraExposurePipelineProvider;
pub(crate) use camera_optics::CameraOpticsPipelineProvider;
pub(crate) use color_quantize::ColorQuantizePipelineProvider;
pub(crate) use crt::CrtPipelineProvider;
pub(crate) use dirty_bloom::DirtyBloomPipelineProvider;
pub(crate) use downscale::DownscalePipelineProvider;
pub(crate) use film_emulsion::FilmEmulsionPipelineProvider;
pub(crate) use film_noise::FilmNoisePipelineProvider;
pub(crate) use focus_blur::FocusBlurPipelineProvider;
pub(crate) use highlight_extract::HighlightExtractPipelineProvider;
pub(crate) use plate_relight::PlateRelightPipelineProvider;
pub(crate) use refractive_material::RefractiveMaterialPipelineProvider;
pub(crate) use scan_output::ScanOutputPipelineProvider;
pub(crate) use shutter_blur::ShutterBlurPipelineProvider;
pub(crate) use wet_reflections::WetReflectionsPipelineProvider;

pub(crate) struct WgpuPostFxPipelineCreateContext<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) camera_visual_source_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) focus_blur_texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) shutter_blur_texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) wet_reflections_texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) wet_reflections_uniform_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) wet_reflections_pipeline_layout: &'a wgpu::PipelineLayout,
    pub(crate) focus_blur_pipeline_layout: &'a wgpu::PipelineLayout,
    pub(crate) camera_exposure_shader: &'a wgpu::ShaderModule,
    pub(crate) camera_optics_shader: &'a wgpu::ShaderModule,
    pub(crate) focus_blur_shader: &'a wgpu::ShaderModule,
    pub(crate) film_emulsion_shader: &'a wgpu::ShaderModule,
    pub(crate) film_noise_shader: &'a wgpu::ShaderModule,
    pub(crate) scan_output_shader: &'a wgpu::ShaderModule,
    pub(crate) color_quantize_shader: &'a wgpu::ShaderModule,
    pub(crate) downscale_shader: &'a wgpu::ShaderModule,
    pub(crate) shutter_blur_shader: &'a wgpu::ShaderModule,
    pub(crate) dirty_bloom_shader: &'a wgpu::ShaderModule,
    pub(crate) highlight_extract_shader: &'a wgpu::ShaderModule,
    pub(crate) crt_shader: &'a wgpu::ShaderModule,
    pub(crate) wet_reflections_shader: &'a wgpu::ShaderModule,
    pub(crate) plate_relight_shader: &'a wgpu::ShaderModule,
    pub(crate) refractive_material_shader: &'a wgpu::ShaderModule,
}

pub(crate) trait WgpuPostFxPipelineProvider {
    fn pipeline_id(&self) -> &'static str;

    fn create_pipeline(
        &self,
        ctx: &WgpuPostFxPipelineCreateContext<'_>,
    ) -> wgpu::RenderPipeline;
}

#[derive(Default)]
pub(crate) struct WgpuPostFxPipelineRegistry {
    pipelines: BTreeMap<&'static str, wgpu::RenderPipeline>,
}

impl WgpuPostFxPipelineRegistry {
    pub(crate) fn register(
        &mut self,
        provider: impl WgpuPostFxPipelineProvider,
        ctx: &WgpuPostFxPipelineCreateContext<'_>,
    ) {
        self.pipelines
            .insert(provider.pipeline_id(), provider.create_pipeline(ctx));
    }

    pub(crate) fn into_pipelines(self) -> BTreeMap<&'static str, wgpu::RenderPipeline> {
        self.pipelines
    }
}

pub(crate) fn build_default_post_fx_pipelines(
    ctx: &WgpuPostFxPipelineCreateContext<'_>,
) -> BTreeMap<&'static str, wgpu::RenderPipeline> {
    let mut registry = WgpuPostFxPipelineRegistry::default();
    registry.register(CameraExposurePipelineProvider, ctx);
    registry.register(CameraOpticsPipelineProvider, ctx);
    registry.register(FocusBlurPipelineProvider, ctx);
    registry.register(FilmEmulsionPipelineProvider, ctx);
    registry.register(FilmNoisePipelineProvider, ctx);
    registry.register(ScanOutputPipelineProvider, ctx);
    registry.register(ColorQuantizePipelineProvider, ctx);
    registry.register(DownscalePipelineProvider, ctx);
    registry.register(ShutterBlurPipelineProvider, ctx);
    registry.register(DirtyBloomPipelineProvider, ctx);
    registry.register(HighlightExtractPipelineProvider, ctx);
    registry.register(CrtPipelineProvider, ctx);
    registry.register(WetReflectionsPipelineProvider, ctx);
    registry.register(PlateRelightPipelineProvider, ctx);
    registry.register(RefractiveMaterialPipelineProvider, ctx);
    registry.into_pipelines()
}

pub(super) fn create_copy_blend_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    label: &'static str,
) -> wgpu::RenderPipeline {
    create_color_pipeline(
        device,
        shader,
        layout,
        format,
        label,
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        },
        &[TextureVertex::layout()],
    )
}
