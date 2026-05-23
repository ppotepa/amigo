use super::{
    WgpuPostFxPipelineCreateContext, WgpuPostFxPipelineProvider, create_copy_blend_pipeline,
};

pub(crate) struct FilmEmulsionPipelineProvider;

impl WgpuPostFxPipelineProvider for FilmEmulsionPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        crate::renderer::service::POST_FX_EXECUTOR_FILM_EMULSION
    }

    fn create_pipeline(
        &self,
        ctx: &WgpuPostFxPipelineCreateContext<'_>,
    ) -> wgpu::RenderPipeline {
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-film-emulsion-pipeline-layout"),
                bind_group_layouts: &[
                    Some(ctx.camera_visual_source_bind_group_layout),
                    Some(ctx.wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        create_copy_blend_pipeline(
            ctx.device,
            ctx.film_emulsion_shader,
            &layout,
            ctx.format,
            "amigo-scene-film-emulsion-pipeline",
        )
    }
}
