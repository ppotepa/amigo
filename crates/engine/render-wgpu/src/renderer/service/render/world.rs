use super::material_candidates::WgpuMaterialCandidate2d;
use super::*;
use crate::renderer::service::WgpuDynamicVertexBuffer;
use amigo_material_api::MaterialCandidateDecision2d;
use amigo_render_api::{LightSource2dCommon, RenderAssetSource, RenderLightMap2dSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NprMeshRenderRoute {
    GpuRealtime,
    CpuReference,
}

fn npr_mesh_render_route(settings: &amigo_render_api::NprLineSettings3d) -> NprMeshRenderRoute {
    match settings.render_strategy {
        amigo_render_api::NprRenderStrategy3d::GpuRealtime => NprMeshRenderRoute::GpuRealtime,
        amigo_render_api::NprRenderStrategy3d::CpuReference => NprMeshRenderRoute::CpuReference,
    }
}

#[derive(Clone, Copy)]
pub(super) struct WorldRenderContext<'a> {
    pub scene_view: &'a amigo_render_api::RenderSceneView,
    pub camera_debug_view: &'a amigo_render_api::CameraDebugView2d,
    pub assets: &'a dyn RenderAssetSource,
    pub renderables: &'a [Renderable2dItem],
    pub light_sources: &'a [LightSource2dCommon],
    pub lightmaps: &'a [RenderLightMap2dSource],
    pub meshes: &'a [MeshDrawCommand],
    pub materials: &'a [MaterialDrawCommand],
    pub text3d: Option<&'a [Text3dDrawCommand]>,
    pub render_layers: &'a [RenderLayer2dCommand],
    pub light_routes: &'a [LightRoute2dCommand],
}

