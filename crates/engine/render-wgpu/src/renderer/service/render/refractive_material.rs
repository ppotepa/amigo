use super::material_candidates::MaterialCandidate2d;
use super::offscreen_ops::{append_fullscreen_texture_vertices, compatible_offscreen_target};
use super::*;
use amigo_2d_sprite::SpriteDrawCommand;
use amigo_2d_text::Text2dDrawCommand;
use amigo_2d_vector::VectorShape2dDrawCommand;
use amigo_render_api::Material2d;
use wgpu::util::DeviceExt;

#[derive(Clone)]
pub(super) enum RefractiveMaterialMaskPayload {
    Text(Text2dDrawCommand),
    Sprite(SpriteDrawCommand),
    Vector(VectorShape2dDrawCommand),
}

#[derive(Clone)]
pub(super) struct RefractiveMaterialMaskCommand {
    pub(super) payload: RefractiveMaterialMaskPayload,
    pub(super) camera: Transform2,
    pub(super) material: Material2d,
    pub(super) layer_opacity: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RefractiveMaterialUniform {
    resolution: [f32; 2],
    transmission: f32,
    refraction_px: f32,
    distortion: f32,
    dispersion: f32,
    roughness: f32,
    edge_boost: f32,
    opacity: f32,
    highlight: f32,
}

pub(super) fn execute_refractive_material_2d(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    assets: &AssetCatalog,
    viewport: &Viewport,
    scene: &SceneService,
    mask_commands: &[RefractiveMaterialMaskCommand],
    candidates: &[MaterialCandidate2d],
) -> AmigoResult<()> {
    let active_commands = mask_commands
        .iter()
        .filter(|command| command.material.is_refractive() && command.layer_opacity > 0.001)
        .collect::<Vec<_>>();
    if active_commands.is_empty() {
        renderer.set_render_materials_last_summary(refractive_material_summary(
            candidates,
            MaterialPassState::Inactive("no_refractive_candidates"),
        ));
        return Ok(());
    }

    let mut scene_source = compatible_offscreen_target(target, "amigo-refractive-material-source");
    copy_target_texture(renderer, target, &mut scene_source)?;

    let mut mask_target = compatible_offscreen_target(target, "amigo-refractive-material-mask");
    let mut mask_batches = Vec::new();
    let mut mask_color_batches = Vec::new();
    let mut fallback_masks = 0usize;
    for command in &active_commands {
        match &command.payload {
            RefractiveMaterialMaskPayload::Text(text_command) => {
                let transform = resolve_transform2(
                    scene,
                    &text_command.entity_name,
                    text_command.text.transform,
                );
                let alpha = (text_command.text.style.color.a
                    * text_command.text.style.opacity
                    * command.layer_opacity)
                    .clamp(0.0, 1.0);
                if !renderer.append_text2d_ttf_font_texture_batch(
                    &mut mask_batches,
                    &mask_target.device,
                    &mask_target.queue,
                    assets,
                    viewport,
                    command.camera,
                    &text_command.text.font,
                    &text_command.text.content,
                    transform,
                    text_command.text.bounds,
                    text_command.text.style.font_size,
                    ColorRgba::new(1.0, 1.0, 1.0, alpha),
                ) {
                    let vertices =
                        color_batch_vertices(&mut mask_color_batches, ParticleBlendMode2d::Alpha);
                    append_text_2d_vertices(
                        vertices,
                        viewport,
                        command.camera,
                        &text_command.text.content,
                        transform,
                        text_command.text.bounds,
                        ColorRgba::new(1.0, 1.0, 1.0, alpha),
                    );
                    fallback_masks += 1;
                }
            }
            RefractiveMaterialMaskPayload::Sprite(sprite_command) => {
                let transform =
                    resolve_transform2(scene, &sprite_command.entity_name, sprite_command.transform);
                let _ = renderer.append_sprite_texture_batch(
                    &mut mask_batches,
                    &mask_target.device,
                    &mask_target.queue,
                    assets,
                    viewport,
                    command.camera,
                    transform,
                    &sprite_command.sprite,
                );
            }
            RefractiveMaterialMaskPayload::Vector(vector_command) => {
                let transform = vector_viewport_fit_transform(
                    viewport,
                    resolve_transform2(scene, &vector_command.entity_name, vector_command.transform),
                    vector_command.viewport_fit,
                    vector_command.viewport_canvas_size,
                );
                let vertices =
                    color_batch_vertices(&mut mask_color_batches, ParticleBlendMode2d::Alpha);
                append_vector_shape_vertices(
                    vertices,
                    viewport,
                    command.camera,
                    transform,
                    &vector_command.shape,
                );
            }
        }
    }

    if mask_batches.is_empty() && mask_color_batches.is_empty() {
        renderer.set_render_materials_last_summary(refractive_material_summary(
            candidates,
            MaterialPassState::Inactive("missing_mask_target"),
        ));
        return Ok(());
    }

    renderer.render_offscreen_batches(
        &mut mask_target,
        wgpu::LoadOp::Clear(wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }),
        &mask_batches,
        &mask_color_batches,
        &[],
    )?;

