use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};

pub(crate) struct WetReflectionsPipelineProvider;

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
