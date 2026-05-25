use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};
use crate::renderer::service::post_fx::shaders::WET_REFLECTIONS_SHADER;

pub(crate) struct WetReflectionsPipelineProvider;

impl WgpuPostFxPipelineProvider for WetReflectionsPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_WET_REFLECTIONS
    }

    fn create_pipeline(&self, ctx: &WgpuPostFxPipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-wet-reflections-shader"),
                source: wgpu::ShaderSource::Wgsl(WET_REFLECTIONS_SHADER.into()),
            });
        create_copy_blend_pipeline(
            ctx.device,
            &shader,
            ctx.wet_reflections_pipeline_layout,
            ctx.format,
            "amigo-render-wet-reflections-pipeline",
        )
    }
}
