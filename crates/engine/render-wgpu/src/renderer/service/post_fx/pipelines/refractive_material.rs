use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};

pub(crate) struct RefractiveMaterialPipelineProvider;

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
