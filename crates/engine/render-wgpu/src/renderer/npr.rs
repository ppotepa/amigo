//! GPU-side NPR preparation contracts.
//!
//! The packet itself remains owned by `amigo-render-npr`; this module only owns
//! backend-friendly immutable buffers and shader sources.

use std::hash::{Hash, Hasher};
mod mask;
pub use mask::NprGpuSurface;

/// Shared execution of declared NPR packets for full scenes and companion targets.
pub struct WgpuNprRenderer {
    pub(crate) npr_pipelines: NprPipelines,
    pub(crate) npr_buffers: std::sync::Mutex<NprBufferPool>,
}
impl WgpuNprRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            npr_pipelines: NprPipelines::new(device, format),
            npr_buffers: std::sync::Mutex::default(),
        }
    }
}

#[derive(Default)]
pub(crate) struct NprBufferPool {
    slots: Vec<NprBufferSlot>,
    cursor: usize,
}
struct NprBufferSlot {
    buffer: wgpu::Buffer,
    capacity: u64,
    hash: u64,
    length: usize,
    usage: wgpu::BufferUsages,
}
impl NprBufferPool {
    pub fn begin(&mut self) {
        self.cursor = 0;
    }
    pub fn finish(&mut self) {
        self.slots.truncate(self.cursor);
    }
    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        usage: wgpu::BufferUsages,
        label: &'static str,
    ) -> wgpu::Buffer {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();
        let index = self.cursor;
        self.cursor += 1;
        let required = (bytes.len().max(4) as u64).next_multiple_of(4);
        let allocate = self
            .slots
            .get(index)
            .is_none_or(|slot| slot.capacity < required || slot.usage != usage);
        if allocate {
            let capacity = required
                .next_power_of_two()
                .min(device.limits().max_buffer_size);
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: capacity,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let slot = NprBufferSlot {
                buffer,
                capacity,
                hash: 0,
                length: usize::MAX,
                usage,
            };
            if index == self.slots.len() {
                self.slots.push(slot);
            } else {
                self.slots[index] = slot;
            }
        }
        let slot = &mut self.slots[index];
        if slot.hash != hash || slot.length != bytes.len() {
            if !bytes.is_empty() {
                queue.write_buffer(&slot.buffer, 0, bytes);
            }
            slot.hash = hash;
            slot.length = bytes.len();
        }
        slot.buffer.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NprPipelineKey {
    pub color_format: wgpu::TextureFormat,
}

pub const NPR_FILL_SHADER: &str = concat!(
    include_str!("npr/mask.wgsl"),
    r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
    @location(5) material: vec4<f32>,
    @location(6) surface_position: vec4<f32>,
    @location(7) surface_normal: vec4<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) coverage: f32,
    @location(5) surface_position: vec4<f32>,
    @location(6) surface_normal: vec4<f32>,
};
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color;
    o.coverage = v.coverage;
    o.surface_position = v.surface_position * (1.0-v.depth);
    o.surface_normal = v.surface_normal * (1.0-v.depth);
    return o;
}
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> {
    let alpha = v.color.a * v.coverage * surface_coverage(v.surface_position,v.surface_normal,v.position.z);
    return vec4<f32>(v.color.rgb * alpha, alpha);
}
"#
);

/// Underpainting uses continuous screen-space pigment granulation. The
/// material coefficient is authored in `NprPaintMedium`; sampling here avoids
/// discontinuities at source triangle boundaries.
pub const NPR_PAINT_SHADER: &str = concat!(
    include_str!("npr/mask.wgsl"),
    r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
    @location(5) material: vec4<f32>,
    @location(6) surface_position: vec4<f32>,
    @location(7) surface_normal: vec4<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) coverage: f32,
    @location(2) granulation: f32,
    @location(5) surface_position: vec4<f32>,
    @location(6) surface_normal: vec4<f32>,
};
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color;
    o.coverage = v.coverage;
    o.surface_position = v.surface_position * (1.0-v.depth);
    o.surface_normal = v.surface_normal * (1.0-v.depth);
    o.granulation = v.material.x;
    return o;
}
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(91.17, 217.43))) * 43758.5453);
}
@fragment fn fs_main(v: Out) -> @location(0) vec4<f32> {
    let coarse = hash(floor(v.position.xy * 0.18));
    let fine = hash(floor(v.position.xy * 0.71 + vec2<f32>(37.0, 19.0)));
    let variation = ((coarse - 0.5) * 0.72 + (fine - 0.5) * 0.28)
        * clamp(v.granulation, 0.0, 1.0) * 0.34;
    let alpha = clamp(v.color.a * v.coverage * surface_coverage(v.surface_position,v.surface_normal,v.position.z) * (1.0 + variation), 0.0, 1.0);
    return vec4<f32>(v.color.rgb * alpha, alpha);
}
"#
);

