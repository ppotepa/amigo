use super::material_candidates::WgpuMaterialCandidate2d;
use super::*;
use crate::renderer::npr::{NprGpuVertex, NprIndexedBuffer, NprPipelines, NprVertexBuffer};
use amigo_material_api::MaterialCandidateDecision2d;
use amigo_render_api::{
    LightSource2dCommon, NprBackgroundCommand, NprDrawCommand, RenderAssetSource,
    RenderLightMap2dSource,
};

#[derive(Clone, Copy)]
pub(super) struct WorldRenderContext<'a> {
    pub scene_view: &'a amigo_render_api::RenderSceneView,
    pub assets: &'a dyn RenderAssetSource,
    pub renderables: &'a [Renderable2dItem],
    pub light_sources: &'a [LightSource2dCommon],
    pub lightmaps: &'a [RenderLightMap2dSource],
    pub meshes: &'a [MeshDrawCommand],
    pub materials: &'a [MaterialDrawCommand],
    pub text3d: Option<&'a [Text3dDrawCommand]>,
    pub npr: &'a [NprDrawCommand],
    pub npr_background: Option<NprBackgroundCommand>,
    pub render_layers: &'a [RenderLayer2dCommand],
    pub light_routes: &'a [LightRoute2dCommand],
}

impl<'a> WorldRenderContext<'a> {
    pub(super) fn from_request(request: &'a WgpuFrameRenderRequest<'a>) -> Self {
        Self {
            scene_view: request.scene_view,
            assets: request.assets,
            renderables: request.world_2d.renderables,
            light_sources: request.world_2d.light_sources,
            lightmaps: request.world_2d.lightmaps,
            meshes: request.world_3d.meshes,
            materials: request.world_3d.materials,
            text3d: request.world_3d.text3d,
            npr: request.world_3d.npr,
            npr_background: request.world_3d.npr_background,
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

    if selection.layer_filter.allows_layerless() && !ctx.meshes.is_empty() {
        for command in ctx.meshes {
            let transform =
                resolve_transform3(ctx.scene_view, &command.entity_name, command.mesh.transform);
            let material = material_lookup.get(&command.entity_name).copied();
            let color = material
                .map(|material| material.albedo)
                .unwrap_or_else(|| mesh_color(command.mesh.mesh_asset.as_str()));
            let render_order = material.map(|material| material.render_order).unwrap_or(0);
            append_mesh_triangles(
                &mut projected_triangles,
                &viewport,
                camera,
                camera_settings,
                light_settings,
                transform,
                color,
                render_order,
            );
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

    // A blank NPR scene still owns its paper background. Do not make the
    // background contingent on geometry contributions: the Asset Browser may
    // intentionally leave the scene empty until the user adds a model.
    if !ctx.npr.is_empty() || ctx.npr_background.is_some() {
        renderer.npr.render(target, ctx.npr, ctx.npr_background)?;
    }
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

enum NprColorBatch {
    Paint {
        mask: wgpu::BindGroup,
        buffer: NprVertexBuffer,
        blend: amigo_render_npr::NprBlendMode,
    },
    Fill {
        mask: wgpu::BindGroup,
        buffer: NprVertexBuffer,
        blend: amigo_render_npr::NprBlendMode,
    },
    Stroke {
        mask: wgpu::BindGroup,
        buffer: NprIndexedBuffer,
        blend: amigo_render_npr::NprBlendMode,
    },
}

fn ordered_npr_layer_ids<'a>(
    layers: impl IntoIterator<Item = &'a amigo_render_npr::NprStyleLayers>,
) -> Vec<&'a str> {
    let mut ids = Vec::new();
    for layers in layers {
        for layer in layers.layers.iter().filter(|layer| layer.enabled) {
            if !ids.iter().any(|id| *id == layer.id) {
                ids.push(layer.id.as_str());
            }
        }
    }
    ids
}

impl crate::renderer::npr::WgpuNprRenderer {
    pub fn render(
        &self,
        target: &mut WgpuOffscreenTarget,
        commands: &[NprDrawCommand],
        background: Option<NprBackgroundCommand>,
    ) -> AmigoResult<()> {
        let renderer = self;
        let mut buffers = renderer.npr_buffers.lock().unwrap();
        buffers.begin();
        if commands.iter().any(|command| {
            command.material_base_color.is_none()
                && command.layers.layers.iter().any(|layer| {
                    layer.enabled
                        && layer.color_source
                            == amigo_render_npr::NprLayerColorSource::ModelBaseColor
                })
        }) {
            return Err(amigo_core::AmigoError::Message(
            "NPR ModelBaseColor layers require declared material-base-colour data in the render packet"
                .into(),
        ));
        }
        let layer_color = |source: amigo_render_npr::NprLayerColorSource,
                           fallback: [f32; 4],
                           material_base_color: Option<[f32; 4]>| {
            match source {
                amigo_render_npr::NprLayerColorSource::StylePalette => fallback,
                amigo_render_npr::NprLayerColorSource::Constant(color) => color.to_array(),
                // The preflight above proves this contribution is present for
                // every active model-colour layer.
                amigo_render_npr::NprLayerColorSource::ModelBaseColor => {
                    material_base_color.expect("validated NPR material base colour")
                }
            }
        };
        let width = target.width as f32;
        let height = target.height as f32;
        let to_clip = |position: amigo_render_npr::Point2| {
            [
                position.x / width * 2.0 - 1.0,
                1.0 - position.y / height * 2.0,
            ]
        };
        let mut occluder_vertices = Vec::new();
        let mut color_batches = Vec::new();
        // Stroke batches are indexed and capped before GPU allocation. The cap is
        // global to this NPR view, rather than an accidental per-object allocation.
        const MAX_STROKE_BATCH_BYTES: usize = 64 * 1024 * 1024;
        const MAX_STROKE_UPLOAD_BYTES: usize = 128 * 1024 * 1024;
        let mut stroke_upload_bytes = 0usize;
        let mut paper_vertices = Vec::new();
        let paper_layer = commands.first().and_then(|command| {
            command
                .layers
                .layers
                .iter()
                .find(|layer| layer.source == amigo_render_npr::NprGeometrySource::Paper)
        });
        if let Some(reference) = paper_layer {
            if commands.iter().skip(1).any(|command| {
                command
                    .layers
                    .layers
                    .iter()
                    .find(|layer| layer.source == amigo_render_npr::NprGeometrySource::Paper)
                    .map(|layer| {
                        layer.enabled == reference.enabled && layer.opacity == reference.opacity
                    })
                    != Some(true)
            }) {
                return Err(amigo_core::AmigoError::Message(
                "NPR Paper is scene-owned and must have one consistent enabled state and opacity"
                    .into(),
            ));
            }
        }
        let paper_enabled = paper_layer.map(|layer| layer.enabled).unwrap_or(true);
        let paper_opacity = paper_layer.map(|layer| layer.opacity).unwrap_or(1.0);
        if let Some(background) = background.filter(|background| {
            paper_enabled && (background.grain > f32::EPSILON || background.tooth > f32::EPSILON)
        }) {
            let mut color = background.color;
            color[3] *= paper_opacity;
            let phase = [
                (background.seed.wrapping_mul(0x9e37_79b9) as u32 as f32 / u32::MAX as f32) * 37.0,
                (background.seed.rotate_left(17) as u32 as f32 / u32::MAX as f32) * 29.0,
            ];
            for position in [
                [-1.0, -1.0],
                [1.0, -1.0],
                [1.0, 1.0],
                [-1.0, -1.0],
                [1.0, 1.0],
                [-1.0, 1.0],
            ] {
                paper_vertices.push(NprGpuVertex {
                    surface: Default::default(),
                    position,
                    color,
                    depth: 1.0,
                    coverage: (background.grain * 0.65 + background.tooth * 0.35).clamp(0.0, 1.0),
                    phase,
                    material: [0.0; 4],
                });
            }
        }
        // Visibility has already been resolved into occluders.  Build that depth
        // information once, before layer ordering can affect colour composition.
        for command in commands {
            for triangle in &command.packet.occluders {
                for (index, position) in triangle.positions.into_iter().enumerate() {
                    occluder_vertices.push(NprGpuVertex {
                        surface: triangle.surface[index].into(),
                        position: to_clip(position),
                        color: triangle.color.to_array(),
                        depth: triangle.depths[index],
                        coverage: 1.0,
                        phase: [0.0; 2],
                        material: [0.0; 4],
                    });
                }
            }
        }
        // The authored stack is global: emit every object contribution for the
        // first declared layer before proceeding to the next layer.  This matters
        // for non-commutative blends such as Multiply and Screen.
        for layer_id in ordered_npr_layer_ids(commands.iter().map(|command| &command.layers)) {
            for command in commands {
                let Some(layer) = command
                    .layers
                    .layers
                    .iter()
                    .find(|layer| layer.enabled && layer.id == layer_id)
                else {
                    continue;
                };
                if !layer.target.includes(&command.object_id, layer.source) {
                    continue;
                }
                layer.validate_mask_inputs(&command.packet).map_err(amigo_core::AmigoError::Message)?;
                let mask = renderer.npr_pipelines.mask_binding(&mut buffers,&target.device,&target.queue,&layer.mask).map_err(amigo_core::AmigoError::Message)?;
                match layer.source {
                    amigo_render_npr::NprGeometrySource::Wash => {
                        let paint = layer.paint.unwrap_or_default();
                        let granulation = paint.granulation;
                        let mut vertices =
                            Vec::with_capacity(command.packet.underpainting.len() * 3);
                        for triangle in command.packet.underpainting.iter().filter(|triangle| {
                            triangle.layer_id.as_deref() == Some(layer.id.as_str())
                        }) {
                            let mut color = layer_color(
                                layer.color_source,
                                triangle.color.to_array(),
                                command.material_base_color,
                            );
                            color[3] *= layer.opacity;
                            for (index, position) in triangle.positions.into_iter().enumerate() {
                                vertices.push(NprGpuVertex {
                                    surface: triangle.surface[index].into(),
                                    position: to_clip(position),
                                    color,
                                    depth: triangle.depths[index],
                                    coverage: triangle.coverage * paint.wash,
                                    phase: [0.0; 2],
                                    material: [granulation, 0.0, 0.0, 0.0],
                                });
                            }
                        }
                        for buffer in NprPipelines::vertex_buffers(
                            &mut buffers,
                            &target.device,
                            &target.queue,
                            &vertices,
                            "amigo-npr-layer-underpainting",
                        ) {
                            color_batches.push(NprColorBatch::Paint {
                                mask: mask.clone(),
                                buffer,
                                blend: layer.blend,
                            });
                        }
                    }
                    amigo_render_npr::NprGeometrySource::FlatFill => {
                        let mut vertices = Vec::with_capacity(command.packet.fills.len() * 3);
                        for triangle in command.packet.fills.iter().filter(|triangle| {
                            triangle.layer_id.as_deref() == Some(layer.id.as_str())
                        }) {
                            let mut color = layer_color(
                                layer.color_source,
                                triangle.color.to_array(),
                                command.material_base_color,
                            );
                            color[3] *= layer.opacity;
                            for (index, position) in triangle.positions.into_iter().enumerate() {
                                vertices.push(NprGpuVertex {
                                    surface: triangle.surface[index].into(),
                                    position: to_clip(position),
                                    color,
                                    depth: triangle.depths[index],
                                    coverage: 1.0,
                                    phase: [0.0; 2],
                                    material: [0.0; 4],
                                });
                            }
                        }
                        for buffer in NprPipelines::vertex_buffers(
                            &mut buffers,
                            &target.device,
                            &target.queue,
                            &vertices,
                            "amigo-npr-layer-fill",
                        ) {
                            color_batches.push(NprColorBatch::Fill {
                                mask: mask.clone(),
                                buffer,
                                blend: layer.blend,
                            });
                        }
                    }
                    amigo_render_npr::NprGeometrySource::ShadowHatch
                    | amigo_render_npr::NprGeometrySource::FormLines
                    | amigo_render_npr::NprGeometrySource::Silhouette
                    | amigo_render_npr::NprGeometrySource::Creases
                    | amigo_render_npr::NprGeometrySource::Construction => {
                        let mut vertices = Vec::new();
                        let mut indices = Vec::new();
                        for stroke in &command.packet.strokes {
                            if stroke.layer_id.as_deref() != Some(layer.id.as_str()) {
                                continue;
                            }
                            let vertex_bytes = stroke
                                .vertices
                                .len()
                                .checked_mul(std::mem::size_of::<NprGpuVertex>());
                            let index_bytes =
                                stroke.indices.len().checked_mul(std::mem::size_of::<u32>());
                            let Some(stroke_bytes) = vertex_bytes.and_then(|bytes| {
                                index_bytes.and_then(|indices| bytes.checked_add(indices))
                            }) else {
                                continue;
                            };
                            let current_bytes = vertices.len()
                                * std::mem::size_of::<NprGpuVertex>()
                                + indices.len() * std::mem::size_of::<u32>();
                            if !indices.is_empty()
                                && current_bytes.saturating_add(stroke_bytes)
                                    > MAX_STROKE_BATCH_BYTES
                            {
                                if let Some(buffer) = NprPipelines::indexed_buffer(
                                    &mut buffers,
                                    &target.device,
                                    &target.queue,
                                    &vertices,
                                    &indices,
                                    "amigo-npr-layer-strokes",
                                ) {
                                    color_batches.push(NprColorBatch::Stroke {
                                        mask: mask.clone(),
                                        buffer,
                                        blend: layer.blend,
                                    });
                                }
                                vertices.clear();
                                indices.clear();
                            }
                            if stroke_bytes > MAX_STROKE_BATCH_BYTES
                                || stroke_upload_bytes.saturating_add(stroke_bytes)
                                    > MAX_STROKE_UPLOAD_BYTES
                                || stroke
                                    .indices
                                    .iter()
                                    .any(|index| *index as usize >= stroke.vertices.len())
                            {
                                continue;
                            }
                            let mut color = layer_color(
                                layer.color_source,
                                command.packet.stroke_color(stroke),
                                command.material_base_color,
                            );
                            color[3] *= layer.opacity;
                            let base = vertices.len() as u32;
                            vertices.extend(stroke.vertices.iter().map(|vertex| NprGpuVertex {
                                surface: vertex.surface.into(),
                                position: to_clip(vertex.position),
                                color,
                                depth: (vertex.depth - 0.00001).max(0.0),
                                coverage: vertex.coverage,
                                phase: [vertex.edge, vertex.grain],
                                material: [
                                    vertex.edge_softness,
                                    vertex.pressure,
                                    vertex.paper_tooth,
                                    vertex.dryness,
                                ],
                            }));
                            indices.extend(stroke.indices.iter().map(|index| base + index));
                            stroke_upload_bytes += stroke_bytes;
                        }
                        if let Some(buffer) = NprPipelines::indexed_buffer(
                            &mut buffers,
                            &target.device,
                            &target.queue,
                            &vertices,
                            &indices,
                            "amigo-npr-layer-strokes",
                        ) {
                            color_batches.push(NprColorBatch::Stroke {
                                mask: mask.clone(),
                                buffer,
                                blend: layer.blend,
                            });
                        }
                    }
                    // Paper is executed from the shared NprBackgroundCommand.
                    amigo_render_npr::NprGeometrySource::Paper => {}
                }
            }
        }
        if occluder_vertices.is_empty()
            && color_batches.is_empty()
            && paper_vertices.is_empty()
            && background.is_none()
        {
            return Ok(());
        }

        let no_mask = renderer.npr_pipelines.mask_binding(&mut buffers,&target.device,&target.queue,&amigo_render_npr::CoverageMask::None).map_err(amigo_core::AmigoError::Message)?;
        let occluder_buffers = NprPipelines::vertex_buffers(
            &mut buffers,
            &target.device,
            &target.queue,
            &occluder_vertices,
            "amigo-npr-occluder-vertices",
        );
        let background_load = background
            .map(|background| {
                let color = background.color;
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: color[0] as f64,
                    g: color[1] as f64,
                    b: color[2] as f64,
                    a: color[3] as f64,
                })
            })
            .unwrap_or(wgpu::LoadOp::Load);
        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-npr-passes"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-npr-depth-pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&renderer.npr_pipelines.depth);
            pass.set_bind_group(0,&no_mask,&[]);
            for buffer in &occluder_buffers {
                pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                pass.draw(0..buffer.vertex_count, 0..1);
            }
        }
        let paper_buffer = (!paper_vertices.is_empty()).then(|| {
            NprPipelines::vertex_buffer(
                &mut buffers,
                &target.device,
                &target.queue,
                &paper_vertices,
                "amigo-npr-paper",
            )
        });
        buffers.finish();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-npr-color-layers-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: background_load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            if let Some(paper_buffer) = &paper_buffer {
                pass.set_pipeline(&renderer.npr_pipelines.paper);
                pass.set_bind_group(0,&no_mask,&[]);
                pass.set_vertex_buffer(0, paper_buffer.slice(..));
                pass.draw(0..paper_vertices.len() as u32, 0..1);
            }
            for batch in &color_batches {
                match batch {
                    NprColorBatch::Paint { buffer, blend, mask } => {
                        pass.set_pipeline(renderer.npr_pipelines.paint_for(*blend));
                        pass.set_bind_group(0,mask,&[]);
                        pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                        pass.draw(0..buffer.vertex_count, 0..1);
                    }
                    NprColorBatch::Fill { buffer, blend, mask } => {
                        pass.set_pipeline(renderer.npr_pipelines.fill_for(*blend));
                        pass.set_bind_group(0,mask,&[]);
                        pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                        pass.draw(0..buffer.vertex_count, 0..1);
                    }
                    NprColorBatch::Stroke { buffer, blend, mask } => {
                        pass.set_pipeline(renderer.npr_pipelines.stroke_for(*blend));
                        pass.set_bind_group(0,mask,&[]);
                        pass.set_vertex_buffer(0, buffer.vertices.slice(..));
                        pass.set_index_buffer(buffer.indices.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..buffer.index_count, 0, 0..1);
                    }
                }
            }
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::ordered_npr_layer_ids;
    use amigo_render_npr::NprStyleLayers;

    fn layers(layer_ids: &[&str]) -> NprStyleLayers {
        let mut layers = NprStyleLayers::default();
        layers
            .layers
            .retain(|layer| layer_ids.contains(&layer.id.as_str()));
        layers
    }

    #[test]
    fn layer_order_is_global_across_npr_commands() {
        let first = layers(&["hatching", "contours"]);
        let second = layers(&["contours", "hatching"]);
        assert_eq!(
            ordered_npr_layer_ids([&first, &second]),
            vec!["hatching", "contours"]
        );
    }
}
