use amigo_render_api::DirtyBloom2d;
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Vec2};
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::service::WgpuSceneRenderer;

#[repr(C)]
#[derive(Clone, Copy)]
struct DirtyBloomUniform {
    resolution: [f32; 2],
    time_seconds: f32,
    threshold: f32,
    strength: f32,
    small_radius_px: f32,
    medium_radius_px: f32,
    large_radius_px: f32,
    dirty_noise: f32,
    halation_strength: f32,
    reflection_smear_x_px: f32,
    reflection_smear_y_px: f32,
    seed: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HighlightExtractUniform {
    resolution: [f32; 2],
    threshold: f32,
    softness: f32,
}

pub(crate) fn execute_dirty_bloom(
    renderer: &mut WgpuSceneRenderer,
    bloom: DirtyBloom2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let bloom = bloom.normalized();
    if !bloom.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let device = &output.device;
    let queue = &output.queue;
    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-dirty-bloom-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-dirty-bloom-texture-bind-group"),
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

    let uniforms = DirtyBloomUniform {
        resolution: [output.width.max(1) as f32, output.height.max(1) as f32],
        time_seconds: runtime_time_seconds(),
        threshold: bloom.threshold,
        strength: bloom.strength,
        small_radius_px: bloom.small_radius_px,
        medium_radius_px: bloom.medium_radius_px,
        large_radius_px: bloom.large_radius_px,
        dirty_noise: bloom.dirty_noise,
        halation_strength: bloom.halation_strength,
        reflection_smear_x_px: bloom.reflection_smear_x_px,
        reflection_smear_y_px: bloom.reflection_smear_y_px,
        seed: bloom.seed as f32,
        _pad0: 0.0,
        _pad1: 0.0,
        _pad2: 0.0,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-dirty-bloom-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-dirty-bloom-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-dirty-bloom-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-dirty-bloom-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-dirty-bloom-pass"),
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
        pass.set_pipeline(renderer.post_fx_pipeline(
            crate::renderer::service::POST_FX_EXECUTOR_DIRTY_BLOOM,
        ));
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
}

pub(crate) fn execute_luma_extract_debug(
    renderer: &mut WgpuSceneRenderer,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
    threshold: f32,
    softness: f32,
    label_prefix: &'static str,
) -> AmigoResult<()> {
    let device = &output.device;
    let queue = &output.queue;
    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label_prefix),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label_prefix),
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
    let uniforms = HighlightExtractUniform {
        resolution: [output.width.max(1) as f32, output.height.max(1) as f32],
        threshold,
        softness,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label_prefix),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label_prefix),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label_prefix),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(label_prefix),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label_prefix),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output.view,
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
        pass.set_pipeline(renderer.post_fx_pipeline(
            crate::renderer::service::POST_FX_AUX_HIGHLIGHT_EXTRACT,
        ));
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
}

pub(crate) fn execute_highlight_extract_debug(
    renderer: &mut WgpuSceneRenderer,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    execute_luma_extract_debug(
        renderer,
        input_view,
        output,
        0.68,
        0.14,
        "amigo-highlight-extract-debug",
    )
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