/// A stroke is a geometric envelope plus an analytic material edge. `phase.x`
/// is the signed lateral coordinate of the envelope and `phase.y` supplies a
/// stable per-stroke grain phase. Material.x is edge softness and material.y is
/// the local pressure response. This keeps paper detail out of the tessellator.
pub const NPR_STROKE_SHADER: &str = concat!(
    include_str!("npr/mask.wgsl"),
    r#"
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) depth: f32,
    @location(3) coverage: f32,
    @location(4) phase: vec2<f32>,
    @location(5) material: vec4<f32>,
    @location(6) surface_position: vec4<f32>,
    @location(7) surface_normal: vec4<f32>,
};
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) coverage: f32,
    @location(2) lateral: f32,
    @location(3) grain: f32,
    @location(4) material: vec4<f32>,
    @location(5) surface_position: vec4<f32>,
    @location(6) surface_normal: vec4<f32>,
};
@vertex fn vs_main(v: Vertex) -> Out {
    var o: Out;
    o.position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color;
    o.coverage = v.coverage;
    o.surface_position = v.surface_position * (1.0-v.depth);
    o.surface_normal = v.surface_normal * (1.0-v.depth);
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
    // Paper lives in screen space: it is a physical sheet under a moving
    // drawing, not noise glued to the object or regenerated per frame.
    let paper_cell = floor(v.position.xy * 1.55 + vec2<f32>(v.grain, v.grain * 1.73));
    let tooth = hash(paper_cell);
    let fiber = hash(floor(v.position.xy * vec2<f32>(0.42, 3.1) + vec2<f32>(v.grain * 3.7, v.grain)));
    let tooth_amount = clamp(v.material.z, 0.0, 1.0);
    let dryness = clamp(v.material.w, 0.0, 1.0);
    let pressure = clamp(v.material.y, 0.0, 1.0);
    // Low-pressure graphite catches on paper peaks; a hard, pressed ink line
    // remains continuous. Dry media gets stable, local dropouts rather than
    // temporal flicker.
    let pigment = mix(
        1.0,
        0.24 + 0.76 * smoothstep(0.12, 0.92, tooth * 0.72 + fiber * 0.28),
        tooth_amount * (0.46 + (1.0 - pressure) * 0.34),
    );
    let dropout = smoothstep(0.79 - pressure * 0.16, 0.98, tooth)
        * dryness
        * (0.28 + tooth_amount * 0.54);
    let alpha = clamp(v.color.a * v.coverage * surface_coverage(v.surface_position,v.surface_normal,v.position.z) * edge * pigment * (1.0 - dropout), 0.0, 1.0);
    return vec4<f32>(v.color.rgb * alpha, alpha);
}
"#
);

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
    pub surface: NprGpuSurface,
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 56,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 72,
                    shader_location: 7,
                },
            ],
        }
    }
}

