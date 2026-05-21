use amigo_composite_plugin::{FocusBlur2d, FocusTarget2d};
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_render_api::{
    RenderDepthMap2d, RenderDepthMapViewportFit2d, RenderPrimitive2d, Renderable2dItem,
};
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
    let highlight_view = renderer
        .visual_source_targets_2d
        .scene_highlight
        .as_ref()
        .map(|target| target.view.clone());
    execute_focus_blur_with_depth_source(
        renderer,
        request,
        effect,
        input_view,
        highlight_view.as_ref(),
        output,
        FocusBlurDepthSource::DepthMap,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FocusBlurDepthSource {
    DepthMap,
    ZDepth { z_depth: f32, blur_scale: f32 },
}

pub(crate) fn execute_focus_blur_with_depth_source(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    effect: FocusBlur2d,
    input_view: &wgpu::TextureView,
    highlight_view: Option<&wgpu::TextureView>,
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
                effect.invert_depth ^ !depth_command.white_is_near,
                [0.0, 0.5, 1.0, 0.0],
            )
        }
        FocusBlurDepthSource::ZDepth {
            z_depth,
            blur_scale,
        } => (
            input_view.clone(),
            false,
            [
                1.0,
                z_depth.clamp(0.0, 1.0),
                blur_scale.clamp(0.0, 4.0),
                0.0,
            ],
        ),
    };

    let focus_uv = resolve_focus_uv(request, &effect).unwrap_or(Vec2::new(0.5, 0.5));
    let focus_depth = resolve_focus_depth(request, &effect).unwrap_or(-1.0);
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
                resource: wgpu::BindingResource::TextureView(highlight_view.unwrap_or(input_view)),
            },
            wgpu::BindGroupEntry {
                binding: 3,
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
        pass.set_pipeline(renderer.post_fx_pipeline(
            crate::renderer::service::POST_FX_EXECUTOR_FOCUS_BLUR,
        ));
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    Ok(())
}

fn resolve_depth_map_command<'a>(
    request: &'a WgpuFrameRenderRequest<'_>,
    effect: &FocusBlur2d,
) -> Option<&'a RenderDepthMap2d> {
    let depth_map = effect.depth_map.as_deref()?.trim();
    if depth_map.is_empty() {
        return None;
    }

    request
        .world_2d
        .depth_maps
        .into_iter()
        .find(|command| {
            command.id == depth_map
                || command.owner_entity == depth_map
                || command.asset.as_str() == depth_map
        })
}

