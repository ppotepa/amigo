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

/// A graphite brush deliberately remains a backend concern.  The neutral
/// packet selects `NprMedium::Graphite`; this shader decides how that mark is
/// deposited on the render target rather than asking the pipeline to know
/// about WGPU primitives.
pub const NPR_GRAPHITE_STROKE_SHADER: &str = r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    // x = normalized mark distance, y = signed ribbon side, z = resolved
    // medium deposition response,
    // w = stable mark seed.  These are deliberately independent of time.
    @location(3) mark: vec4<f32>,
    // x = paper tooth scale, y = fibre strength.
    @location(4) paper: vec2<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) mark: vec4<f32>,
    @location(2) paper: vec2<f32>,
};
fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color;
    o.mark = v.mark;
    o.paper = v.paper;
    return o;
}
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> {
    let pixel = floor(v.position.xy);
    let seed = vec2<f32>(v.mark.w, v.mark.w * 1.618);
    // A low-frequency paper tooth and a finer, deterministic graphite grain.
    let tooth = 0.62 + 0.38 * hash21(floor(pixel * (0.45 / max(v.paper.x, 0.01))) + seed);
    let grain = 0.72 + 0.28 * hash21(pixel * 1.7 + seed * 7.0);
    let fibres = 1.0 - v.paper.y * 0.16 + v.paper.y * 0.16 * sin(pixel.x * 0.35 + seed.x * 17.0);
    // A pencil ribbon is not a perfectly opaque marker: it has a soft, uneven
    // centre and fades slightly at the two physical sides of the stroke.
    let edge = smoothstep(1.08, 0.28, abs(v.mark.y));
    let pressure = 0.58 + 0.42 * sin(v.mark.x * 3.14159265);
    let deposit = clamp(tooth * grain * fibres * edge * pressure * v.mark.z, 0.0, 1.0);
    return vec4<f32>(v.color.rgb * (0.72 + 0.28 * grain), v.color.a * deposit);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NprGpuVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub depth: f32,
    pub mark: [f32; 4],
    pub paper: [f32; 2],
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
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 28,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 44,
                    shader_location: 4,
                },
            ],
        }
    }
}

pub struct NprPipelines {
    pub depth: wgpu::RenderPipeline,
    pub fill: wgpu::RenderPipeline,
    pub stroke: wgpu::RenderPipeline,
    pub graphite_stroke: wgpu::RenderPipeline,
}

impl NprPipelines {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_FILL_SHADER.into()),
        });
        let graphite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-graphite-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_GRAPHITE_STROKE_SHADER.into()),
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
        let graphite_stroke = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("amigo-npr-graphite-stroke"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &graphite_shader,
                entry_point: Some("vs_main"),
                buffers: &buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &graphite_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            depth: make("amigo-npr-depth", false, Some(wgpu::Face::Back), true, None),
            fill: make(
                "amigo-npr-fill",
                true,
                Some(wgpu::Face::Back),
                false,
                Some(wgpu::BlendState::ALPHA_BLENDING),
            ),
            stroke: make(
                "amigo-npr-stroke",
                true,
                None,
                false,
                Some(wgpu::BlendState::ALPHA_BLENDING),
            ),
            graphite_stroke,
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