pub struct NprPipelines {
    mask_layout: wgpu::BindGroupLayout,
    pub depth: wgpu::RenderPipeline,
    pub paper: wgpu::RenderPipeline,
    pub fill_normal: wgpu::RenderPipeline,
    pub fill_multiply: wgpu::RenderPipeline,
    pub fill_screen: wgpu::RenderPipeline,
    pub paint_normal: wgpu::RenderPipeline,
    pub paint_multiply: wgpu::RenderPipeline,
    pub paint_screen: wgpu::RenderPipeline,
    pub stroke_normal: wgpu::RenderPipeline,
    pub stroke_multiply: wgpu::RenderPipeline,
    pub stroke_screen: wgpu::RenderPipeline,
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

/// NPR fragments carry premultiplied colour.  Keeping the mode mapping in one
/// place guarantees fills, washes and strokes compose with identical opacity
/// semantics regardless of the pipeline selected by their source geometry.
fn npr_blend_state(mode: amigo_render_npr::NprBlendMode) -> wgpu::BlendState {
    let alpha = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    let color = match mode {
        amigo_render_npr::NprBlendMode::Normal => alpha,
        amigo_render_npr::NprBlendMode::Multiply => wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Dst,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        amigo_render_npr::NprBlendMode::Screen => wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrc,
            operation: wgpu::BlendOperation::Add,
        },
    };
    wgpu::BlendState { color, alpha }
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
        let paint_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-paint-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_PAINT_SHADER.into()),
        });
        let paper_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-paper-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_PAPER_SHADER.into()),
        });
        let mask_layout = mask::layout(device);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-npr-pipeline-layout"),
            bind_group_layouts: &[Some(&mask_layout)],
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
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let normal = npr_blend_state(amigo_render_npr::NprBlendMode::Normal);
        let multiply = npr_blend_state(amigo_render_npr::NprBlendMode::Multiply);
        let screen = npr_blend_state(amigo_render_npr::NprBlendMode::Screen);
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
            fill_normal: make(
                "amigo-npr-fill-normal",
                &fill_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                Some(normal),
            ),
            fill_multiply: make(
                "amigo-npr-fill-multiply",
                &fill_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                Some(multiply),
            ),
            fill_screen: make(
                "amigo-npr-fill-screen",
                &fill_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                Some(screen),
            ),
            paint_normal: make(
                "amigo-npr-paint-normal",
                &paint_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                Some(normal),
            ),
            paint_multiply: make(
                "amigo-npr-paint-multiply",
                &paint_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                Some(multiply),
            ),
            paint_screen: make(
                "amigo-npr-paint-screen",
                &paint_shader,
                true,
                Some(wgpu::Face::Back),
                false,
                Some(screen),
            ),
            stroke_normal: make(
                "amigo-npr-stroke-normal",
                &stroke_shader,
                true,
                None,
                false,
                Some(normal),
            ),
            stroke_multiply: make(
                "amigo-npr-stroke-multiply",
                &stroke_shader,
                true,
                None,
                false,
                Some(multiply),
            ),
            stroke_screen: make(
                "amigo-npr-stroke-screen",
                &stroke_shader,
                true,
                None,
                false,
                Some(screen),
            ),
            mask_layout,
        }
    }

    pub fn fill_for(&self, blend: amigo_render_npr::NprBlendMode) -> &wgpu::RenderPipeline {
        match blend {
            amigo_render_npr::NprBlendMode::Normal => &self.fill_normal,
            amigo_render_npr::NprBlendMode::Multiply => &self.fill_multiply,
            amigo_render_npr::NprBlendMode::Screen => &self.fill_screen,
        }
    }

    pub fn paint_for(&self, blend: amigo_render_npr::NprBlendMode) -> &wgpu::RenderPipeline {
        match blend {
            amigo_render_npr::NprBlendMode::Normal => &self.paint_normal,
            amigo_render_npr::NprBlendMode::Multiply => &self.paint_multiply,
            amigo_render_npr::NprBlendMode::Screen => &self.paint_screen,
        }
    }

    pub fn stroke_for(&self, blend: amigo_render_npr::NprBlendMode) -> &wgpu::RenderPipeline {
        match blend {
            amigo_render_npr::NprBlendMode::Normal => &self.stroke_normal,
            amigo_render_npr::NprBlendMode::Multiply => &self.stroke_multiply,
            amigo_render_npr::NprBlendMode::Screen => &self.stroke_screen,
        }
    }

    pub fn vertex_buffer(
        pool: &mut NprBufferPool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[NprGpuVertex],
        label: &'static str,
    ) -> wgpu::Buffer {
        pool.upload(
            device,
            queue,
            bytemuck::cast_slice(vertices),
            wgpu::BufferUsages::VERTEX,
            label,
        )
    }

    pub fn vertex_buffers(
        pool: &mut NprBufferPool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
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
                buffer: pool.upload(
                    device,
                    queue,
                    bytemuck::cast_slice(chunk),
                    wgpu::BufferUsages::VERTEX,
                    label,
                ),
                vertex_count: chunk.len() as u32,
            })
            .collect()
    }

    /// Creates one bounded indexed draw. Callers partition batches before this
    /// boundary so no packet can request a device allocation above the limit.
    pub fn indexed_buffer(
        pool: &mut NprBufferPool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
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
            vertices: pool.upload(
                device,
                queue,
                bytemuck::cast_slice(vertices),
                wgpu::BufferUsages::VERTEX,
                label,
            ),
            indices: pool.upload(
                device,
                queue,
                bytemuck::cast_slice(indices),
                wgpu::BufferUsages::INDEX,
                "amigo-npr-stroke-indices",
            ),
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
        assert_eq!(
            layout.array_stride,
            std::mem::size_of::<NprGpuVertex>() as u64
        );
        assert_eq!(layout.attributes[4].shader_location, 4);
        assert_eq!(layout.attributes[4].offset, 32);
        assert_eq!(layout.attributes[4].format, wgpu::VertexFormat::Float32x2);
        assert_eq!(layout.attributes[5].shader_location, 5);
        assert_eq!(layout.attributes[5].offset, 40);
        assert_eq!(layout.attributes[5].format, wgpu::VertexFormat::Float32x4);
        assert_eq!(layout.attributes[6].offset, 56);
        assert_eq!(layout.attributes[7].offset, 72);
        assert_eq!(layout.array_stride, 88);
    }

    #[test]
    fn npr_blend_modes_preserve_premultiplied_layer_opacity() {
        let normal = npr_blend_state(amigo_render_npr::NprBlendMode::Normal);
        assert_eq!(normal.color.src_factor, wgpu::BlendFactor::One);
        assert_eq!(normal.color.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);

        let multiply = npr_blend_state(amigo_render_npr::NprBlendMode::Multiply);
        assert_eq!(multiply.color.src_factor, wgpu::BlendFactor::Dst);
        assert_eq!(
            multiply.color.dst_factor,
            wgpu::BlendFactor::OneMinusSrcAlpha
        );

        let screen = npr_blend_state(amigo_render_npr::NprBlendMode::Screen);
        assert_eq!(screen.color.src_factor, wgpu::BlendFactor::One);
        assert_eq!(screen.color.dst_factor, wgpu::BlendFactor::OneMinusSrc);

        for blend in [normal, multiply, screen] {
            assert_eq!(blend.alpha.src_factor, wgpu::BlendFactor::One);
            assert_eq!(blend.alpha.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
        }
    }
}
