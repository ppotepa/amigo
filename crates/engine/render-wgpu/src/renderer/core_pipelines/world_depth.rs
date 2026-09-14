use super::{WgpuCorePipelineCreateContext, WgpuCorePipelineProvider};
use crate::renderer::{ColorVertex, pipelines::create_color_pipeline_with_depth, shaders::{COLOR_SHADER, PENCIL_COLOR_SHADER}};

pub(crate) struct WorldDepthPipelineProvider { pub pencil: bool, pub depth_write: bool }

impl WgpuCorePipelineProvider for WorldDepthPipelineProvider {
    fn pipeline_id(&self) -> &'static str {
        match (self.pencil, self.depth_write) {
            (false, true) => "core.world-depth.color",
            (true, true) => "core.world-depth.pencil",
            (false, false) => "core.world-depth.color-read",
            (true, false) => "core.world-depth.pencil-read",
        }
    }

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(self.pipeline_id()),
            source: wgpu::ShaderSource::Wgsl(if self.pencil { PENCIL_COLOR_SHADER } else { COLOR_SHADER }.into()),
        });
        let layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(self.pipeline_id()), bind_group_layouts: &[], immediate_size: 0,
        });
        create_color_pipeline_with_depth(ctx.device, &shader, &layout, ctx.surface_format,
            self.pipeline_id(), wgpu::BlendState::ALPHA_BLENDING, &[ColorVertex::layout()],
            Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(self.depth_write),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: Default::default(), bias: Default::default(),
            }))
    }
}
