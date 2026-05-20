use super::material_candidates::{collect_material_candidate_2d, WgpuMaterialCandidate2d};
use super::world_filters::WorldPassLoad;
use super::*;
use amigo_text_2d_plugin::Text2dDrawCommand;
use amigo_material_2d_plugin::MaterialCandidateDecision2d;

#[derive(Clone, Copy)]
pub(super) struct WorldRenderContext<'a> {
    pub scene: &'a SceneService,
    pub assets: &'a AssetCatalog,
    pub renderables: &'a [Renderable2dItem],
    pub layered_images: &'a amigo_layered_image_2d_plugin::LayeredImageSceneService,
    pub global_lights: &'a GlobalLight2dSceneService,
    pub lightmaps: &'a LightMap2dSceneService,
    pub meshes: &'a [MeshDrawCommand],
    pub materials: &'a [MaterialDrawCommand],
    pub text3d: Option<&'a [Text3dDrawCommand]>,
    pub render_layers: &'a [RenderLayer2dCommand],
    pub light_routes: &'a [LightRoute2dCommand],
    pub light_groups: &'a [LightGroup2dCommand],
    pub particles: &'a [Particle2dDrawCommand],
    pub active_camera_2d_entity: Option<&'a str>,
}

impl<'a> WorldRenderContext<'a> {
    pub(super) fn from_request(request: &'a WgpuFrameRenderRequest<'a>) -> Self {
        Self {
            scene: request.scene,
            assets: request.assets,
            renderables: request.world_2d.renderables,
            layered_images: request.world_2d.layered_images,
            global_lights: request.world_2d.global_lights,
            lightmaps: request.world_2d.lightmaps,
            meshes: request.world_3d.meshes,
            materials: request.world_3d.materials,
            text3d: request.world_3d.text3d,
            render_layers: request.world_2d.render_layers,
            light_routes: request.world_2d.light_routes,
            light_groups: request.world_2d.light_groups,
            particles: request.world_2d.particles,
            active_camera_2d_entity: request.active_camera_2d_entity,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_text2d_draw_command(
    renderer: &mut WgpuSceneRenderer,
    texture_batches: &mut Vec<TextureBatch>,
    color_batches: &mut Vec<ColorBatch>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    assets: &AssetCatalog,
    viewport: &Viewport,
    camera2d: Transform2,
    scene: &SceneService,
    command: &Text2dDrawCommand,
    layer_opacity: f32,
) {
    let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
    let mut style = command.text.style;
    style.color.a = (style.color.a * style.opacity * layer_opacity).clamp(0.0, 1.0);
    if style.color.a <= 0.001 {
        return;
    }

    if let Some(glow) = style.glow {
        let passes = glow.passes.max(1);
        let step = glow.radius.max(0.0) / passes as f32;
        for pass in 1..=passes {
            let radius = pass as f32 * step;
            let alpha = glow.intensity.max(0.0) / pass as f32;
            for (dx, dy) in super::util::text2d_effect_offsets(radius) {
                let glow_transform =
                    super::util::translated_transform2(transform, Vec2::new(dx, dy));
                let color = super::util::color_with_alpha_mul(glow.color, alpha);
                let _ = renderer.append_text2d_ttf_font_texture_batch(
                    texture_batches,
                    device,
                    queue,
                    assets,
                    viewport,
                    camera2d,
                    &command.text.font,
                    &command.text.content,
                    glow_transform,
                    command.text.bounds,
                    style.font_size,
                    color,
                );
            }
        }
    }

    if let Some(outline) = style.outline {
        let width = outline.width.max(0.0);
        if width > 0.0 {
            for (dx, dy) in super::util::text2d_effect_offsets(width) {
                let outline_transform =
                    super::util::translated_transform2(transform, Vec2::new(dx, dy));
                let _ = renderer.append_text2d_ttf_font_texture_batch(
                    texture_batches,
                    device,
                    queue,
                    assets,
                    viewport,
                    camera2d,
                    &command.text.font,
                    &command.text.content,
                    outline_transform,
                    command.text.bounds,
                    style.font_size,
                    outline.color,
                );
            }
        }
    }

    if let Some(shadow) = style.shadow {
        let shadow_transform = super::util::translated_transform2(transform, shadow.offset);
        let _ = renderer.append_text2d_ttf_font_texture_batch(
            texture_batches,
            device,
            queue,
            assets,
            viewport,
            camera2d,
            &command.text.font,
            &command.text.content,
            shadow_transform,
            command.text.bounds,
            style.font_size,
            shadow.color,
        );
    }

    if renderer.append_text2d_ttf_font_texture_batch(
        texture_batches,
        device,
        queue,
        assets,
        viewport,
        camera2d,
        &command.text.font,
        &command.text.content,
        transform,
        command.text.bounds,
        style.font_size,
        style.color,
    ) {
        return;
    }

    let vertices = color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha);
    append_text_2d_vertices(
        vertices,
        viewport,
        camera2d,
        &command.text.content,
        transform,
        command.text.bounds,
        style.color,
    );
}

fn camera2d_for_render_layer(
    base_camera: Transform2,
    render_layer: &str,
    render_layers: &BTreeMap<String, RenderLayer2dCommand>,
) -> Transform2 {
    let Some(layer) = render_layers.get(render_layer) else {
        return base_camera;
    };

    if layer.depth.is_overlay() {
        return base_camera;
    }

    let motion_scale = amigo_2d_spatial::z_depth_to_camera_motion_scale(layer.depth.z_depth);
    Transform2 {
        translation: Vec2::new(
            base_camera.translation.x * motion_scale,
            base_camera.translation.y * motion_scale,
        ),
        rotation_radians: base_camera.rotation_radians,
        scale: base_camera.scale,
    }
}

pub(super) fn execute_world_to_offscreen(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    ctx: WorldRenderContext<'_>,
    selection: WorldRenderSelection<'_>,
    ui_primitives: &[UiDrawPrimitive],
) -> AmigoResult<()> {
    let viewport = Viewport::from_offscreen(target);
    let mut color_batches = Vec::new();
    let mut texture_batches = Vec::new();
    let mut ui_texture_batches = Vec::new();
    let camera2d = resolve_camera2d_transform(ctx.scene, ctx.active_camera_2d_entity);
    let particle_lights = particle_render_lights(ctx.particles);
    let layered_image_commands = ctx.layered_images.commands();
    let global_light_commands = ctx.global_lights.commands();
    let lightmap_sources = ctx.lightmaps.commands();
    let render_layer_lookup = render_layer_lookup(ctx.render_layers);
    let lightmap_samplers = renderer.lightmap_2d_samplers(
        ctx.assets,
        ctx.scene,
        &viewport,
        &layered_image_commands,
        &lightmap_sources,
    );
    let mut world2d_items = ctx.renderables.to_vec();
    world2d_items.retain(|item| {
        selection.layer_filter.allows(item.render_layer())
            && selection.object_filter.allows(item.owner_entity())
    });
    let renderable_adapters = crate::default_renderable_2d_adapter_registry();
    world2d_items.retain(|item| renderable_adapters.supports_kind(&item.payload_kind_id()));
    world2d_items.sort_by_key(|item| world2d_sort_key(item, &render_layer_lookup));
    let mut material_candidates: Vec<WgpuMaterialCandidate2d> = Vec::new();
    let mut material_decisions: Vec<MaterialCandidateDecision2d> = Vec::new();

    for item in world2d_items {
        let layer_camera =
            camera2d_for_render_layer(camera2d, item.render_layer(), &render_layer_lookup);
        match &item.payload {
            Renderable2dPayload::TileMap(command) => {
                let transform =
                    resolve_transform2(ctx.scene, &command.entity_name, Transform2::default());
                if !renderer.append_tilemap_texture_batch(
                    &mut texture_batches,
                    &target.device,
                    &target.queue,
                    ctx.assets,
                    &viewport,
                    layer_camera,
                    transform,
                    &command.tilemap,
                ) {
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                    append_tilemap_fallback_vertices(
                        vertices,
                        &viewport,
                        layer_camera,
                        transform,
                        &command.tilemap,
                    );
                }
            }
            Renderable2dPayload::Sprite(command) => {
                let layer_opacity =
                    render_layer_opacity(&command.render_layer, &render_layer_lookup);
                let transform =
                    resolve_transform2(ctx.scene, &command.entity_name, command.transform);
                if command.render_contributions.enabled_or(
                    amigo_render_api::render_contribution_roles::WORLD_COLOR,
                    true,
                ) {
                    if !renderer.append_sprite_texture_batch(
                        &mut texture_batches,
                        &target.device,
                        &target.queue,
                        ctx.assets,
                        &viewport,
                        layer_camera,
                        transform,
                        &command.sprite,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_sprite_vertices(
                            vertices,
                            &viewport,
                            layer_camera,
                            transform,
                            &command.sprite,
                            sprite_color(command.sprite.texture.as_str()),
                        );
                    }
                }

                collect_material_candidate_2d(
                    &item,
                    layer_camera,
                    layer_opacity,
                    &mut material_candidates,
                    &mut material_decisions,
                );
            }
            Renderable2dPayload::LayeredImage(command) => {
                let transform =
                    resolve_transform2(ctx.scene, &command.entity_name, command.transform);
                renderer.append_layered_image_texture_batches_filtered(
                    &mut texture_batches,
                    &target.device,
                    &target.queue,
                    ctx.assets,
                    &viewport,
                    layer_camera,
                    transform,
                    command,
                    selection
                        .layered_image_part_filter
                        .included_parts(&command.entity_name),
                    selection
                        .layered_image_part_filter
                        .excluded_parts(&command.entity_name),
                    selection
                        .layered_image_part_filter
                        .included_parts(&command.entity_name)
                        .is_none(),
                );
                let layer_opacity = render_layer_opacity(&command.render_layer, &render_layer_lookup);
                collect_material_candidate_2d(
                    &item,
                    layer_camera,
                    layer_opacity,
                    &mut material_candidates,
                    &mut material_decisions,
                );
            }
            Renderable2dPayload::Vector(command) => {
                let layer_opacity =
                    render_layer_opacity(&command.render_layer, &render_layer_lookup);
                let transform = vector_viewport_fit_transform(
                    &viewport,
                    resolve_transform2(ctx.scene, &command.entity_name, command.transform),
                    command.viewport_fit,
                    command.viewport_canvas_size,
                );
                if command.render_contributions.enabled_or(
                    amigo_render_api::render_contribution_roles::WORLD_COLOR,
                    true,
                ) {
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                    append_vector_shape_vertices(
                        vertices,
                        &viewport,
                        layer_camera,
                        transform,
                        &command.shape,
                    );
                }

                collect_material_candidate_2d(
                    &item,
                    layer_camera,
                    layer_opacity,
                    &mut material_candidates,
                    &mut material_decisions,
                );
            }
            Renderable2dPayload::Beacon(command) => {
                let vertices =
                    color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Additive);
                append_beacon_vfx_vertices(vertices, &viewport, layer_camera, command);
            }
            Renderable2dPayload::Particle(command) => {
                if command
                    .light
                    .is_some_and(|light| light.glow && light.mode == ParticleLightMode2d::Particle)
                {
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Additive);
                    append_particle_light_vertices(vertices, &viewport, layer_camera, &command);
                }
                let vertices = color_batch_vertices(&mut color_batches, command.blend_mode);
                append_particle_vertices(
                    vertices,
                    &viewport,
                    layer_camera,
                    &command,
                    &particle_lights,
                    &lightmap_samplers,
                    &global_light_commands,
                    ctx.light_groups,
                    ctx.light_routes,
                );
                let layer_opacity = render_layer_opacity(&command.render_layer, &render_layer_lookup);
                collect_material_candidate_2d(
                    &item,
                    layer_camera,
                    layer_opacity,
                    &mut material_candidates,
                    &mut material_decisions,
                );
            }
            Renderable2dPayload::Text(command) => {
                let layer_opacity =
                    render_layer_opacity(&command.render_layer, &render_layer_lookup);
                if command.render_contributions.enabled_or(
                    amigo_render_api::render_contribution_roles::WORLD_COLOR,
                    true,
                ) {
                    append_text2d_draw_command(
                        renderer,
                        &mut texture_batches,
                        &mut color_batches,
                        &target.device,
                        &target.queue,
                        ctx.assets,
                        &viewport,
                        layer_camera,
                        ctx.scene,
                        &command,
                        layer_opacity,
                    );
                }
                collect_material_candidate_2d(
                    &item,
                    layer_camera,
                    layer_opacity,
                    &mut material_candidates,
                    &mut material_decisions,
                );
            }
        }
    }

