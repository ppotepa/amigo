use amigo_composite_plugin::PostFxWetReflections2d;
use amigo_assets::AssetKey;
use amigo_core::{AmigoError, AmigoResult};
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::TextureVertex;
use crate::renderer::assets::resolve_image_path;
use crate::renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer};

#[repr(C)]
#[derive(Clone, Copy)]
struct WetReflectionsUniform {
    resolution: [f32; 2],
    time_seconds: f32,
    mask_invert: f32,
    blur_px: f32,
    distortion_px: f32,
    shimmer_strength: f32,
    ripple_strength: f32,
    wet_darken: f32,
    specular_boost: f32,
    edge_power: f32,
    light_reflection_strength: f32,
    foreground_strength: f32,
    background_strength: f32,
    horizon_y: f32,
    noise_scale: f32,
    noise_speed: f32,
    ripple_speed: f32,
    debug_view: f32,
    _pad0: f32,
}

struct OwnedTexture {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl OwnedTexture {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

enum TextureRef {
    Cached(wgpu::TextureView),
    Owned(OwnedTexture),
}

impl TextureRef {
    fn view(&self) -> &wgpu::TextureView {
        match self {
            Self::Cached(view) => view,
            Self::Owned(owned) => owned.view(),
        }
    }
}

pub(crate) fn execute_wet_reflections(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    wet: PostFxWetReflections2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let wet = wet.normalized();
    if !wet.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let resolution = [output.width.max(1) as f32, output.height.max(1) as f32];
    let device = &output.device;
    let queue = &output.queue;

    let mask_view = load_texture_ref(
        renderer,
        device,
        queue,
        request.assets,
        &wet.reflection_mask,
        [0.0, 0.0, 0.0, 0.0],
    )?;
    let edge_view = wet
        .edge_map
        .as_deref()
        .map(|path| {
            load_texture_ref(
                renderer,
                device,
                queue,
                request.assets,
                path,
                [0.0, 0.0, 0.0, 0.0],
            )
        })
        .transpose()?
        .unwrap_or_else(|| {
            TextureRef::Owned(create_solid_texture(
                device,
                queue,
                "amigo-wet-reflections-edge-fallback",
                [0.0, 0.0, 0.0, 0.0],
            ))
        });
    let reflection_color_view = wet
        .reflection_color
        .as_deref()
        .map(|path| {
            load_texture_ref(
                renderer,
                device,
                queue,
                request.assets,
                path,
                [0.0, 0.0, 0.0, 0.0],
            )
        })
        .transpose()?
        .unwrap_or_else(|| {
            TextureRef::Owned(create_solid_texture(
                device,
                queue,
                "amigo-wet-reflections-color-fallback",
                [0.0, 0.0, 0.0, 0.0],
            ))
        });

    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-wet-reflections-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let uniforms = WetReflectionsUniform {
        resolution,
        time_seconds: 0.0,
        mask_invert: if wet.reflection_mask_invert { 1.0 } else { 0.0 },
        blur_px: wet.blur_px,
        distortion_px: wet.distortion_px,
        shimmer_strength: wet.shimmer_strength,
        ripple_strength: wet.ripple_strength,
        wet_darken: wet.wet_darken,
        specular_boost: wet.specular_boost,
        edge_power: wet.edge_power,
        light_reflection_strength: wet.light_reflection_strength,
        foreground_strength: wet.foreground_strength,
        background_strength: wet.background_strength,
        horizon_y: wet.horizon_y,
        noise_scale: wet.noise_scale,
        noise_speed: wet.noise_speed,
        ripple_speed: wet.ripple_speed,
        debug_view: wet.debug_view as u32 as f32,
        _pad0: 0.0,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-wet-reflections-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-wet-reflections-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-wet-reflections-texture-bind-group"),
        layout: &renderer.wet_reflections_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(input_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(mask_view.view()),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(edge_view.view()),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(reflection_color_view.view()),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&source_sampler),
            },
        ],
    });

    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-wet-reflections-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-wet-reflections-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-wet-reflections-pass"),
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
            crate::renderer::service::POST_FX_EXECUTOR_WET_REFLECTIONS,
        ));
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
}

pub(crate) fn render_texture_asset_debug(
    renderer: &mut WgpuSceneRenderer,
    assets: &amigo_assets::AssetCatalog,
    output: &mut WgpuOffscreenTarget,
    asset_path: &str,
    fallback: [f32; 4],
    label: &str,
) -> AmigoResult<()> {
    let texture = load_texture_ref(
        renderer,
        &output.device,
        &output.queue,
        assets,
        asset_path,
        fallback,
    )?;
    renderer.clear_offscreen_to_color(output, wgpu::Color::BLACK)?;
    renderer.composite_offscreen_over_offscreen(output, texture.view())?;
    let _ = label;
    Ok(())
}

fn load_texture_ref(
    renderer: &mut WgpuSceneRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    assets: &amigo_assets::AssetCatalog,
    asset_path: &str,
    fallback: [f32; 4],
) -> AmigoResult<TextureRef> {
    let key = AssetKey::new(asset_path);
    let Some(prepared) = assets.prepared_asset(&key) else {
        return Ok(TextureRef::Owned(create_solid_texture(
            device,
            queue,
            format!("amigo-wet-reflections-fallback:{asset_path}"),
            fallback,
        )));
    };
    let Some(image_path) = resolve_image_path(&prepared) else {
        return Ok(TextureRef::Owned(create_solid_texture(
            device,
            queue,
            format!("amigo-wet-reflections-fallback:{asset_path}"),
            fallback,
        )));
    };
    let Some(texture) = renderer.ensure_texture_from_path(
        device,
        queue,
        format!("wet-reflections:{asset_path}"),
        image_path,
        true,
        false,
    ) else {
        return Err(AmigoError::Message(format!(
            "failed to resolve wet reflections texture {asset_path}"
        )));
    };
    Ok(TextureRef::Cached(texture.view().clone()))
}

fn create_solid_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: impl Into<String>,
    rgba: [f32; 4],
) -> OwnedTexture {
    let label = label.into();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(Box::leak(label.into_boxed_str())),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let bytes = [
        (rgba[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgba[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgba[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (rgba[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    ];
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
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    OwnedTexture {
        _texture: texture,
        view,
    }
}

fn fullscreen_vertices() -> [TextureVertex; 6] {
    use amigo_math::ColorRgba;
    use amigo_math::Vec2;

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
        std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>())
    }
}

fn bytes_of_slice<T>(slice: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<u8>(), std::mem::size_of_val(slice)) }
}
