use amigo_composite_plugin::CameraOptics2d;
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Vec2};
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::service::WgpuSceneRenderer;

#[repr(C)]
#[derive(Clone, Copy)]
struct CameraOpticsUniform {
    focal_length_mm: f32,
    aberration_px: f32,
    distortion: f32,
    vignette: f32,
    edge_softness_px: f32,
    glare_strength: f32,
    lens_bloom: f32,
    flare_ghosts: f32,
    anamorphic_squeeze: f32,
    coma: f32,
    dirt: f32,
    halation_bias: f32,
    opacity: f32,
}

pub(crate) fn execute_camera_optics(
    renderer: &mut WgpuSceneRenderer,
    effect: CameraOptics2d,
    input_view: &wgpu::TextureView,
    normal_view: Option<&wgpu::TextureView>,
    wetness_view: Option<&wgpu::TextureView>,
    highlight_view: Option<&wgpu::TextureView>,
    emissive_view: Option<&wgpu::TextureView>,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let effect = effect.normalized();
    if !effect.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let device = &output.device;
    let queue = &output.queue;
    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-camera-optics-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-camera-optics-texture-bind-group"),
        layout: &renderer.camera_visual_source_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(input_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(normal_view.unwrap_or(input_view)),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(wetness_view.unwrap_or(input_view)),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(highlight_view.unwrap_or(input_view)),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(emissive_view.unwrap_or(input_view)),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&source_sampler),
            },
        ],
    });

    let uniforms = CameraOpticsUniform {
        focal_length_mm: effect.focal_length_mm,
        aberration_px: effect.aberration_px,
        distortion: effect.distortion,
        vignette: effect.vignette,
        edge_softness_px: effect.edge_softness_px,
        glare_strength: effect.glare_strength,
        lens_bloom: effect.lens_bloom,
        flare_ghosts: effect.flare_ghosts,
        anamorphic_squeeze: effect.anamorphic_squeeze,
        coma: effect.coma,
        dirt: effect.dirt,
        halation_bias: effect.halation_bias,
        opacity: effect.opacity,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-camera-optics-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-camera-optics-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-camera-optics-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-camera-optics-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-camera-optics-pass"),
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
        pass.set_pipeline(&renderer.camera_optics_pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
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
