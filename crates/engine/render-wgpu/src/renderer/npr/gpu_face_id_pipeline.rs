use crate::renderer::shaders::NPR_FACE_ID_SHADER;

use super::{vertex_storage_entry, vertex_uniform_entry};

#[derive(Debug)]
pub(crate) struct NprGpuFaceIdPipelineSet3d {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub(crate) fn create_npr_gpu_face_id_pipeline_set(
    device: &wgpu::Device,
) -> NprGpuFaceIdPipelineSet3d {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("amigo-npr-face-id-shader"),
        source: wgpu::ShaderSource::Wgsl(NPR_FACE_ID_SHADER.into()),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("amigo-npr-face-id-bind-group-layout"),
        entries: &[
            vertex_storage_entry(0),
            vertex_storage_entry(1),
            vertex_uniform_entry(8),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("amigo-npr-face-id-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("amigo-npr-face-id-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Uint,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    });
    NprGpuFaceIdPipelineSet3d {
        pipeline,
        bind_group_layout,
    }
}
