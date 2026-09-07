//! GPU-side NPR preparation contracts.
//!
//! The packet itself remains owned by `amigo-render-npr`; this module only owns
//! backend-friendly immutable buffers and shader sources.

use amigo_render_npr::NprGeometry;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, PartialEq)]
pub struct NprPreparedGeometry {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl NprPreparedGeometry {
    pub fn cube() -> Self {
        let geometry = NprGeometry::canonical_cube();
        Self {
            positions: geometry
                .vertices
                .iter()
                .map(|vertex| vertex.position.to_array())
                .collect(),
            indices: geometry
                .triangles
                .iter()
                .flat_map(|triangle| triangle.iter().copied())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NprPipelineKey {
    pub color_format: wgpu::TextureFormat,
}

pub const NPR_FILL_SHADER: &str = r#"
struct Vertex { @location(0) position: vec2<f32>, @location(1) color: vec4<f32>, @location(2) depth: f32 };
struct Out { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> };
@vertex fn vs_main(v: Vertex) -> Out { var o: Out; o.position = vec4<f32>(v.position, v.depth, 1.0); o.color = v.color; return o; }
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> { return v.color; }
"#;

pub const NPR_STROKE_SHADER: &str = NPR_FILL_SHADER;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NprGpuVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub depth: f32,
}

impl NprGpuVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 24,
                    shader_location: 2,
                },
            ],
        }
    }
}

pub struct NprPipelines {
    pub depth: wgpu::RenderPipeline,
    pub fill: wgpu::RenderPipeline,
    pub stroke: wgpu::RenderPipeline,
}

impl NprPipelines {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_FILL_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-npr-pipeline-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let buffers = [NprGpuVertex::layout()];
        let make = |label: &'static str,
                    fragment: bool,
                    cull_mode: Option<wgpu::Face>,
                    depth_write: bool,
                    blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &buffers,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: fragment.then_some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            depth: make("amigo-npr-depth", false, Some(wgpu::Face::Back), true, None),
            fill: make("amigo-npr-fill", true, Some(wgpu::Face::Back), false, None),
            stroke: make(
                "amigo-npr-stroke",
                true,
                None,
                false,
                Some(wgpu::BlendState::ALPHA_BLENDING),
            ),
        }
    }

    pub fn vertex_buffer(
        device: &wgpu::Device,
        vertices: &[NprGpuVertex],
        label: &'static str,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}
