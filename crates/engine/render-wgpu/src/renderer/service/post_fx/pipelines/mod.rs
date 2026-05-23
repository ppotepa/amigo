use std::collections::BTreeMap;

use crate::renderer::TextureVertex;
use crate::renderer::pipelines::create_color_pipeline;

mod camera_exposure;
mod camera_optics;
mod film_emulsion;
mod film_noise;
mod focus_blur;
mod scan_output;

pub(crate) use camera_exposure::CameraExposurePipelineProvider;
pub(crate) use camera_optics::CameraOpticsPipelineProvider;
pub(crate) use film_emulsion::FilmEmulsionPipelineProvider;
pub(crate) use film_noise::FilmNoisePipelineProvider;
pub(crate) use focus_blur::FocusBlurPipelineProvider;
pub(crate) use scan_output::ScanOutputPipelineProvider;

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

pub(crate) struct ColorQuantizePipelineProvider;
pub(crate) struct DownscalePipelineProvider;
pub(crate) struct ShutterBlurPipelineProvider;
pub(crate) struct DirtyBloomPipelineProvider;
pub(crate) struct HighlightExtractPipelineProvider;
pub(crate) struct CrtPipelineProvider;
pub(crate) struct WetReflectionsPipelineProvider;
pub(crate) struct PlateRelightPipelineProvider;
pub(crate) struct RefractiveMaterialPipelineProvider;

impl WgpuPostFxPipelineProvider for ColorQuantizePipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_COLOR_QUANTIZE
    }

    fn create_pipeline(
        &self,
        ctx: &WgpuPostFxPipelineCreateContext<'_>,
    ) -> wgpu::RenderPipeline {
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-color-quantize-pipeline-layout"),
                bind_group_layouts: &[
                    Some(ctx.texture_bind_group_layout),
                    Some(ctx.wet_reflections_uniform_bind_group_layout),
                    Some(ctx.texture_bind_group_layout),
                ],
                immediate_size: 0,
            });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.color_quantize_shader,
            &layout,
            ctx.format,
            "amigo-scene-color-quantize-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for DownscalePipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_DOWNSCALE
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-scene-downscale-pipeline-layout"),
            bind_group_layouts: &[
                Some(ctx.texture_bind_group_layout),
                Some(ctx.wet_reflections_uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.downscale_shader,
            &layout,
            ctx.format,
            "amigo-scene-downscale-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for ShutterBlurPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_SHUTTER_BLUR
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-scene-shutter-blur-pipeline-layout"),
            bind_group_layouts: &[
                Some(ctx.shutter_blur_texture_bind_group_layout),
                Some(ctx.wet_reflections_uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.shutter_blur_shader,
            &layout,
            ctx.format,
            "amigo-scene-shutter-blur-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for DirtyBloomPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_DIRTY_BLOOM
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-scene-dirty-bloom-pipeline-layout"),
            bind_group_layouts: &[
                Some(ctx.texture_bind_group_layout),
                Some(ctx.wet_reflections_uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.dirty_bloom_shader,
            &layout,
            ctx.format,
            "amigo-scene-dirty-bloom-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for HighlightExtractPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_AUX_HIGHLIGHT_EXTRACT
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-scene-highlight-extract-pipeline-layout"),
            bind_group_layouts: &[
                Some(ctx.texture_bind_group_layout),
                Some(ctx.wet_reflections_uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.highlight_extract_shader,
            &layout,
            ctx.format,
            "amigo-scene-highlight-extract-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for CrtPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_CRT
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-scene-crt-pipeline-layout"),
            bind_group_layouts: &[
                Some(ctx.texture_bind_group_layout),
                Some(ctx.wet_reflections_uniform_bind_group_layout),
            ],
            immediate_size: 0,
        });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.crt_shader,
            &layout,
            ctx.format,
            "amigo-scene-crt-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for WetReflectionsPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_WET_REFLECTIONS
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        create_copy_blend_pipeline(
            ctx.device,
            ctx.wet_reflections_shader,
            ctx.wet_reflections_pipeline_layout,
            ctx.format,
            "amigo-scene-wet-reflections-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for PlateRelightPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_AUX_PLATE_RELIGHT
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        create_copy_blend_pipeline(
            ctx.device,
            ctx.plate_relight_shader,
            ctx.wet_reflections_pipeline_layout,
            ctx.format,
            "amigo-scene-plate-relight-pipeline",
        )
    }
}

impl WgpuPostFxPipelineProvider for RefractiveMaterialPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_AUX_REFRACTIVE_MATERIAL
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        create_copy_blend_pipeline(
            ctx.device,
            ctx.refractive_material_shader,
            ctx.focus_blur_pipeline_layout,
            ctx.format,
            "amigo-scene-refractive-material-pipeline",
        )
    }
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
