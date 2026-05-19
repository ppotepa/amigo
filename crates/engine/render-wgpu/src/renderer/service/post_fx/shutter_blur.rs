use std::time::Instant;

use amigo_composite_plugin::ShutterBlur2d;
use amigo_core::AmigoResult;
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::service::WgpuSceneRenderer;

#[derive(Default)]
pub(crate) struct ShutterBlurRuntime {
    history_a: Option<ShutterBlurHistory>,
    history_b: Option<ShutterBlurHistory>,
    last_frame: Option<Instant>,
}

struct ShutterBlurHistory {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    initialized: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ShutterBlurUniform {
    resolution: [f32; 2],
    opacity: f32,
    shutter_fraction: f32,
    exposure_seconds: f32,
    history_mix: f32,
    history_mix_2: f32,
    edge_rejection: f32,
    luma_threshold: f32,
    dt: f32,
    target_dt: f32,
    history_ready_a: f32,
    history_ready_b: f32,
    frame_hold: f32,
    debug_motion: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct DownscaleUniform {
    resolution: [f32; 2],
    factor: f32,
    opacity: f32,
}

pub(crate) fn execute_shutter_blur(
    renderer: &mut WgpuSceneRenderer,
    host_id: &amigo_composite_plugin::PostFxHost2dId,
    effect_id: &amigo_composite_plugin::PostFx2dId,
    effect: ShutterBlur2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let key = super::runtime_key::PostFxRuntimeKey::new(host_id, effect_id);
    let mut runtime = renderer
        .shutter_blur_runtimes
        .remove(&key)
        .unwrap_or_default();
    let result = runtime.execute(renderer, effect, input_view, output);
    renderer.shutter_blur_runtimes.insert(key, runtime);
    result
}

pub(crate) fn execute_motion_debug(
    renderer: &mut WgpuSceneRenderer,
    host_id: &amigo_composite_plugin::PostFxHost2dId,
    effect_id: &amigo_composite_plugin::PostFx2dId,
    effect: ShutterBlur2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let key = super::runtime_key::PostFxRuntimeKey::new(host_id, effect_id);
    let mut runtime = renderer
        .shutter_blur_runtimes
        .remove(&key)
        .unwrap_or_default();
    let result = runtime.execute_with_mode(renderer, effect, input_view, output, true);
    renderer.shutter_blur_runtimes.insert(key, runtime);
    result
}

impl ShutterBlurRuntime {
    fn execute(
        &mut self,
        renderer: &mut WgpuSceneRenderer,
        effect: ShutterBlur2d,
        input_view: &wgpu::TextureView,
        output: &mut WgpuOffscreenTarget,
    ) -> AmigoResult<()> {
        self.execute_with_mode(renderer, effect, input_view, output, false)
    }

