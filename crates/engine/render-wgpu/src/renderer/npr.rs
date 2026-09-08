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
    @location(5) material: vec4<f32>,
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

/// A stroke is a geometric envelope plus an analytic material edge. `phase.x`
/// is the signed lateral coordinate of the envelope and `phase.y` supplies a
/// stable per-stroke grain phase. Material.x is edge softness and material.y is
/// the local pressure response. This keeps paper detail out of the tessellator.
pub const NPR_STROKE_SHADER: &str = r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
    @location(5) material: vec4<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) coverage: f32,
    @location(2) lateral: f32,
    @location(3) grain: f32,
    @location(4) material: vec4<f32>,
};
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color;
    o.coverage = v.coverage;
    o.lateral = v.phase.x;
    o.grain = v.phase.y;
    o.material = v.material;
    return o;
}
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> {
    let edge_width = max(fwidth(v.lateral), 0.008) * (1.0 + v.material.x * 6.0);
    let edge = 1.0 - smoothstep(1.0 - edge_width, 1.0 + edge_width, abs(v.lateral));
    let tooth = hash(floor(v.position.xy * 1.7 + vec2<f32>(v.grain, v.grain * 1.73)));
    let graphite = 1.0 - (tooth - 0.5) * v.material.z * (0.32 + (1.0 - v.material.y) * 0.24)
        - v.material.w * (tooth - 0.5) * 0.08;
    let alpha = clamp(v.color.a * v.coverage * edge * graphite, 0.0, 1.0);
    return vec4<f32>(v.color.rgb * alpha, alpha);
}
"#;

pub const NPR_PAPER_SHADER: &str = r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
    @location(5) material: vec4<f32>,
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
    pub material: [f32; 4],
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 40,
                    shader_location: 5,
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

pub(crate) struct NprVertexBuffer {
    pub buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

pub(crate) struct NprIndexedBuffer {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: u32,
}

impl NprPipelines {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let fill_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_FILL_SHADER.into()),
        });
        let stroke_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-stroke-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_STROKE_SHADER.into()),
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
                    shader: &wgpu::ShaderModule,
                    fragment: bool,
                    cull_mode: Option<wgpu::Face>,
                    depth_write: bool,
                    blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &buffers,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: fragment.then_some(wgpu::FragmentState {
                    module: shader,
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
            depth: make(
                "amigo-npr-depth",
                &fill_shader,
                false,
                Some(wgpu::Face::Back),
                true,
                None,
            ),
            paper,
            fill: make(
                "amigo-npr-fill",
                &fill_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                None,
            ),
            stroke: make(
                "amigo-npr-stroke",
                &stroke_shader,
                true,
                None,
                false,
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

    pub fn vertex_buffers(
        device: &wgpu::Device,
        vertices: &[NprGpuVertex],
        label: &'static str,
    ) -> Vec<NprVertexBuffer> {
        const MAX_BUFFER_BYTES: usize = 64 * 1024 * 1024;

        if vertices.is_empty() {
            return Vec::new();
        }
        let device_limit = device.limits().max_buffer_size as usize;
        let max_bytes = MAX_BUFFER_BYTES.min(device_limit);
        let stride = std::mem::size_of::<NprGpuVertex>();
        let mut max_vertices = max_bytes / stride;
        max_vertices -= max_vertices % 3;
        if max_vertices < 3 {
            // A renderer limit is external input. Returning no batch leaves the
            // target valid and lets the domain/backend diagnostics report the
            // quality limit instead of panicking inside `create_buffer`.
            return Vec::new();
        }

        vertices
            .chunks(max_vertices)
            .map(|chunk| NprVertexBuffer {
                buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytemuck::cast_slice(chunk),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                vertex_count: chunk.len() as u32,
            })
            .collect()
    }

    /// Creates one bounded indexed draw. Callers partition batches before this
    /// boundary so no packet can request a device allocation above the limit.
    pub fn indexed_buffer(
        device: &wgpu::Device,
        vertices: &[NprGpuVertex],
        indices: &[u32],
        label: &'static str,
    ) -> Option<NprIndexedBuffer> {
        if vertices.is_empty() || indices.is_empty() || indices.len() > u32::MAX as usize {
            return None;
        }
        let vertex_bytes = std::mem::size_of_val(vertices);
        let index_bytes = std::mem::size_of_val(indices);
        let device_limit = device.limits().max_buffer_size as usize;
        if vertex_bytes > device_limit || index_bytes > device_limit {
            return None;
        }
        Some(NprIndexedBuffer {
            vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-npr-stroke-indices"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            index_count: indices.len() as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_vertex_layout_matches_the_stroke_shader_contract() {
        let layout = NprGpuVertex::layout();
        assert_eq!(layout.array_stride, std::mem::size_of::<NprGpuVertex>() as u64);
        assert_eq!(layout.attributes[4].shader_location, 4);
        assert_eq!(layout.attributes[4].offset, 32);
        assert_eq!(layout.attributes[4].format, wgpu::VertexFormat::Float32x2);
        assert_eq!(layout.attributes[5].shader_location, 5);
        assert_eq!(layout.attributes[5].offset, 40);
        assert_eq!(layout.attributes[5].format, wgpu::VertexFormat::Float32x4);
    }
}
