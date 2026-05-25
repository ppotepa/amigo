use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};
use crate::renderer::service::post_fx::shaders::DIRTY_BLOOM_SHADER;

pub(crate) struct DirtyBloomPipelineProvider;

impl WgpuPostFxPipelineProvider for DirtyBloomPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_DIRTY_BLOOM
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-dirty-bloom-shader"),
                source: wgpu::ShaderSource::Wgsl(DIRTY_BLOOM_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-dirty-bloom-pipeline-layout"),
                bind_group_layouts: &[
                    Some(ctx.texture_bind_group_layout),
                    Some(ctx.wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        create_copy_blend_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.format,
            "amigo-render-dirty-bloom-pipeline",
        )
    }
}