fn render_depth_map_texture(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    output: &WgpuOffscreenTarget,
    command: &RenderDepthMap2d,
) -> Option<(wgpu::Texture, wgpu::TextureView)> {
    let device = &output.device;
    let queue = &output.queue;
    let prepared = request.assets.prepared_asset(&command.asset)?;
    let image_path = resolve_image_path(&prepared)?;
    let (source_bind_group, source_size) = {
        let texture = renderer.ensure_data_texture_from_path(
            device,
            queue,
            format!("depth-map-data:{}", command.asset.as_str()),
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
        .transform_of(&command.owner_entity)
        .map(|value| Transform2 {
            translation: Vec2::new(value.translation.x, value.translation.y),
            rotation_radians: value.rotation_euler.z,
            scale: Vec2::new(value.scale.x, value.scale.y),
        })
        .unwrap_or(command.transform);
    let size = depth_map_render_size(
        &viewport,
        command.size,
        source_size,
        command.viewport_fit,
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
    let clear = if command.white_is_near {
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
    fit: RenderDepthMapViewportFit2d,
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
        RenderDepthMapViewportFit2d::Fixed => fixed_size,
        RenderDepthMapViewportFit2d::Stretch => viewport_size,
        RenderDepthMapViewportFit2d::Contain => {
            scaled_to_viewport(source_size, viewport_size, f32::min)
        }
        RenderDepthMapViewportFit2d::Cover => {
            scaled_to_viewport(source_size, viewport_size, f32::max)
        }
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
        FocusTarget2d::Distance { meters } => Some(Vec2::new(
            0.5,
            amigo_2d_spatial::distance_to_z_depth(
                *meters,
                amigo_2d_spatial::DepthSpace2d::default(),
            ),
        )),
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

fn resolve_focus_depth(request: &WgpuFrameRenderRequest<'_>, effect: &FocusBlur2d) -> Option<f32> {
    resolve_focus_depth_for_target(&effect.focus, request.camera_capture_input_2d)
}

fn resolve_focus_depth_for_target(
    focus: &FocusTarget2d,
    capture_input: Option<&amigo_render_api::CameraCaptureInput2d>,
) -> Option<f32> {
    match focus {
        FocusTarget2d::None => None,
        FocusTarget2d::Depth { value } => Some(value.clamp(0.0, 1.0)),
        FocusTarget2d::Distance { meters } => Some(amigo_2d_spatial::distance_to_z_depth(
            *meters,
            capture_input
                .map(|input| input.depth_space)
                .unwrap_or_default(),
        )),
        FocusTarget2d::RenderLayer { layer } => capture_input.and_then(|input| {
            input
                .layers
                .iter()
                .find(|candidate| candidate.layer_id == *layer)
                .map(|candidate| candidate.z_depth.clamp(0.0, 1.0))
        }),
        FocusTarget2d::SceneObject { .. } => None,
    }
}

fn average_render_layer_uv(request: &WgpuFrameRenderRequest<'_>, layer: &str) -> Option<Vec2> {
    let mut samples = Vec::new();
    sample_renderable_positions(
        request.world_2d.renderables,
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

fn sample_renderable_positions(
    renderables: &[Renderable2dItem],
    layer: &str,
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
    width: f32,
    height: f32,
    out: &mut Vec<Vec2>,
) {
    for renderable in renderables.iter().filter(|item| item.render_layer() == layer) {
        let world = match &renderable.primitive {
            RenderPrimitive2d::TexturedQuad(primitive) => primitive.transform.translation,
            RenderPrimitive2d::GlyphRun(primitive) => primitive.transform.translation,
            RenderPrimitive2d::LayeredTexturedQuads(primitive) => primitive.transform.translation,
            RenderPrimitive2d::ParticleBatch(primitive) => primitive.position,
            _ => continue,
        };
        out.push(world_to_uv(
            scene,
            active_camera_2d_entity,
            width,
            height,
            world,
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

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_2d_spatial::{DepthCurve2d, DepthSpace2d, OpticalLayerRole2d};
    use amigo_render_api::{
        CameraCaptureInput2d, ResolvedLayerOptics2d, VisualSourceKind2d, VisualSourceOrigin2d,
        VisualSourceRef2d,
    };

    fn capture_input(depth_space: DepthSpace2d) -> CameraCaptureInput2d {
        CameraCaptureInput2d {
            depth_space,
            color: VisualSourceRef2d::produced(
                VisualSourceKind2d::SceneColor,
                "world.color",
                VisualSourceOrigin2d::WorldPass,
            ),
            depth: None,
            layer_mask: None,
            normal: None,
            wetness: None,
            emissive: None,
            highlight: None,
            motion: None,
            layers: Vec::new(),
        }
    }

    #[test]
    fn resolve_focus_depth_uses_capture_depth_space_for_distance_focus() {
        let input = capture_input(DepthSpace2d {
            near_m: 1.0,
            far_m: 1500.0,
            curve: DepthCurve2d::Logarithmic,
        });
        let expected = amigo_2d_spatial::distance_to_z_depth(75.0, input.depth_space);
        let actual =
            resolve_focus_depth_for_target(&FocusTarget2d::Distance { meters: 75.0 }, Some(&input))
                .expect("focus depth should resolve");
        assert!((actual - expected).abs() < 0.0001);
    }

    #[test]
    fn resolve_focus_depth_uses_render_layer_z_depth_from_capture_layers() {
        let mut input = capture_input(DepthSpace2d::default());
        input.layers.push(ResolvedLayerOptics2d {
            layer_id: "weather.rain.mid".to_owned(),
            role: OpticalLayerRole2d::SceneMedium,
            depth_mode: "distance".to_owned(),
            distance_m: Some(75.0),
            z_depth: 0.41,
            base_z_depth: 0.41,
            effective_z_depth: 0.41,
            effective_distance_m: Some(75.0),
            blur_scale: 0.25,
            camera_motion_scale: amigo_2d_spatial::z_depth_to_camera_motion_scale(0.41),
        });

        let actual = resolve_focus_depth_for_target(
            &FocusTarget2d::RenderLayer {
                layer: "weather.rain.mid".to_owned(),
            },
            Some(&input),
        );
        assert_eq!(actual, Some(0.41));
    }
}
