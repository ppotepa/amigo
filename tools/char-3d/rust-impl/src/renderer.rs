use crate::pipeline::{ContourKind, Mark, RenderFrame};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

pub struct GpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    size: PhysicalSize<u32>,
}

impl GpuRenderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("char-3d renderer device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| anyhow::anyhow!("surface is not supported by adapter"))?;
        config.view_formats = vec![config.format];
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("char-3d vector shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("char-3d pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("char-3d vector pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            size,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, frame: &RenderFrame) -> bool {
        if self.size.width == 0 || self.size.height == 0 {
            return true;
        }
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
            _ => return false,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut vertices = Vec::new();
        build_vertices(frame, &mut vertices);
        let vertex_buffer = (!vertices.is_empty()).then(|| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("char-3d frame vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("char-3d render encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("char-3d render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: frame.paper[0] as f64,
                            g: frame.paper[1] as f64,
                            b: frame.paper[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            if let Some(vertex_buffer) = &vertex_buffer {
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw(0..vertices.len() as u32, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
        output.present();
        true
    }
}

fn build_vertices(frame: &RenderFrame, out: &mut Vec<Vertex>) {
    for region in &frame.paint_regions {
        if region.points.len() >= 3 {
            let mut color = region.color;
            color[3] *= region.alpha;
            for i in 1..region.points.len() - 1 {
                tri(
                    out,
                    frame,
                    region.points[0],
                    region.points[i],
                    region.points[i + 1],
                    color,
                );
            }
        }
    }
    for contour in &frame.contours {
        let color = match contour.kind {
            ContourKind::Contour => [0.09, 0.07, 0.04, if contour.visible { 0.85 } else { 0.28 }],
            ContourKind::Crease => [0.17, 0.13, 0.09, 0.58],
            ContourKind::Suggestive => [0.24, 0.21, 0.18, 0.38],
            ContourKind::Hidden => [0.40, 0.35, 0.30, 0.25],
        };
        line(out, frame, contour.a, contour.b, 1.25, color);
    }
    for mark in &frame.marks {
        match mark {
            Mark::Line {
                pts,
                color,
                width,
                alpha,
            } => {
                let mut c = *color;
                c[3] = *alpha;
                for pair in pts.windows(2) {
                    line(out, frame, pair[0], pair[1], *width, c);
                }
            }
            Mark::Dot {
                center,
                radius,
                color,
                alpha,
            } => {
                let mut c = *color;
                c[3] = *alpha;
                circle(out, frame, *center, *radius, c);
            }
        }
    }
}

fn ndc(frame: &RenderFrame, p: Vec2) -> [f32; 2] {
    [
        p.x / frame.width.max(1) as f32 * 2.0 - 1.0,
        1.0 - p.y / frame.height.max(1) as f32 * 2.0,
    ]
}

fn push(out: &mut Vec<Vertex>, frame: &RenderFrame, p: Vec2, color: [f32; 4]) {
    out.push(Vertex {
        pos: ndc(frame, p),
        color,
    });
}

fn tri(out: &mut Vec<Vertex>, frame: &RenderFrame, a: Vec2, b: Vec2, c: Vec2, color: [f32; 4]) {
    push(out, frame, a, color);
    push(out, frame, b, color);
    push(out, frame, c, color);
}

fn line(out: &mut Vec<Vertex>, frame: &RenderFrame, a: Vec2, b: Vec2, width: f32, color: [f32; 4]) {
    let dir = b - a;
    let len = dir.length();
    if len <= 0.01 {
        return;
    }
    let n = Vec2::new(-dir.y, dir.x) / len * width.max(0.2) * 0.5;
    tri(out, frame, a - n, b - n, b + n, color);
    tri(out, frame, a - n, b + n, a + n, color);
}

fn circle(out: &mut Vec<Vertex>, frame: &RenderFrame, center: Vec2, radius: f32, color: [f32; 4]) {
    let steps = 14;
    for i in 0..steps {
        let a0 = i as f32 / steps as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / steps as f32 * std::f32::consts::TAU;
        tri(
            out,
            frame,
            center,
            center + Vec2::new(a0.cos(), a0.sin()) * radius,
            center + Vec2::new(a1.cos(), a1.sin()) * radius,
            color,
        );
    }
}