    let uniform = aggregate_uniform(target, &active_commands);
    let device = &target.device;
    let queue = &target.queue;
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-refractive-material-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-refractive-material-texture-bind-group"),
        layout: &renderer.focus_blur_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&scene_source.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&mask_target.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&mask_target.view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-refractive-material-uniform-buffer"),
        contents: bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-refractive-material-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let mut vertices = Vec::new();
    append_fullscreen_texture_vertices(&mut vertices);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-refractive-material-vertex-buffer"),
        contents: texture_vertices_as_bytes(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-refractive-material-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-refractive-material-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
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
        pass.set_pipeline(&renderer.refractive_material_pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    renderer.set_render_materials_last_summary(refractive_material_summary(
        candidates,
        MaterialPassState::Active {
            mask_commands: active_commands.len(),
            fallback_masks,
        },
    ));
    Ok(())
}

fn copy_target_texture(
    renderer: &mut WgpuSceneRenderer,
    source: &WgpuOffscreenTarget,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    renderer.copy_offscreen_to_offscreen(target, &source.view)
}

fn aggregate_uniform(
    target: &WgpuOffscreenTarget,
    commands: &[&RefractiveMaterialMaskCommand],
) -> RefractiveMaterialUniform {
    let mut transmission = 0.0f32;
    let mut refraction_px = 0.0f32;
    let mut distortion = 0.0f32;
    let mut dispersion = 0.0f32;
    let mut roughness = 0.0f32;
    let mut edge_boost = 0.0f32;
    let mut opacity = 0.0f32;
    let mut highlight = 0.0f32;
    let count = commands.len().max(1) as f32;

    for command in commands {
        let optical = command.material.optical;
        transmission += optical.transmission;
        refraction_px = refraction_px.max(optical.refraction_px);
        distortion += optical.distortion;
        dispersion += optical.dispersion;
        roughness += optical.roughness;
        edge_boost = edge_boost.max(optical.edge_boost);
        opacity = opacity.max(command.layer_opacity);
        highlight = highlight.max(command.material.camera_response.highlight);
    }

    RefractiveMaterialUniform {
        resolution: [target.width.max(1) as f32, target.height.max(1) as f32],
        transmission: (transmission / count).clamp(0.0, 1.0),
        refraction_px: refraction_px.max(0.0),
        distortion: (distortion / count).clamp(0.0, 1.0),
        dispersion: (dispersion / count).clamp(0.0, 1.0),
        roughness: (roughness / count).clamp(0.0, 1.0),
        edge_boost: edge_boost.clamp(0.0, 2.0),
        opacity: opacity.clamp(0.0, 1.0),
        highlight: highlight.clamp(0.0, 2.0),
    }
}

#[derive(Clone, Copy)]
enum MaterialPassState {
    Active {
        mask_commands: usize,
        fallback_masks: usize,
    },
    Inactive(&'static str),
}

fn refractive_material_summary(
    candidates: &[MaterialCandidate2d],
    state: MaterialPassState,
) -> String {
    let refractive = candidates
        .iter()
        .filter(|candidate| candidate.is_refractive())
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    lines.push("render.materials:".to_owned());
    lines.push(String::new());
    lines.push("refractive_material_2d:".to_owned());

    match state {
        MaterialPassState::Active {
            mask_commands,
            fallback_masks,
        } => {
            lines.push("active=true".to_owned());
            lines.push(format!("candidates={}", refractive.len()));
            lines.push(format!("mask_commands={mask_commands}"));
            lines.push(format!("fallback_masks={fallback_masks}"));
            lines.push("mask=input_ok".to_owned());
            lines.push(format!(
                "mask_source={}",
                if fallback_masks > 0 {
                    "ttf_font+fallback_geometry"
                } else {
                    "ttf_font_or_texture"
                }
            ));
            lines.push("scene_color=input_ok".to_owned());
            lines.push("output=composited_scene_color".to_owned());
        }
        MaterialPassState::Inactive(reason) => {
            lines.push("active=false".to_owned());
            lines.push(format!("reason={reason}"));
            lines.push(format!("candidates={}", refractive.len()));
        }
    }

    for candidate in refractive {
        let optical = candidate.material.optical;
        lines.push(String::new());
        lines.push(format!(
            "entity={} component={} layer={}",
            candidate.entity_name, candidate.component_kind, candidate.render_layer
        ));
        lines.push("material=optical.refractive".to_owned());
        lines.push(format!("coverage_source={}", candidate.coverage_label()));
        lines.push(format!("transmission={:.2}", optical.transmission));
        lines.push(format!("refraction_px={:.2}", optical.refraction_px));
        lines.push(format!("distortion={:.2}", optical.distortion));
        lines.push(format!(
            "highlight={:.2}",
            candidate.material.camera_response.highlight
        ));
        lines.push(format!("layer_opacity={:.2}", candidate.layer_opacity));
        lines.push(format!(
            "pass={}",
            if matches!(state, MaterialPassState::Active { .. }) {
                "active"
            } else {
                "skipped"
            }
        ));
    }

    lines.join("\n")
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_render_api::{Material2dCameraResponse, Material2dOptical, Material2dOpticalMode};

    #[test]
    fn refractive_material_summary_reports_real_composite_inputs() {
        let candidate = candidate();
        let summary = refractive_material_summary(
            &[candidate],
            MaterialPassState::Active {
                mask_commands: 1,
                fallback_masks: 0,
            },
        );

        assert!(summary.contains("active=true"));
        assert!(summary.contains("mask_commands=1"));
        assert!(summary.contains("fallback_masks=0"));
        assert!(summary.contains("mask=input_ok"));
        assert!(summary.contains("mask_source=ttf_font_or_texture"));
        assert!(summary.contains("scene_color=input_ok"));
        assert!(summary.contains("output=composited_scene_color"));
        assert!(summary.contains("entity=title component=Text2D layer=title.depth2d"));
        assert!(summary.contains("highlight=0.46"));
    }

    #[test]
    fn refractive_material_summary_reports_inactive_reason() {
        let summary = refractive_material_summary(&[], MaterialPassState::Inactive("no_refractive_candidates"));

        assert!(summary.contains("active=false"));
        assert!(summary.contains("reason=no_refractive_candidates"));
    }

    #[test]
    fn refractive_material_summary_reports_texture_alpha_candidate() {
        let candidate = MaterialCandidate2d {
            entity_name: "poster".to_owned(),
            component_kind: "Sprite2D",
            render_layer: "foreground.props".to_owned(),
            z_index: 0.0,
            material: refractive_material(),
            coverage_source: super::super::material_candidates::MaterialCoverageSource2d::TextureAlpha {
                entity_name: "poster".to_owned(),
                render_layer: "foreground.props".to_owned(),
            },
            layer_opacity: 1.0,
            visible: true,
        };

        let summary = refractive_material_summary(
            &[candidate],
            MaterialPassState::Active {
                mask_commands: 1,
                fallback_masks: 0,
            },
        );

        assert!(summary.contains("entity=poster component=Sprite2D layer=foreground.props"));
        assert!(summary.contains("coverage_source=texture_alpha"));
    }

    #[test]
    fn refractive_material_summary_reports_vector_coverage_candidate() {
        let candidate = MaterialCandidate2d {
            entity_name: "vector-glass".to_owned(),
            component_kind: "VectorShape2D",
            render_layer: "foreground.props".to_owned(),
            z_index: 0.0,
            material: refractive_material(),
            coverage_source: super::super::material_candidates::MaterialCoverageSource2d::VectorCoverage {
                entity_name: "vector-glass".to_owned(),
                render_layer: "foreground.props".to_owned(),
            },
            layer_opacity: 1.0,
            visible: true,
        };

        let summary = refractive_material_summary(
            &[candidate],
            MaterialPassState::Active {
                mask_commands: 1,
                fallback_masks: 0,
            },
        );

        assert!(summary.contains("entity=vector-glass component=VectorShape2D layer=foreground.props"));
        assert!(summary.contains("coverage_source=vector_coverage"));
    }

    fn candidate() -> MaterialCandidate2d {
        MaterialCandidate2d {
            entity_name: "title".to_owned(),
            component_kind: "Text2D",
            render_layer: "title.depth2d".to_owned(),
            z_index: 40.0,
            material: refractive_material(),
            coverage_source: super::super::material_candidates::MaterialCoverageSource2d::Glyphs {
                entity_name: "title".to_owned(),
                render_layer: "title.depth2d".to_owned(),
            },
            layer_opacity: 0.72,
            visible: true,
        }
    }

    fn refractive_material() -> Material2d {
        Material2d {
            optical: Material2dOptical {
                mode: Material2dOpticalMode::Refractive,
                transmission: 0.58,
                refraction_px: 4.5,
                distortion: 0.22,
                dispersion: 0.08,
                roughness: 0.32,
                edge_boost: 0.45,
            },
            camera_response: Material2dCameraResponse {
                highlight: 0.46,
                ..Default::default()
            },
            ..Default::default()
        }
        .normalized()
    }
}
