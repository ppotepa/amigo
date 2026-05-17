use amigo_2d_depth_map::{DepthMap2dDrawCommand, DepthMapViewportFit2d};
use amigo_2d_layered_image::LayeredImageSceneService;
use amigo_2d_post_fx::{FocusBlur2d, FocusTarget2d};
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::SceneService;
use wgpu::util::DeviceExt;

use crate::WgpuOffscreenTarget;
use crate::renderer::assets::resolve_image_path;
use crate::renderer::scene::resolve_camera2d_transform;
use crate::renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer};
use crate::renderer::world_2d::append_tinted_textured_sprite_vertices;
use crate::renderer::{TextureBlendMode, TextureUvRect, TextureVertex, Viewport};

#[repr(C)]
#[derive(Clone, Copy)]
struct FocusBlurUniform {
    focus: [f32; 4],
    optics: [f32; 4],
    boost: [f32; 4],
    flags: [f32; 4],
    aperture: [f32; 4],
    highlight: [f32; 4],
    depth_override: [f32; 4],
}

pub(crate) fn execute_focus_blur(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    effect: FocusBlur2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    execute_focus_blur_with_depth_source(
        renderer,
        request,
        effect,
        input_view,
        output,
        FocusBlurDepthSource::DepthMap,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FocusBlurDepthSource {
    DepthMap,
    Plane { value: f32, blur_scale: f32 },
}

pub(crate) fn execute_focus_blur_with_depth_source(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    effect: FocusBlur2d,
    input_view: &wgpu::TextureView,
    output: &mut WgpuOffscreenTarget,
    depth_source: FocusBlurDepthSource,
) -> AmigoResult<()> {
    let effect = effect.normalized();
    if !effect.is_active() {
        return renderer.copy_offscreen_to_offscreen(output, input_view);
    }

    let (depth_view, invert_depth, depth_override) = match depth_source {
        FocusBlurDepthSource::DepthMap => {
            let Some(depth_command) = resolve_depth_map_command(request, &effect) else {
                return renderer.copy_offscreen_to_offscreen(output, input_view);
            };
            let Some((_depth_texture, depth_view)) =
                render_depth_map_texture(renderer, request, output, &depth_command)
            else {
                return renderer.copy_offscreen_to_offscreen(output, input_view);
            };
            (
                depth_view,
                effect.invert_depth ^ !depth_command.depth_map.white_is_near,
                [0.0, 0.5, 1.0, 0.0],
            )
        }
        FocusBlurDepthSource::Plane { value, blur_scale } => (
            input_view.clone(),
            false,
            [1.0, value.clamp(0.0, 1.0), blur_scale.clamp(0.0, 4.0), 0.0],
        ),
    };

    let focus_uv = resolve_focus_uv(request, &effect).unwrap_or(Vec2::new(0.5, 0.5));
    let focus_depth = match &effect.focus {
        FocusTarget2d::Depth { value } => (*value).clamp(0.0, 1.0),
        _ => -1.0,
    };
    let device = &output.device;
    let queue = &output.queue;
    let source_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-focus-blur-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-focus-blur-texture-bind-group"),
        layout: &renderer.focus_blur_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(input_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&source_sampler),
            },
        ],
    });
    let uniforms = FocusBlurUniform {
        focus: [focus_uv.x, focus_uv.y, focus_depth, effect.f_stop],
        optics: [
            effect.focal_length_mm,
            effect.max_blur_px,
            effect.depth_contrast,
            effect.focus_width,
        ],
        boost: [
            effect.foreground_blur_boost,
            effect.background_blur_boost,
            effect.opacity,
            if effect.edge_aware { 1.0 } else { 0.0 },
        ],
        flags: [
            if invert_depth { 1.0 } else { 0.0 },
            effect.debug_view.shader_value(),
            effect.anamorphic_ratio,
            effect.cat_eye_bokeh,
        ],
        aperture: [
            effect.aperture_blades as f32,
            effect.aperture_roundness,
            effect.aperture_rotation_degrees.to_radians(),
            effect.sample_count.min(64) as f32,
        ],
        highlight: [
            effect.highlight_threshold,
            effect.highlight_knee,
            effect.highlight_gain,
            effect.highlight_saturation,
        ],
        depth_override,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-focus-blur-uniform-buffer"),
        contents: bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-focus-blur-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let vertices = fullscreen_vertices();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-focus-blur-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-focus-blur-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-focus-blur-pass"),
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
        pass.set_pipeline(&renderer.focus_blur_pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
}

fn resolve_depth_map_command(
    request: &WgpuFrameRenderRequest<'_>,
    effect: &FocusBlur2d,
) -> Option<DepthMap2dDrawCommand> {
    let depth_map = effect.depth_map.as_deref()?.trim();
    if depth_map.is_empty() {
        return None;
    }

    request
        .world_2d
        .depth_maps
        .commands()
        .into_iter()
        .find(|command| {
            command.depth_map.id == depth_map
                || command.entity_name == depth_map
                || command.depth_map.asset.as_str() == depth_map
        })
}

fn render_depth_map_texture(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    output: &WgpuOffscreenTarget,
    command: &DepthMap2dDrawCommand,
) -> Option<(wgpu::Texture, wgpu::TextureView)> {
    let device = &output.device;
    let queue = &output.queue;
    let prepared = request.assets.prepared_asset(&command.depth_map.asset)?;
    let image_path = resolve_image_path(&prepared)?;
    let (source_bind_group, source_size) = {
        let texture = renderer.ensure_data_texture_from_path(
            device,
            queue,
            format!("depth-map-data:{}", command.depth_map.asset.as_str()),
            image_path,
            true,
            false,
        )?;
        (texture.bind_group.clone(), texture.dimensions())
    };

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("amigo-focus-blur-depth-map-texture"),
        size: wgpu::Extent3d {
            width: output.width.max(1),
            height: output.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: output.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let viewport = Viewport::from_offscreen(output);
    let camera = resolve_camera2d_transform(request.scene, request.active_camera_2d_entity);
    let transform = request
        .scene
        .transform_of(&command.entity_name)
        .map(|value| Transform2 {
            translation: Vec2::new(value.translation.x, value.translation.y),
            rotation_radians: value.rotation_euler.z,
            scale: Vec2::new(value.scale.x, value.scale.y),
        })
        .unwrap_or(command.transform);
    let size = depth_map_render_size(
        &viewport,
        command.depth_map.size,
        source_size,
        command.depth_map.viewport_fit,
    );
    let mut vertices = Vec::with_capacity(6);
    append_tinted_textured_sprite_vertices(
        &mut vertices,
        &viewport,
        camera,
        transform,
        size,
        TextureUvRect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        },
        ColorRgba::WHITE,
    );
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-focus-blur-depth-map-vertex-buffer"),
        contents: bytes_of_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let clear = if command.depth_map.white_is_near {
        0.0
    } else {
        1.0
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-focus-blur-depth-map-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-focus-blur-depth-map-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &depth_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear,
                        g: clear,
                        b: clear,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(renderer.texture_pipeline_for(TextureBlendMode::Alpha));
        pass.set_bind_group(0, &source_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
    queue.submit(Some(encoder.finish()));

    Some((depth_texture, depth_view))
}

fn depth_map_render_size(
    viewport: &Viewport,
    fixed_size: Vec2,
    source_size: Vec2,
    fit: DepthMapViewportFit2d,
) -> Vec2 {
    let viewport_size = viewport.size();
    let source_size = if source_size.x > 0.0 && source_size.y > 0.0 {
        source_size
    } else {
        viewport_size
    };
    let fixed_size = if fixed_size.x > 0.0 && fixed_size.y > 0.0 {
        fixed_size
    } else {
        source_size
    };
    match fit {
        DepthMapViewportFit2d::Fixed => fixed_size,
        DepthMapViewportFit2d::Stretch => viewport_size,
        DepthMapViewportFit2d::Contain => scaled_to_viewport(source_size, viewport_size, f32::min),
        DepthMapViewportFit2d::Cover => scaled_to_viewport(source_size, viewport_size, f32::max),
    }
}

fn scaled_to_viewport(
    source_size: Vec2,
    viewport_size: Vec2,
    choose_scale: impl Fn(f32, f32) -> f32,
) -> Vec2 {
    let scale = choose_scale(
        viewport_size.x / source_size.x,
        viewport_size.y / source_size.y,
    );
    Vec2::new(source_size.x * scale, source_size.y * scale)
}

fn resolve_focus_uv(request: &WgpuFrameRenderRequest<'_>, effect: &FocusBlur2d) -> Option<Vec2> {
    match &effect.focus {
        FocusTarget2d::None => None,
        FocusTarget2d::Depth { value } => Some(Vec2::new(0.5, value.clamp(0.0, 1.0))),
        FocusTarget2d::SceneObject { object } => {
            let transform = request.scene.transform_of(object)?;
            Some(world_to_uv(
                request.scene,
                request.active_camera_2d_entity,
                request.target.width() as f32,
                request.target.height() as f32,
                Transform2 {
                    translation: Vec2::new(transform.translation.x, transform.translation.y),
                    rotation_radians: transform.rotation_euler.z,
                    scale: Vec2::new(transform.scale.x, transform.scale.y),
                }
                .translation,
            ))
        }
        FocusTarget2d::RenderLayer { layer } => average_render_layer_uv(request, layer),
    }
}

fn average_render_layer_uv(request: &WgpuFrameRenderRequest<'_>, layer: &str) -> Option<Vec2> {
    let mut samples = Vec::new();
    sample_layered_image_positions(
        request.world_2d.layered_images,
        layer,
        request.scene,
        request.active_camera_2d_entity,
        request.target.width() as f32,
        request.target.height() as f32,
        &mut samples,
    );
    sample_layer_positions(
        request.world_2d.sprites,
        layer,
        request.scene,
        request.active_camera_2d_entity,
        request.target.width() as f32,
        request.target.height() as f32,
        &mut samples,
    );
    sample_text_positions(
        request.world_2d.text2d,
        layer,
        request.scene,
        request.active_camera_2d_entity,
        request.target.width() as f32,
        request.target.height() as f32,
        &mut samples,
    );
    if samples.is_empty() {
        return None;
    }
    let sum = samples.iter().fold(Vec2::ZERO, |acc, value| {
        Vec2::new(acc.x + value.x, acc.y + value.y)
    });
    let count = samples.len() as f32;
    Some(Vec2::new(sum.x / count, sum.y / count))
}

fn sample_layered_image_positions(
    layered_images: &LayeredImageSceneService,
    layer: &str,
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
    width: f32,
    height: f32,
    out: &mut Vec<Vec2>,
) {
    for command in layered_images
        .commands()
        .into_iter()
        .filter(|command| command.render_layer == layer)
    {
        let transform = scene
            .transform_of(&command.entity_name)
            .map(|value| Transform2 {
                translation: Vec2::new(value.translation.x, value.translation.y),
                rotation_radians: value.rotation_euler.z,
                scale: Vec2::new(value.scale.x, value.scale.y),
            })
            .unwrap_or(command.transform);
        out.push(world_to_uv(
            scene,
            active_camera_2d_entity,
            width,
            height,
            transform.translation,
        ));
    }
}

fn sample_layer_positions(
    sprites: &SpriteSceneService,
    layer: &str,
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
    width: f32,
    height: f32,
    out: &mut Vec<Vec2>,
) {
    for command in sprites
        .commands()
        .into_iter()
        .filter(|command| command.render_layer == layer)
    {
        let transform = scene
            .transform_of(&command.entity_name)
            .map(|value| Transform2 {
                translation: Vec2::new(value.translation.x, value.translation.y),
                rotation_radians: value.rotation_euler.z,
                scale: Vec2::new(value.scale.x, value.scale.y),
            })
            .unwrap_or(command.transform);
        out.push(world_to_uv(
            scene,
            active_camera_2d_entity,
            width,
            height,
            transform.translation,
        ));
    }
}

fn sample_text_positions(
    text: &Text2dSceneService,
    layer: &str,
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
    width: f32,
    height: f32,
    out: &mut Vec<Vec2>,
) {
    for command in text
        .commands()
        .into_iter()
        .filter(|command| command.render_layer == layer)
    {
        let transform = scene
            .transform_of(&command.entity_name)
            .map(|value| Transform2 {
                translation: Vec2::new(value.translation.x, value.translation.y),
                rotation_radians: value.rotation_euler.z,
                scale: Vec2::new(value.scale.x, value.scale.y),
            })
            .unwrap_or(command.text.transform);
        out.push(world_to_uv(
            scene,
            active_camera_2d_entity,
            width,
            height,
            transform.translation,
        ));
    }
}

fn world_to_uv(
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
    width: f32,
    height: f32,
    world: Vec2,
) -> Vec2 {
    let camera = resolve_camera2d_transform(scene, active_camera_2d_entity);
    Vec2::new(
        ((world.x - camera.translation.x) / width + 0.5).clamp(0.0, 1.0),
        (0.5 - (world.y - camera.translation.y) / height).clamp(0.0, 1.0),
    )
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
