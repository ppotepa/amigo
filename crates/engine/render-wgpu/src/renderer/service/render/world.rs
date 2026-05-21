use super::material_candidates::WgpuMaterialCandidate2d;
use super::world_filters::WorldPassLoad;
use super::*;
use amigo_render_api::{LightSource2dCommon, RenderLightMap2dSource};
use amigo_material_api::MaterialCandidateDecision2d;

#[derive(Clone, Copy)]
pub(super) struct WorldRenderContext<'a> {
    pub scene: &'a SceneService,
    pub assets: &'a AssetCatalog,
    pub renderables: &'a [Renderable2dItem],
    pub light_sources: &'a [LightSource2dCommon],
    pub lightmaps: &'a [RenderLightMap2dSource],
    pub meshes: &'a [MeshDrawCommand],
    pub materials: &'a [MaterialDrawCommand],
    pub text3d: Option<&'a [Text3dDrawCommand]>,
    pub render_layers: &'a [RenderLayer2dCommand],
    pub light_routes: &'a [LightRoute2dCommand],
    pub active_camera_2d_entity: Option<&'a str>,
}

impl<'a> WorldRenderContext<'a> {
    pub(super) fn from_request(request: &'a WgpuFrameRenderRequest<'a>) -> Self {
        Self {
            scene: request.scene,
            assets: request.assets,
            renderables: request.world_2d.renderables,
            light_sources: request.world_2d.light_sources,
            lightmaps: request.world_2d.lightmaps,
            meshes: request.world_3d.meshes,
            materials: request.world_3d.materials,
            text3d: request.world_3d.text3d,
            render_layers: request.world_2d.render_layers,
            light_routes: request.world_2d.light_routes,
            active_camera_2d_entity: request.active_camera_2d_entity,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn renderable_adapter_context<'a>(
    renderer: &'a mut WgpuSceneRenderer,
    texture_batches: &'a mut Vec<TextureBatch>,
    color_batches: &'a mut Vec<ColorBatch>,
    target: &'a WgpuOffscreenTarget,
    assets: &'a AssetCatalog,
    viewport: &'a Viewport,
    layer_camera: Transform2,
    layer_opacity: f32,
    transform: Transform2,
    material_candidates: &'a mut Vec<WgpuMaterialCandidate2d>,
    material_decisions: &'a mut Vec<MaterialCandidateDecision2d>,
    included_layered_image_parts: Option<&'a BTreeSet<String>>,
    excluded_layered_image_parts: Option<&'a BTreeSet<String>>,
    include_base_layered_image: bool,
    particle_lights: &'a [ParticleRenderLight],
    lightmap_samplers: &'a [LightMap2dSampler],
    light_sources: &'a [LightSource2dCommon],
    light_routes: &'a [LightRoute2dCommand],
) -> crate::WgpuRenderable2dAdapterContext<'a> {
    crate::WgpuRenderable2dAdapterContext {
        renderer,
        texture_batches,
        color_batches,
        device: &target.device,
        queue: &target.queue,
        assets,
        viewport,
        layer_camera,
        layer_opacity,
        transform,
        material_candidates,
        material_decisions,
        included_layered_image_parts,
        excluded_layered_image_parts,
        include_base_layered_image,
        particle_lights,
        lightmap_samplers,
        light_sources,
        light_routes,
    }
}

#[allow(clippy::too_many_arguments)]
fn render_renderable_2d_item(
    renderer: &mut WgpuSceneRenderer,
    target: &WgpuOffscreenTarget,
    ctx: WorldRenderContext<'_>,
    selection: &WorldRenderSelection<'_>,
    viewport: &Viewport,
    render_layer_lookup: &BTreeMap<String, RenderLayer2dCommand>,
    renderable_adapters: &crate::WgpuRenderable2dAdapterRegistry,
    item: &Renderable2dItem,
    layer_camera: Transform2,
    texture_batches: &mut Vec<TextureBatch>,
    color_batches: &mut Vec<ColorBatch>,
    particle_lights: &[ParticleRenderLight],
    lightmap_samplers: &[LightMap2dSampler],
    light_sources: &[LightSource2dCommon],
    material_candidates: &mut Vec<WgpuMaterialCandidate2d>,
    material_decisions: &mut Vec<MaterialCandidateDecision2d>,
) {
    let layer_opacity = render_layer_opacity(item.render_layer(), render_layer_lookup);
    let included_parts = selection
        .layered_image_part_filter
        .included_parts(item.owner_entity());
    let mut adapter_ctx = renderable_adapter_context(
        renderer,
        texture_batches,
        color_batches,
        target,
        ctx.assets,
        viewport,
        layer_camera,
        layer_opacity,
        item.primitive.transform(),
        material_candidates,
        material_decisions,
        included_parts,
        selection
            .layered_image_part_filter
            .excluded_parts(item.owner_entity()),
        included_parts.is_none(),
        particle_lights,
        lightmap_samplers,
        light_sources,
        ctx.light_routes,
    );
    let _ = renderable_adapters.append_batches(&mut adapter_ctx, item);
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
    let particle_lights = particle_render_lights_from_renderables(ctx.renderables);
    let render_layer_lookup = render_layer_lookup(ctx.render_layers);
    let lightmap_samplers = renderer.lightmap_2d_samplers(
        ctx.assets,
        &viewport,
        ctx.renderables,
        ctx.lightmaps,
    );
    let mut world2d_items = ctx.renderables.to_vec();
    world2d_items.retain(|item| {
        selection.layer_filter.allows(item.render_layer())
            && selection.object_filter.allows(item.owner_entity())
    });
    let renderable_adapters = crate::default_renderable_2d_adapter_registry();
    world2d_items.retain(|item| renderable_adapters.supports_kind(item.primitive_kind()));
    world2d_items.sort_by_key(|item| world2d_sort_key(item, &render_layer_lookup));
    let mut material_candidates: Vec<WgpuMaterialCandidate2d> = Vec::new();
    let mut material_decisions: Vec<MaterialCandidateDecision2d> = Vec::new();

    for item in world2d_items {
        let layer_camera =
            camera2d_for_render_layer(camera2d, item.render_layer(), &render_layer_lookup);
        render_renderable_2d_item(
            renderer,
            target,
            ctx,
            &selection,
            &viewport,
            &render_layer_lookup,
            renderable_adapters,
            &item,
            layer_camera,
            &mut texture_batches,
            &mut color_batches,
            &particle_lights,
            &lightmap_samplers,
            ctx.light_sources,
            &mut material_candidates,
            &mut material_decisions,
        );
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
    renderables: &[Renderable2dItem],
    assets: &AssetCatalog,
    render_layers: &[RenderLayer2dCommand],
    part_targets: &BTreeMap<String, BTreeSet<String>>,
    pass_load: WorldPassLoad,
) -> AmigoResult<()> {
    let viewport = Viewport::from_offscreen(target);
    let render_layer_lookup = render_layer_lookup(render_layers);
    let mut texture_batches = Vec::new();

    let mut items = renderables
        .iter()
        .filter_map(|item| {
            item.primitive
                .layered_image()
                .filter(|_| part_targets.contains_key(item.owner_entity()))
                .map(|layered| (item, layered))
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|(item, _)| {
        let layer_order = render_layer_lookup
            .get(item.render_layer())
            .map(|layer| layer.order)
            .unwrap_or(0.0);
        ((layer_order * 1000.0).round() as i32, (item.z_index() * 1000.0).round() as i32)
    });

    for (item, layered) in items {
        let Some(parts) = part_targets.get(item.owner_entity()) else {
            continue;
        };
        renderer.append_layered_image_primitive_texture_batches_filtered(
            &mut texture_batches,
            &target.device,
            &target.queue,
            assets,
            &viewport,
            Transform2::default(),
            layered,
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
