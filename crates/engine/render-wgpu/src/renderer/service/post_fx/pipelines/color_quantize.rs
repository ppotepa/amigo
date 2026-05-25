use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};
use crate::renderer::service::post_fx::shaders::COLOR_QUANTIZE_SHADER;

pub(crate) struct ColorQuantizePipelineProvider;

impl WgpuPostFxPipelineProvider for ColorQuantizePipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_COLOR_QUANTIZE
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-color-quantize-shader"),
                source: wgpu::ShaderSource::Wgsl(COLOR_QUANTIZE_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-color-quantize-pipeline-layout"),
                bind_group_layouts: &[
                    Some(ctx.texture_bind_group_layout),
                    Some(ctx.wet_reflections_uniform_bind_group_layout),
                    Some(ctx.texture_bind_group_layout),
                ],
                immediate_size: 0,
            });
        create_copy_blend_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.format,
            "amigo-render-color-quantize-pipeline",
        )
    }
}