    let camera = resolve_camera_transform(ctx.scene);
    let material_lookup = material_lookup_from_commands(ctx.materials);
    let mut projected_triangles = Vec::new();

    if selection.layer_filter.allows_layerless() {
        for command in ctx.meshes {
            let transform =
                resolve_transform3(ctx.scene, &command.entity_name, command.mesh.transform);
            let color = material_lookup
                .get(&command.entity_name)
                .copied()
                .unwrap_or_else(|| mesh_color(command.mesh.mesh_asset.as_str()));
            append_mesh_triangles(
                &mut projected_triangles,
                &viewport,
                camera,
                transform,
                color,
            );
        }
    }

    projected_triangles.sort_by(|left, right| {
        right
            .depth
            .partial_cmp(&left.depth)
            .unwrap_or(Ordering::Equal)
    });

    for triangle in projected_triangles {
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        push_triangle(vertices, triangle.points, triangle.color);
    }

    if let Some(text3d) = ctx
        .text3d
        .filter(|_| selection.layer_filter.allows_layerless())
    {
        for command in text3d {
            let transform =
                resolve_transform3(ctx.scene, &command.entity_name, command.text.transform);
            if renderer.append_text3d_ttf_font_texture_batch(
                &mut ui_texture_batches,
                &target.device,
                &target.queue,
                ctx.assets,
                &viewport,
                camera,
                &command.text.font,
                &command.text.content,
                transform,
                command.text.size,
                ColorRgba::new(0.94, 0.98, 1.0, 1.0),
            ) {
                continue;
            }

            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_text_3d_vertices(
                vertices,
                &viewport,
                camera,
                &command.text.content,
                transform,
                command.text.size,
                ColorRgba::new(0.94, 0.98, 1.0, 1.0),
            );
        }
    }

