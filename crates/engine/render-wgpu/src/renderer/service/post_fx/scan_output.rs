use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Vec2};
use amigo_render_api::ScanOutput2d;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::service::WgpuSceneRenderer;

static SCAN_OUTPUT_FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
#[derive(Clone, Copy)]
struct ScanOutputUniform {
    resolution: [f32; 2],
    time_seconds: f32,
    iso: f32,
    grain_chroma: f32,
    _grain_padding0: f32,
    flicker: f32,
    vignette: f32,
    print_fade: f32,
    dust: f32,
    scratches: f32,
    gate_weave: f32,
    scan_softness: f32,
    opacity: f32,
    seed: f32,
    grain_luma: f32,
    shadow_grain: f32,
    midtone_grain: f32,
    highlight_grain: f32,
    highlight_suppression: f32,
    fine_grain_px: f32,
    medium_grain_px: f32,
    coarse_grain_px: f32,
    clumpiness: f32,
    grain_softness: f32,
    underexposure_grain_boost: f32,
    push_process_boost: f32,
    density_pivot: f32,
    temporal_jitter: f32,
    grain_regenerate_per_frame: f32,
    grain_frame: f32,
    channel_balance_r: f32,
    channel_balance_g: f32,
    channel_balance_b: f32,
    _padding0: f32,
    _padding1: f32,
}

pub(crate) fn execute_scan_output(
    renderer: &mut WgpuSceneRenderer,
    effect: ScanOutput2d,
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
        label: Some("amigo-scan-output-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-scan-output-texture-bind-group"),
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

    let grain_frame = if effect.grain_regenerate_per_frame {
        SCAN_OUTPUT_FRAME_COUNTER.fetch_add(1, Ordering::Relaxed) as f32
    } else {
        0.0
    };

    let uniforms = ScanOutputUniform {
        resolution: [output.width.max(1) as f32, output.height.max(1) as f32],
        time_seconds: runtime_time_seconds(),
        iso: effect.iso,
        grain_chroma: effect.grain_chroma,
        _grain_padding0: 0.0,
        flicker: effect.flicker,
        vignette: effect.vignette,
        print_fade: effect.print_fade,
        dust: effect.dust,
        scratches: effect.scratches,
        gate_weave: effect.gate_weave,
        scan_softness: effect.scan_softness,
        opacity: effect.opacity,
        seed: effect.seed as f32,
        grain_luma: effect.grain_luma,
        shadow_grain: effect.shadow_grain,
        midtone_grain: effect.midtone_grain,
        highlight_grain: effect.highlight_grain,
        highlight_suppression: effect.highlight_suppression,
        fine_grain_px: effect.fine_grain_px,
        medium_grain_px: effect.medium_grain_px,
        coarse_grain_px: effect.coarse_grain_px,
        clumpiness: effect.clumpiness,
        grain_softness: effect.grain_softness,
        underexposure_grain_boost: effect.underexposure_grain_boost,
        push_process_boost: effect.push_process_boost,
        density_pivot: effect.density_pivot,
        temporal_jitter: effect.temporal_jitter,
        grain_regenerate_per_frame: if effect.grain_regenerate_per_frame {
            1.0
        } else {
            0.0
        },
        grain_frame,
        channel_balance_r: effect.channel_balance[0],
        channel_balance_g: effect.channel_balance[1],
        channel_balance_b: effect.channel_balance[2],
        _padding0: 0.0,
        _padding1: 0.0,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-scan-output-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-scan-output-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-scan-output-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-scan-output-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-scan-output-pass"),
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
        pass.set_pipeline(
            renderer.post_fx_pipeline(crate::renderer::service::POST_FX_EXECUTOR_SCAN_OUTPUT),
        );
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
