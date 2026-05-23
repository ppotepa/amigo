use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};

pub(crate) struct PlateRelightPipelineProvider;

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
