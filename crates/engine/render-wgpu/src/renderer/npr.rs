//! GPU-side NPR preparation contracts.
//!
//! The packet itself remains owned by `amigo-render-npr`; this module only owns
//! backend-friendly immutable buffers and shader sources.

use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NprPipelineKey {
    pub color_format: wgpu::TextureFormat,
}

pub const NPR_FILL_SHADER: &str = r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) coverage: f32,
};
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color;
    o.coverage = v.coverage;
    return o;
}
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(v.color.rgb, v.color.a * v.coverage);
}
"#;

pub const NPR_STROKE_SHADER: &str = NPR_FILL_SHADER;

pub const NPR_PAPER_SHADER: &str = r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) grain: f32,
    @location(2) phase: vec2<f32>,
};
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, 1.0, 1.0);
    o.color = v.color;
    o.grain = v.coverage;
    o.phase = v.phase;
    return o;
}
fn hash(p: vec2<f32>) -> f32 {
    let h = sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453;
    return fract(h);
}
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> {
    let coarse = hash(floor(v.position.xy * 0.22 + v.phase));
    let fine = hash(floor(v.position.xy * 1.37 + v.phase * 1.73));
    let fibers = sin(v.position.x * 0.045 + v.phase.x + sin(v.position.y * 0.011 + v.phase.y) * 1.7) * 0.5 + 0.5;
    let variation = (coarse - 0.5) * v.grain * 0.11
        + (fine - 0.5) * v.grain * v.grain * 0.035
        + (fibers - 0.5) * v.grain * 0.025;
    return vec4<f32>(v.color.rgb * (1.0 + variation), v.color.a);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NprGpuVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub depth: f32,
    pub coverage: f32,
    pub phase: [f32; 2],
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 28,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 32,
                    shader_location: 4,
                },
            ],
        }
    }
}

pub struct NprPipelines {
    pub depth: wgpu::RenderPipeline,
    pub paper: wgpu::RenderPipeline,
    pub fill: wgpu::RenderPipeline,
    pub stroke: wgpu::RenderPipeline,
}

impl NprPipelines {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_FILL_SHADER.into()),
        });
        let paper_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-paper-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_PAPER_SHADER.into()),
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
        let paper = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("amigo-npr-paper"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &paper_shader,
                entry_point: Some("vs_main"),
                buffers: &buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &paper_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
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
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            depth: make("amigo-npr-depth", false, Some(wgpu::Face::Back), true, None),
            paper,
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