    let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());
    if selection.layer_filter.allows_layerless() {
        for primitive in ui_primitives {
            if let UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: Some(font),
                anchor,
                word_wrap,
                fit_to_width,
            } = primitive
            {
                if renderer.append_ui_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    ctx.assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }

                if renderer.append_ui_bitmap_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    ctx.assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }
    }

    {
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
    }

    renderer.render_offscreen_batches(
        target,
        selection.pass_load.to_load_op(),
        &texture_batches,
        &color_batches,
        &ui_texture_batches,
    )?;
    super::refractive_material::execute_refractive_material_2d(
        renderer,
        target,
        ctx.assets,
        &viewport,
        ctx.scene,
        &material_candidates,
        &material_decisions,
    )
}

fn render_layer_opacity(
    render_layer: &str,
    render_layers: &BTreeMap<String, RenderLayer2dCommand>,
) -> f32 {
    render_layers
        .get(render_layer)
        .map(|layer| if layer.visible { layer.opacity } else { 0.0 })
        .unwrap_or(1.0)
        .clamp(0.0, 1.0)
}

pub(super) fn execute_layered_image_parts_to_offscreen(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    scene: &SceneService,
    assets: &AssetCatalog,
    layered_images: &amigo_layered_image_2d_plugin::LayeredImageSceneService,
    render_layers: &[RenderLayer2dCommand],
    active_camera_2d_entity: Option<&str>,
    part_targets: &BTreeMap<String, BTreeSet<String>>,
    pass_load: WorldPassLoad,
) -> AmigoResult<()> {
    let viewport = Viewport::from_offscreen(target);
    let camera2d = resolve_camera2d_transform(scene, active_camera_2d_entity);
    let render_layer_lookup = render_layer_lookup(render_layers);
    let mut texture_batches = Vec::new();

    let mut commands = layered_images
        .commands()
        .into_iter()
        .filter(|command| part_targets.contains_key(&command.entity_name))
        .collect::<Vec<_>>();
    commands.sort_by_key(|command| {
        let layer_order = render_layer_lookup
            .get(&command.render_layer)
            .map(|layer| layer.order)
            .unwrap_or(0.0);
        (
            (layer_order * 1000.0).round() as i32,
            (command.z_index * 1000.0).round() as i32,
        )
    });

    for command in commands {
        let Some(parts) = part_targets.get(&command.entity_name) else {
            continue;
        };
        let transform = resolve_transform2(scene, &command.entity_name, command.transform);
        renderer.append_layered_image_texture_batches_filtered(
            &mut texture_batches,
            &target.device,
            &target.queue,
            assets,
            &viewport,
            camera2d,
            transform,
            &command,
            Some(parts),
            None,
            false,
        );
    }

    renderer.render_offscreen_batches(target, pass_load.to_load_op(), &texture_batches, &[], &[])
}

impl WgpuSceneRenderer {
    pub(super) fn render_offscreen_batches(
        &self,
        target: &mut WgpuOffscreenTarget,
        load_op: wgpu::LoadOp<wgpu::Color>,
        texture_batches: &[TextureBatch],
        color_batches: &[ColorBatch],
        ui_texture_batches: &[TextureBatch],
    ) -> AmigoResult<()> {
        let texture_batches = texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let color_batches = color_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let ui_texture_batches = ui_texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-render-encoder"),
            });

        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-color-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let texture_vertex_buffers = texture_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let ui_texture_vertex_buffers = ui_texture_batches
            .iter()
            .map(|batch| {
                target
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-offscreen-ui-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in ui_texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}
