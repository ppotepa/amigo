use amigo_2d_post_fx::{ColorQuantize2d, ColorRamp2d};
use amigo_core::AmigoResult;
use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};
use amigo_math::{ColorRgba, Vec2};
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::service::WgpuSceneRenderer;

#[repr(C)]
#[derive(Clone, Copy)]
struct ColorQuantizeUniform {
    resolution: [f32; 2],
    palette_size: f32,
    dither_strength: f32,
    dither_scale: f32,
    layered_dither: f32,
    opacity: f32,
    luma_preserve: f32,
    highlight_bias: f32,
    shadow_bias: f32,
    contrast: f32,
    saturation: f32,
    gamma: f32,
    seed: f32,
    _padding: [f32; 2],
}

pub(crate) fn execute_color_quantize(
    renderer: &mut WgpuSceneRenderer,
    effect: ColorQuantize2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    execute_color_ramp(renderer, ColorRamp2d::from(effect), input_view, output)
}

pub(crate) fn execute_color_ramp(
    renderer: &mut WgpuSceneRenderer,
    effect: ColorRamp2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let effect = effect.normalized();
    if !effect.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let device = &output.device;
    let queue = &output.queue;
    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-color-quantize-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-color-quantize-texture-bind-group"),
        layout: &renderer.texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(input_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&source_sampler),
            },
        ],
    });
    let (palette_texture, palette_view) =
        create_color_quantize_palette(device, queue, effect.palette_size);
    let palette_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-color-quantize-palette-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let palette_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-color-quantize-palette-bind-group"),
        layout: &renderer.texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&palette_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&palette_sampler),
            },
        ],
    });

    let uniforms = ColorQuantizeUniform {
        resolution: [output.width.max(1) as f32, output.height.max(1) as f32],
        palette_size: effect.palette_size as f32,
        dither_strength: effect.dither_strength,
        dither_scale: effect.dither_scale,
        layered_dither: effect.layered_dither,
        opacity: effect.opacity,
        luma_preserve: effect.luma_preserve,
        highlight_bias: effect.highlight_bias,
        shadow_bias: effect.shadow_bias,
        contrast: effect.contrast,
        saturation: effect.saturation,
        gamma: effect.gamma,
        seed: effect.seed as f32,
        _padding: [0.0; 2],
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-color-quantize-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-color-quantize-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-color-quantize-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-color-quantize-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-color-quantize-pass"),
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
        pass.set_pipeline(&renderer.color_quantize_pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_bind_group(2, &palette_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    drop(palette_texture);
    Ok(())
}

fn create_color_quantize_palette(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    palette_size: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let width = palette_size.clamp(2, 256);
    let ramp = lofi_cartoon_ramp(width);
    let mut bytes = Vec::with_capacity(width as usize * 4);
    for index in 0..width {
        let t = if width <= 1 {
            0.0
        } else {
            index as f32 / (width - 1) as f32
        };
        let color = ramp.sample(t);
        bytes.extend_from_slice(&[
            (color.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (color.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (color.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]);
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("amigo-color-quantize-palette-texture"),
        size: wgpu::Extent3d {
            width,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn lofi_cartoon_ramp(palette_size: u32) -> ColorRamp {
    let steps = palette_size.clamp(2, 256);
    let mut stops = Vec::with_capacity(steps as usize);
    for index in 0..steps {
        let t = if steps <= 1 {
            0.0
        } else {
            index as f32 / (steps - 1) as f32
        };
        let shaped = (t * 1.08).powf(1.35).clamp(0.0, 1.0);
        stops.push(ColorStop {
            t,
            color: ColorRgba::new(shaped, shaped, shaped, 1.0),
        });
    }
    ColorRamp {
        stops,
        interpolation: ColorInterpolation::Step,
    }
}

fn fullscreen_vertices() -> [TextureVertex; 6] {
    [
        TextureVertex::new(Vec2::new(-1.0, -1.0), Vec2::new(0.0, 1.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(1.0, -1.0), Vec2::new(1.0, 1.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(-1.0, -1.0), Vec2::new(0.0, 1.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), ColorRgba::WHITE),
        TextureVertex::new(Vec2::new(-1.0, 1.0), Vec2::new(0.0, 0.0), ColorRgba::WHITE),
    ]
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T) as *const u8, std::mem::size_of::<T>())
    }
}

fn bytes_of_slice<T>(slice: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, std::mem::size_of_val(slice)) }
}
