use super::{
    CORE_TEXTURE_ADDITIVE_PIPELINE, CORE_TEXTURE_ALPHA_PIPELINE, CORE_TEXTURE_LIGHTEN_PIPELINE,
    CORE_TEXTURE_MULTIPLY_PIPELINE, CORE_TEXTURE_OPAQUE_PIPELINE, CORE_TEXTURE_SCREEN_PIPELINE,
    WgpuCorePipelineCreateContext, WgpuCorePipelineProvider,
};
use crate::renderer::pipelines::{
    additive_blend_state, create_color_pipeline, lighten_blend_state, multiply_blend_state,
    screen_blend_state,
};
use crate::renderer::shaders::TEXTURE_SHADER;
use crate::renderer::TextureVertex;

pub(crate) struct TextureAlphaPipelineProvider;
pub(crate) struct TextureOpaquePipelineProvider;
pub(crate) struct TextureAdditivePipelineProvider;
pub(crate) struct TextureMultiplyPipelineProvider;
pub(crate) struct TextureScreenPipelineProvider;
pub(crate) struct TextureLightenPipelineProvider;

fn texture_pipeline_layout(
    ctx: &WgpuCorePipelineCreateContext<'_>,
) -> wgpu::PipelineLayout {
    ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("amigo-scene-texture-pipeline-layout"),
        bind_group_layouts: &[Some(ctx.texture_bind_group_layout)],
        immediate_size: 0,
    })
}

fn texture_shader(ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::ShaderModule {
    ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("amigo-scene-texture-shader"),
        source: wgpu::ShaderSource::Wgsl(TEXTURE_SHADER.into()),
    })
}

impl WgpuCorePipelineProvider for TextureAlphaPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_TEXTURE_ALPHA_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = texture_shader(ctx);
        let layout = texture_pipeline_layout(ctx);
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-scene-texture-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[TextureVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for TextureOpaquePipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_TEXTURE_OPAQUE_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = texture_shader(ctx);
        let layout = texture_pipeline_layout(ctx);
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-scene-texture-opaque-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            &[TextureVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for TextureAdditivePipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_TEXTURE_ADDITIVE_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = texture_shader(ctx);
        let layout = texture_pipeline_layout(ctx);
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-scene-texture-additive-pipeline",
            additive_blend_state(),
            &[TextureVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for TextureMultiplyPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_TEXTURE_MULTIPLY_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = texture_shader(ctx);
        let layout = texture_pipeline_layout(ctx);
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-scene-texture-multiply-pipeline",
            multiply_blend_state(),
            &[TextureVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for TextureScreenPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_TEXTURE_SCREEN_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = texture_shader(ctx);
        let layout = texture_pipeline_layout(ctx);
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-scene-texture-screen-pipeline",
            screen_blend_state(),
            &[TextureVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for TextureLightenPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_TEXTURE_LIGHTEN_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = texture_shader(ctx);
        let layout = texture_pipeline_layout(ctx);
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-scene-texture-lighten-pipeline",
            lighten_blend_state(),
            &[TextureVertex::layout()],
        )
    }
}