impl<'a> WorldRenderContext<'a> {
    pub(super) fn from_request(request: &'a WgpuFrameRenderRequest<'a>) -> Self {
        Self {
            scene_view: request.scene_view,
            camera_debug_view: &request.camera_debug_view,
            assets: request.assets,
            renderables: request.world_2d.renderables,
            light_sources: request.world_2d.light_sources,
            lightmaps: request.world_2d.lightmaps,
            meshes: request.world_3d.meshes,
            materials: request.world_3d.materials,
            text3d: request.world_3d.text3d,
            render_layers: request.world_2d.render_layers,
            light_routes: request.world_2d.light_routes,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn renderable_adapter_context<'a>(
    renderer: &'a mut WgpuSceneRenderer,
    texture_batches: &'a mut Vec<TextureBatch>,
    color_batches: &'a mut Vec<ColorBatch>,
    target: &'a WgpuOffscreenTarget,
    assets: &'a dyn RenderAssetSource,
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
        .included_parts(item.object_id());
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
            .excluded_parts(item.object_id()),
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
    let camera2d = resolve_camera2d_transform(ctx.scene_view);
    let particle_lights = particle_render_lights_from_renderables(ctx.renderables);
    let render_layer_lookup = render_layer_lookup(ctx.render_layers);
    let lightmap_samplers =
        renderer.lightmap_2d_samplers(ctx.assets, &viewport, ctx.renderables, ctx.lightmaps);
    let mut world2d_items = ctx.renderables.iter().collect::<Vec<_>>();
    world2d_items.retain(|item| {
        selection.layer_filter.allows(item.render_layer())
            && selection.object_filter.allows(item.object_id())
    });
    let renderable_adapters = crate::default_renderable_2d_adapter_registry();
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
            item,
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

    let camera = resolve_camera_transform(ctx.scene_view);
    let camera_settings = ctx.scene_view.camera_3d_settings();
    let light_settings = ctx.scene_view.light_3d_settings();
    let material_lookup = material_lookup_from_commands(ctx.materials);
    let mut projected_triangles = Vec::new();
    let mut npr_line_vertices = Vec::new();
    let mut npr_stroke_segment_vertices = Vec::new();
    let mut npr_debug_vertices = Vec::new();
    let npr_debug_overlay = NprDebugOverlay3d::from_camera_debug_view(ctx.camera_debug_view);
    renderer.npr_stroke_stats_3d = crate::renderer::NprStrokeFrameStats3d::default();
    renderer.npr_gpu_realtime.begin_frame();

    if selection.layer_filter.allows_layerless() && !ctx.meshes.is_empty() {
        for command in ctx.meshes {
            let transform =
                resolve_transform3(ctx.scene_view, &command.entity_name, command.mesh.transform);
            let geometry = renderer.mesh_geometry_3d(ctx.assets, &command.mesh.mesh_asset);
            let material = material_lookup.get(&command.entity_name).copied();
            let color = material
                .map(|material| material.albedo)
                .unwrap_or_else(|| mesh_color(command.mesh.mesh_asset.as_str()));
            let render_order = material.map(|material| material.render_order).unwrap_or(0);
            let shading = material
                .map(|material| material.shading)
                .unwrap_or_default();
            let render_fill = command
                .mesh
                .npr
                .as_ref()
                .is_none_or(|npr| npr.fill_mode == amigo_render_api::NprFillMode3d::Shaded);
            if render_fill {
                append_mesh_triangles(
                    &mut projected_triangles,
                    &viewport,
                    camera,
                    camera_settings,
                    light_settings,
                    &geometry,
                    transform,
                    color,
                    render_order,
                    shading,
                );
            }
            if let Some(npr) = command.mesh.npr.as_ref() {
                if npr.pipeline.fill_strategy
                    == amigo_render_api::NprInkFillStrategy3d::MaterialBlackMass
                    && !npr.black_mass_material_ids.is_empty()
                {
                    append_mesh_black_mass_triangles(
                        &mut projected_triangles,
                        &viewport,
                        camera,
                        camera_settings,
                        &geometry,
                        transform,
                        &npr.black_mass_material_ids,
                        render_order + 1,
                    );
                }
            }
            if let Some(npr) = command.mesh.npr.as_ref() {
                match npr_mesh_render_route(npr) {
                    NprMeshRenderRoute::CpuReference => {
                        let mut stats = renderer.npr_cpu_reference.append_mesh(
                            renderer.frame_counter,
                            &command.entity_name,
                            &mut npr_line_vertices,
                            &mut npr_stroke_segment_vertices,
                            &viewport,
                            camera,
                            camera_settings,
                            &geometry,
                            transform,
                            npr,
                        );
                        stats.record_strategy(amigo_render_api::NprRenderStrategy3d::CpuReference);
                        renderer.npr_stroke_stats_3d.add(stats);

                        if let Some(overlay) = npr_debug_overlay {
                            renderer.npr_cpu_reference.append_debug_overlay(
                                &command.entity_name,
                                &mut npr_debug_vertices,
                                &viewport,
                                camera,
                                camera_settings,
                                &geometry,
                                transform,
                                npr,
                                overlay,
                            );
                        }
                    }
                    NprMeshRenderRoute::GpuRealtime => {
                        renderer.npr_stroke_stats_3d.meshes += 1;
                        renderer
                            .npr_stroke_stats_3d
                            .record_strategy(amigo_render_api::NprRenderStrategy3d::GpuRealtime);
                        renderer
                            .npr_gpu_realtime
                            .enqueue_mesh(
                                &command.entity_name,
                                &command.mesh.mesh_asset,
                                &geometry,
                                transform,
                                npr,
                            )
                            .map_err(|error| {
                                amigo_core::AmigoError::Message(format!(
                                    "NPR gpu_realtime enqueue failed for `{}`: {}",
                                    command.entity_name, error
                                ))
                            })?;
                    }
                }
            }
        }
    }

    projected_triangles.sort_by(|left, right| {
        left.render_order.cmp(&right.render_order).then_with(|| {
            right
                .depth
                .partial_cmp(&left.depth)
                .unwrap_or(Ordering::Equal)
        })
    });

    for triangle in projected_triangles {
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        push_triangle(vertices, triangle.points, triangle.color);
    }
    if !npr_line_vertices.is_empty() {
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        vertices.extend(npr_line_vertices);
    }
    let npr_stroke_segment_batches = if npr_stroke_segment_vertices.is_empty() {
        Vec::new()
    } else {
        vec![NprStrokeSegmentBatch {
            blend_mode: ParticleBlendMode2d::Alpha,
            vertices: npr_stroke_segment_vertices,
        }]
    };
    if !npr_debug_vertices.is_empty() {
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        vertices.extend(npr_debug_vertices);
    }

    if let Some(text3d) = ctx
        .text3d
        .filter(|text3d| selection.layer_filter.allows_layerless() && !text3d.is_empty())
    {
        for command in text3d {
            let transform =
                resolve_transform3(ctx.scene_view, &command.entity_name, command.text.transform);
            const USE_TEXTURED_3D_TEXT: bool = false;
            if USE_TEXTURED_3D_TEXT
                && renderer.append_text3d_ttf_font_texture_batch(
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
                )
            {
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

    let mut ui_color_primitives = Vec::new();
    if selection.layer_filter.allows_layerless() && !ui_primitives.is_empty() {
        ui_color_primitives.reserve(ui_primitives.len());
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

    let mut npr_encoder = target
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("amigo-npr-gpu-realtime-encoder"),
        });
    let gpu_npr_stats = match renderer.npr_gpu_realtime.execute(
        &target.device,
        &target.queue,
        &mut npr_encoder,
        &viewport,
        camera,
        camera_settings,
        npr_debug_overlay,
    ) {
        Ok(stats) => stats,
        Err(error) => {
            renderer.record_frame_diagnostic("npr.gpu_realtime.execution_failed", error.clone());
            return Err(amigo_core::AmigoError::Message(format!(
                "NPR gpu_realtime execution failed: {}",
                error
            )));
        }
    };
    renderer.npr_stroke_stats_3d.gpu_realtime_enqueued_edges += gpu_npr_stats.classified_edges;
    renderer.npr_stroke_stats_3d.gpu_realtime_enqueued_triangles +=
        gpu_npr_stats.enqueued_triangles;
    renderer.npr_stroke_stats_3d.gpu_realtime_topology_uploads += gpu_npr_stats.topology_uploads;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_buffer_capacity_bytes += gpu_npr_stats.buffer_capacity_bytes;
    renderer.npr_stroke_stats_3d.gpu_realtime_frame_jobs += gpu_npr_stats.frame_jobs;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_projected_vertices_capacity += gpu_npr_stats.projected_vertices_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_visible_segments_capacity += gpu_npr_stats.visible_segments_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_endpoint_heads_capacity += gpu_npr_stats.endpoint_heads_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_endpoint_entries_capacity += gpu_npr_stats.endpoint_entries_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_path_links_capacity += gpu_npr_stats.path_links_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_path_states_capacity += gpu_npr_stats.path_states_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_path_segments_capacity += gpu_npr_stats.path_segments_capacity;
    renderer
        .npr_stroke_stats_3d
        .gpu_realtime_stroke_segments_capacity += gpu_npr_stats.stroke_segments_capacity;
    if renderer
        .npr_stroke_stats_3d
        .gpu_realtime_debug_mode
        .is_empty()
    {
        renderer.npr_stroke_stats_3d.gpu_realtime_debug_mode = gpu_npr_stats.debug_mode.to_owned();
    } else if renderer.npr_stroke_stats_3d.gpu_realtime_debug_mode != gpu_npr_stats.debug_mode {
        renderer.npr_stroke_stats_3d.gpu_realtime_debug_mode = "mixed".to_owned();
    }
    let npr_command_buffer = npr_encoder.finish();

    {
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);
    }

    renderer.render_offscreen_batches_after_command_buffers(
        target,
        vec![npr_command_buffer],
        selection
            .pass_load
            .to_load_op_with_clear(ctx.scene_view.background_color()),
        &texture_batches,
        &color_batches,
        &npr_stroke_segment_batches,
        &ui_texture_batches,
    )?;
    if material_candidates.is_empty() {
        return Ok(());
    }

    super::refractive_material::execute_refractive_material_2d(
        renderer,
        target,
        ctx.assets,
        &viewport,
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

impl WgpuSceneRenderer {
    pub(super) fn render_offscreen_batches(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        load_op: wgpu::LoadOp<wgpu::Color>,
        texture_batches: &[TextureBatch],
        color_batches: &[ColorBatch],
        npr_stroke_segment_batches: &[NprStrokeSegmentBatch],
        ui_texture_batches: &[TextureBatch],
    ) -> AmigoResult<()> {
        self.render_offscreen_batches_after_command_buffers(
            target,
            Vec::new(),
            load_op,
            texture_batches,
            color_batches,
            npr_stroke_segment_batches,
            ui_texture_batches,
        )
    }

    pub(super) fn render_offscreen_batches_after_command_buffers(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        mut prelude_command_buffers: Vec<wgpu::CommandBuffer>,
        load_op: wgpu::LoadOp<wgpu::Color>,
        texture_batches: &[TextureBatch],
        color_batches: &[ColorBatch],
        npr_stroke_segment_batches: &[NprStrokeSegmentBatch],
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
        let npr_stroke_segment_batches = npr_stroke_segment_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let ui_texture_batches = ui_texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();

        self.upload_offscreen_color_vertex_buffers(target, &color_batches);
        self.upload_offscreen_npr_stroke_segment_vertex_buffers(
            target,
            &npr_stroke_segment_batches,
        );
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

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-render-encoder"),
            });

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
                pass.set_vertex_buffer(
                    0,
                    self.offscreen_color_vertex_buffers[index].buffer.slice(..),
                );
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in npr_stroke_segment_batches.iter().enumerate() {
                pass.set_pipeline(self.npr_stroke_segment_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(
                    0,
                    self.offscreen_npr_stroke_segment_vertex_buffers[index]
                        .buffer
                        .slice(..),
                );
                pass.draw(0..6, 0..batch.vertices.len() as u32);
            }

            if self.npr_gpu_realtime.has_draw_output() {
                self.npr_gpu_realtime.draw_to_offscreen_pass(
                    &mut pass,
                    self.npr_stroke_segment_pipeline_for(ParticleBlendMode2d::Alpha),
                );
            }

            for (index, batch) in ui_texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        prelude_command_buffers.push(encoder.finish());
        target.queue.submit(prelude_command_buffers);
        Ok(())
    }

    fn npr_stroke_segment_pipeline_for(
        &self,
        _blend_mode: ParticleBlendMode2d,
    ) -> &wgpu::RenderPipeline {
        self.pipeline(CORE_NPR_STROKE_SEGMENT_ALPHA_PIPELINE)
    }

    fn upload_offscreen_color_vertex_buffers(
        &mut self,
        target: &WgpuOffscreenTarget,
        color_batches: &[&ColorBatch],
    ) {
        self.offscreen_upload_stats.color_buffer_writes = 0;
        self.offscreen_upload_stats.color_buffer_reallocs = 0;
        self.offscreen_upload_stats.color_upload_bytes = 0;
        for (index, batch) in color_batches.iter().enumerate() {
            let contents = vertices_as_bytes(&batch.vertices);
            let required_bytes = contents.len() as u64;
            if self.offscreen_color_vertex_buffers.len() <= index {
                self.offscreen_color_vertex_buffers
                    .push(create_dynamic_color_vertex_buffer(
                        &target.device,
                        required_bytes,
                    ));
                self.offscreen_upload_stats.color_buffer_reallocs += 1;
            } else if self.offscreen_color_vertex_buffers[index].capacity_bytes < required_bytes {
                self.offscreen_color_vertex_buffers[index] =
                    create_dynamic_color_vertex_buffer(&target.device, required_bytes);
                self.offscreen_upload_stats.color_buffer_reallocs += 1;
            }
            self.offscreen_upload_stats.color_buffer_writes += 1;
            self.offscreen_upload_stats.color_upload_bytes += required_bytes;
            target.queue.write_buffer(
                &self.offscreen_color_vertex_buffers[index].buffer,
                0,
                contents,
            );
        }
        self.offscreen_color_vertex_buffers
            .truncate(color_batches.len());
        self.offscreen_upload_stats.color_buffer_capacity_bytes = self
            .offscreen_color_vertex_buffers
            .iter()
            .map(|buffer| buffer.capacity_bytes)
            .sum();
    }

    fn upload_offscreen_npr_stroke_segment_vertex_buffers(
        &mut self,
        target: &WgpuOffscreenTarget,
        batches: &[&NprStrokeSegmentBatch],
    ) {
        self.offscreen_upload_stats.npr_stroke_segment_buffer_writes = 0;
        self.offscreen_upload_stats
            .npr_stroke_segment_buffer_reallocs = 0;
        self.offscreen_upload_stats.npr_stroke_segment_upload_bytes = 0;
        for (index, batch) in batches.iter().enumerate() {
            let contents = npr_stroke_segment_vertices_as_bytes(&batch.vertices);
            let required_bytes = contents.len() as u64;
            if self.offscreen_npr_stroke_segment_vertex_buffers.len() <= index {
                self.offscreen_npr_stroke_segment_vertex_buffers.push(
                    create_dynamic_vertex_buffer(
                        &target.device,
                        required_bytes,
                        "amigo-offscreen-npr-stroke-segment-vertices-dynamic",
                    ),
                );
                self.offscreen_upload_stats
                    .npr_stroke_segment_buffer_reallocs += 1;
            } else if self.offscreen_npr_stroke_segment_vertex_buffers[index].capacity_bytes
                < required_bytes
            {
                self.offscreen_npr_stroke_segment_vertex_buffers[index] =
                    create_dynamic_vertex_buffer(
                        &target.device,
                        required_bytes,
                        "amigo-offscreen-npr-stroke-segment-vertices-dynamic",
                    );
                self.offscreen_upload_stats
                    .npr_stroke_segment_buffer_reallocs += 1;
            }
            self.offscreen_upload_stats.npr_stroke_segment_buffer_writes += 1;
            self.offscreen_upload_stats.npr_stroke_segment_upload_bytes += required_bytes;
            target.queue.write_buffer(
                &self.offscreen_npr_stroke_segment_vertex_buffers[index].buffer,
                0,
                contents,
            );
        }
        self.offscreen_npr_stroke_segment_vertex_buffers
            .truncate(batches.len());
        self.offscreen_upload_stats
            .npr_stroke_segment_buffer_capacity_bytes = self
            .offscreen_npr_stroke_segment_vertex_buffers
            .iter()
            .map(|buffer| buffer.capacity_bytes)
            .sum();
    }
}

fn create_dynamic_color_vertex_buffer(
    device: &wgpu::Device,
    required_bytes: u64,
) -> WgpuDynamicVertexBuffer {
    create_dynamic_vertex_buffer(
        device,
        required_bytes,
        "amigo-offscreen-color-vertices-dynamic",
    )
}

fn create_dynamic_vertex_buffer(
    device: &wgpu::Device,
    required_bytes: u64,
    label: &'static str,
) -> WgpuDynamicVertexBuffer {
    let capacity_bytes = required_bytes.max(256).next_power_of_two();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: capacity_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    WgpuDynamicVertexBuffer {
        buffer,
        capacity_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_npr_gpu_realtime_meshes() {
        let settings = amigo_render_api::NprLineSettings3d {
            render_strategy: amigo_render_api::NprRenderStrategy3d::GpuRealtime,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_mesh_render_route(&settings),
            NprMeshRenderRoute::GpuRealtime
        );
    }

    #[test]
    fn routes_npr_cpu_reference_meshes() {
        let settings = amigo_render_api::NprLineSettings3d {
            render_strategy: amigo_render_api::NprRenderStrategy3d::CpuReference,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_mesh_render_route(&settings),
            NprMeshRenderRoute::CpuReference
        );
    }

    #[test]
    #[ignore = "benchmarks GPU upload path; run explicitly"]
    fn benchmark_offscreen_color_upload_reuses_dynamic_buffers() {
        let backend = crate::WgpuRenderBackend::default();
        let mut target = backend
            .initialize_offscreen(1280, 720)
            .expect("headless offscreen target should initialize");
        let mut renderer = WgpuSceneRenderer::new_for_offscreen(&target);
        let vertices = (0..40_000)
            .flat_map(|index| {
                let x = ((index % 200) as f32 / 100.0) - 1.0;
                let y = (((index / 200) % 200) as f32 / 100.0) - 1.0;
                [
                    ColorVertex::new(Vec2::new(x, y), ColorRgba::WHITE),
                    ColorVertex::new(Vec2::new((x + 0.002).min(1.0), y), ColorRgba::WHITE),
                    ColorVertex::new(Vec2::new(x, (y + 0.002).min(1.0)), ColorRgba::WHITE),
                ]
            })
            .collect::<Vec<_>>();
        let batch = ColorBatch {
            blend_mode: ParticleBlendMode2d::Alpha,
            vertices,
        };

        renderer
            .render_offscreen_batches(
                &mut target,
                wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                &[],
                &[batch.clone()],
                &[],
                &[],
            )
            .expect("first upload render should pass");
        let first = renderer.offscreen_upload_stats();
        let start = std::time::Instant::now();
        renderer
            .render_offscreen_batches(
                &mut target,
                wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                &[],
                &[batch],
                &[],
                &[],
            )
            .expect("second upload render should pass");
        let second_elapsed_us = start.elapsed().as_secs_f64() * 1_000_000.0;
        let second = renderer.offscreen_upload_stats();

        println!(
            "offscreen color upload benchmark: first_reallocs={} second_reallocs={} writes={} upload_bytes={} capacity_bytes={} second_us={second_elapsed_us:.2}",
            first.color_buffer_reallocs,
            second.color_buffer_reallocs,
            second.color_buffer_writes,
            second.color_upload_bytes,
            second.color_buffer_capacity_bytes,
        );
        assert!(first.color_buffer_reallocs > 0);
        assert_eq!(second.color_buffer_reallocs, 0);
        assert_eq!(second.color_buffer_writes, 1);
        assert!(second.color_upload_bytes > 0);
    }
}
