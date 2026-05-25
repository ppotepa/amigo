use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};
use crate::renderer::service::post_fx::shaders::REFRACTIVE_MATERIAL_SHADER;

pub(crate) struct RefractiveMaterialPipelineProvider;

impl WgpuPostFxPipelineProvider for RefractiveMaterialPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_AUX_REFRACTIVE_MATERIAL
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-refractive-material-shader"),
                source: wgpu::ShaderSource::Wgsl(REFRACTIVE_MATERIAL_SHADER.into()),
            });
        create_copy_blend_pipeline(
            ctx.device,
            &shader,
            ctx.focus_blur_pipeline_layout,
            ctx.format,
            "amigo-render-refractive-material-pipeline",
        )
    }
}
