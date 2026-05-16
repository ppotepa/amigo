use amigo_2d_post_fx::FilmNoise2d;
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Vec2};
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::service::WgpuSceneRenderer;

#[repr(C)]
#[derive(Clone, Copy)]
struct FilmNoiseUniform {
    resolution: [f32; 2],
    time_seconds: f32,
    iso: f32,
    grain_size: f32,
    chroma_noise: f32,
    color_shift: f32,
    contrast: f32,
    saturation: f32,
    flicker: f32,
    vignette: f32,
    toe: f32,
    shoulder: f32,
    black_lift: f32,
    print_fade: f32,
    dust: f32,
    scratches: f32,
    push_pull: f32,
    gate_weave: f32,
    scan_softness: f32,
    opacity: f32,
    seed: f32,
    _pad0: [f32; 3],
}

pub(crate) fn execute_film_noise(
    renderer: &mut WgpuSceneRenderer,
    noise: FilmNoise2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let noise = noise.normalized();
    if !noise.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let device = &output.device;
    let queue = &output.queue;
    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-film-noise-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-film-noise-texture-bind-group"),
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

    let uniforms = FilmNoiseUniform {
        resolution: [output.width.max(1) as f32, output.height.max(1) as f32],
        time_seconds: runtime_time_seconds(),
        iso: noise.iso,
        grain_size: noise.grain_size,
        chroma_noise: noise.chroma_noise,
        color_shift: noise.color_shift,
        contrast: noise.contrast,
        saturation: noise.saturation,
        flicker: noise.flicker,
        vignette: noise.vignette,
        toe: noise.toe,
        shoulder: noise.shoulder,
        black_lift: noise.black_lift,
        print_fade: noise.print_fade,
        dust: noise.dust,
        scratches: noise.scratches,
        push_pull: noise.push_pull,
        gate_weave: noise.gate_weave,
        scan_softness: noise.scan_softness,
        opacity: noise.opacity,
        seed: noise.seed as f32,
        _pad0: [0.0, 0.0, 0.0],
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-film-noise-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-film-noise-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-film-noise-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-film-noise-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-film-noise-pass"),
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
        pass.set_pipeline(&renderer.film_noise_pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
}

fn runtime_time_seconds() -> f32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f32())
        .unwrap_or_default()
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