    fn execute_with_mode(
        &mut self,
        renderer: &mut WgpuSceneRenderer,
        effect: ShutterBlur2d,
        input_view: &wgpu::TextureView,
        output: &mut WgpuOffscreenTarget,
        debug_motion: bool,
    ) -> AmigoResult<()> {
        let effect = effect.normalized();
        if !effect.is_active() {
            return renderer.copy_offscreen_to_offscreen(output, input_view);
        }

        let width = output.width.max(1);
        let height = output.height.max(1);
        self.ensure_history(&output.device, width, height, output.format);

        let now = Instant::now();
        let dt = self
            .last_frame
            .map(|last| (now - last).as_secs_f32().clamp(1.0 / 240.0, 0.25))
            .unwrap_or(1.0 / 60.0);
        self.last_frame = Some(now);
        let target_dt = 1.0 / effect.fps.max(1.0);

        let history_ready_a = self
            .history_a
            .as_ref()
            .map(|history| history.initialized)
            .unwrap_or(false);
        let history_ready_b = self
            .history_b
            .as_ref()
            .map(|history| history.initialized)
            .unwrap_or(false);
        let history_a = self.history_a.as_mut().expect("history A must be ensured");
        let history_b = self.history_b.as_mut().expect("history B must be ensured");
        let sampler = output.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("amigo-shutter-blur-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = output.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-shutter-blur-texture-bind-group"),
            layout: &renderer.shutter_blur_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&history_a.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&history_b.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let uniforms = ShutterBlurUniform {
            resolution: [width as f32, height as f32],
            opacity: effect.opacity,
            shutter_fraction: (effect.shutter_angle / 360.0).clamp(0.0, 1.0),
            exposure_seconds: effect.exposure_seconds,
            history_mix: effect.history_mix,
            history_mix_2: effect.history_mix_2,
            edge_rejection: effect.edge_rejection,
            luma_threshold: effect.luma_threshold,
            dt,
            target_dt,
            history_ready_a: if history_ready_a { 1.0 } else { 0.0 },
            history_ready_b: if history_ready_b { 1.0 } else { 0.0 },
            frame_hold: if effect.frame_hold { 1.0 } else { 0.0 },
            debug_motion: if debug_motion { 1.0 } else { 0.0 },
            _padding: 0.0,
        };
        let uniform_buffer = output
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-shutter-blur-uniform-buffer"),
                contents: bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let uniform_bind_group = output.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-shutter-blur-uniform-bind-group"),
            layout: &renderer.wet_reflections_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let vertices = fullscreen_vertices();
        let vertex_buffer = output
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-shutter-blur-vertex-buffer"),
                contents: bytes_of_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let history_texture_bind_group =
            output.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-shutter-blur-history-store-texture-bind-group"),
                layout: &renderer.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&output.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
        let history_a_texture_bind_group =
            output.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-shutter-blur-history-a-store-texture-bind-group"),
                layout: &renderer.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&history_a.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
        let history_uniforms = DownscaleUniform {
            resolution: [width as f32, height as f32],
            factor: 1.0,
            opacity: 1.0,
        };
        let history_uniform_buffer =
            output
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("amigo-shutter-blur-history-store-uniform-buffer"),
                    contents: bytes_of(&history_uniforms),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let history_uniform_bind_group =
            output.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-shutter-blur-history-store-uniform-bind-group"),
                layout: &renderer.wet_reflections_uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: history_uniform_buffer.as_entire_binding(),
                }],
            });

        let mut encoder = output
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-shutter-blur-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-shutter-blur-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&renderer.shutter_blur_pipeline);
            pass.set_bind_group(0, &texture_bind_group, &[]);
            pass.set_bind_group(1, &uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-shutter-blur-history-b-store-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &history_b.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&renderer.downscale_pipeline);
            pass.set_bind_group(0, &history_a_texture_bind_group, &[]);
            pass.set_bind_group(1, &history_uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-shutter-blur-history-a-store-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &history_a.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&renderer.downscale_pipeline);
            pass.set_bind_group(0, &history_texture_bind_group, &[]);
            pass.set_bind_group(1, &history_uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }

        output.queue.submit(Some(encoder.finish()));
        history_b.initialized = history_ready_a;
        history_a.initialized = true;
        Ok(())
    }

    fn ensure_history(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        let recreate = self
            .history_a
            .as_ref()
            .map(|history| {
                history.width != width || history.height != height || history.format != format
            })
            .unwrap_or(true)
            || self
                .history_b
                .as_ref()
                .map(|history| {
                    history.width != width || history.height != height || history.format != format
                })
                .unwrap_or(true);

        if !recreate {
            return;
        }

        let texture_a = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("amigo-shutter-blur-history-a-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view_a = texture_a.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_b = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("amigo-shutter-blur-history-b-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view_b = texture_b.create_view(&wgpu::TextureViewDescriptor::default());
        self.history_a = Some(ShutterBlurHistory {
            texture: texture_a,
            view: view_a,
            width,
            height,
            format,
            initialized: false,
        });
        self.history_b = Some(ShutterBlurHistory {
            texture: texture_b,
            view: view_b,
            width,
            height,
            format,
            initialized: false,
        });
    }
}

fn fullscreen_vertices() -> [TextureVertex; 6] {
    [
        texture_vertex(-1.0, -1.0, 0.0, 1.0),
        texture_vertex(1.0, -1.0, 1.0, 1.0),
        texture_vertex(1.0, 1.0, 1.0, 0.0),
        texture_vertex(-1.0, -1.0, 0.0, 1.0),
        texture_vertex(1.0, 1.0, 1.0, 0.0),
        texture_vertex(-1.0, 1.0, 0.0, 0.0),
    ]
}

fn texture_vertex(x: f32, y: f32, u: f32, v: f32) -> TextureVertex {
    TextureVertex {
        position: [x, y],
        uv: [u, v],
        color: [1.0, 1.0, 1.0, 1.0],
    }
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T) as *const u8, std::mem::size_of::<T>())
    }
}

fn bytes_of_slice<T>(slice: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, std::mem::size_of_val(slice)) }
}
