use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};
use crate::renderer::service::post_fx::shaders::PLATE_RELIGHT_SHADER;

pub(crate) struct PlateRelightPipelineProvider;

impl WgpuPostFxPipelineProvider for PlateRelightPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_AUX_PLATE_RELIGHT
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-plate-relight-shader"),
            source: wgpu::ShaderSource::Wgsl(PLATE_RELIGHT_SHADER.into()),
        });
        create_copy_blend_pipeline(
            ctx.device,
            &shader,
            ctx.wet_reflections_pipeline_layout,
            ctx.format,
            "amigo-scene-plate-relight-pipeline",
        )
    }
}
