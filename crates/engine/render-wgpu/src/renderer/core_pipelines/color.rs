use super::{
    CORE_COLOR_ADDITIVE_PIPELINE, CORE_COLOR_ALPHA_PIPELINE, CORE_COLOR_MULTIPLY_PIPELINE,
    CORE_COLOR_SCREEN_PIPELINE, CORE_NPR_STROKE_SEGMENT_ALPHA_PIPELINE,
    WgpuCorePipelineCreateContext, WgpuCorePipelineProvider,
};
use crate::renderer::{ColorVertex, NprStrokeSegmentVertex};
use crate::renderer::pipelines::{
    additive_blend_state, create_color_pipeline, multiply_blend_state, screen_blend_state,
};
use crate::renderer::shaders::{COLOR_SHADER, NPR_STROKE_SEGMENT_SHADER};

pub(crate) struct ColorAlphaPipelineProvider;
pub(crate) struct ColorAdditivePipelineProvider;
pub(crate) struct ColorMultiplyPipelineProvider;
pub(crate) struct ColorScreenPipelineProvider;
pub(crate) struct NprStrokeSegmentAlphaPipelineProvider;

impl WgpuCorePipelineProvider for ColorAlphaPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_COLOR_ALPHA_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-color-shader"),
                source: wgpu::ShaderSource::Wgsl(COLOR_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-render-color-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[ColorVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for ColorAdditivePipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_COLOR_ADDITIVE_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-color-shader"),
                source: wgpu::ShaderSource::Wgsl(COLOR_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-render-color-additive-pipeline",
            additive_blend_state(),
            &[ColorVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for ColorMultiplyPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_COLOR_MULTIPLY_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-color-shader"),
                source: wgpu::ShaderSource::Wgsl(COLOR_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-render-color-multiply-pipeline",
            multiply_blend_state(),
            &[ColorVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for ColorScreenPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_COLOR_SCREEN_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-color-shader"),
                source: wgpu::ShaderSource::Wgsl(COLOR_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-render-color-screen-pipeline",
            screen_blend_state(),
            &[ColorVertex::layout()],
        )
    }
}

impl WgpuCorePipelineProvider for NprStrokeSegmentAlphaPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        CORE_NPR_STROKE_SEGMENT_ALPHA_PIPELINE
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-render-npr-stroke-segment-shader"),
                source: wgpu::ShaderSource::Wgsl(NPR_STROKE_SEGMENT_SHADER.into()),
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-npr-stroke-segment-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        create_color_pipeline(
            ctx.device,
            &shader,
            &layout,
            ctx.surface_format,
            "amigo-render-npr-stroke-segment-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[NprStrokeSegmentVertex::layout()],
        )
    }
}
