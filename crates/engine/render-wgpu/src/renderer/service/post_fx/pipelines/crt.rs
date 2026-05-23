use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};

pub(crate) struct CrtPipelineProvider;

impl WgpuPostFxPipelineProvider for CrtPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_CRT
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
