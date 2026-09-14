use super::material_candidates::WgpuMaterialCandidate2d;
use super::*;
use crate::renderer::npr::{NprGpuVertex, NprIndexedBuffer, NprPipelines, NprVertexBuffer};
use amigo_material_api::MaterialCandidateDecision2d;
use amigo_render_api::{
    LightSource2dCommon, NprBackgroundCommand, NprDrawCommand, NprMeshDrawCommand,
    RenderAssetSource, RenderLightMap2dSource,
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
    pub npr_meshes: &'a [NprMeshDrawCommand],
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
            npr_meshes: request.world_3d.npr_meshes,
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
    renderer.npr_stroke_history.begin_frame();
    let mut color_batches = Vec::new();
    let mut texture_batches = Vec::new();
    let mut ui_texture_batches = Vec::new();
    if let Some(style) = ctx.npr_meshes.first().map(|command| command.style) {
        let color = ColorRgba::new(
            style.background[0],
            style.background[1],
            style.background[2],
            style.background[3],
        );
        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        push_quad(
            vertices,
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            color,
        );
    }
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
    renderer.npr_stroke_history.observe_camera(camera);
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
                command.mesh.geometry.as_deref(),
                color,
                render_order,
            );
        }
    }

    if selection.layer_filter.allows_layerless() && !ctx.npr_meshes.is_empty() {
        for command in ctx.npr_meshes {
            let transform = resolve_transform3(
                ctx.scene_view,
                &command.mesh.entity_name,
                command.mesh.mesh.transform,
            );
            append_npr_mesh_triangles(
                &mut projected_triangles,
                &viewport,
                camera,
                camera_settings,
                transform,
                command.mesh.entity_id,
                command.mesh.mesh.geometry.as_deref(),
                command.style,
                &mut renderer.npr_stroke_history,
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

    // Establish surface visibility before drawing translucent graphite. Strokes
    // test this depth but must not occlude one another through empty coverage.
    for triangle in &projected_triangles {
        let vertices = world_depth_vertices(&mut color_batches, false, true);
        if triangle.color.a > 0.0 {
            for (point, depth) in triangle.points.into_iter().zip(triangle.depths) {
                let mut vertex = ColorVertex::new(point, triangle.color);
                vertex.depth = 0.001 / depth;
                vertices.push(vertex);
            }
        }
    }
    let mut hatch_segments = Vec::new();
    for triangle in &projected_triangles {
        if let Some(hatch) = triangle.hatch {
            collect_npr_triangle_hatching(&mut hatch_segments, &viewport, triangle.points, hatch);
        }
    }
    if !hatch_segments.is_empty() {
        let hatch_vertices = world_depth_vertices(&mut color_batches, true, false);
        push_npr_hatch_segments(hatch_vertices, &viewport, &mut hatch_segments);
    }
    for triangle in projected_triangles {
        if let Some(ink) = triangle.ink {
            for (edge, style, depths) in [
                ([triangle.points[0], triangle.points[1]], ink.edges[0], [triangle.depths[0], triangle.depths[1]]),
                ([triangle.points[1], triangle.points[2]], ink.edges[1], [triangle.depths[1], triangle.depths[2]]),
                ([triangle.points[2], triangle.points[0]], ink.edges[2], [triangle.depths[2], triangle.depths[0]]),
            ] {
                if let Some(mut style) = style {
                    style.depths = depths;
                    let ink_vertices = world_depth_vertices(&mut color_batches, style.pencil_grain > 0.0, false);
                    push_npr_ink_edge(ink_vertices, &viewport, edge, style);
                }
            }
        }
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

fn world_depth_vertices(batches: &mut Vec<ColorBatch>, pencil: bool, depth_write: bool) -> &mut Vec<ColorVertex> {
    if batches.last().is_none_or(|b| !b.world_depth || b.pencil != pencil || b.depth_write != depth_write) {
        batches.push(ColorBatch { world_depth: true, depth_write, pencil, blend_mode: ParticleBlendMode2d::Alpha, vertices: Vec::new() });
    }
    &mut batches.last_mut().unwrap().vertices
}

struct ProjectedHatchSegment {
    points: [Vec2; 2],
    style: ProjectedInkEdge,
    family: u64,
}

fn collect_npr_triangle_hatching(
    segments: &mut Vec<ProjectedHatchSegment>,
    viewport: &Viewport,
    points: [Vec2; 3],
    style: ProjectedTriangleHatch,
) {
    collect_npr_hatch_family(segments, viewport, points, style, style.angle_degrees, 0);
    if style.cross > 0.01 && npr_unit_noise(style.seed, 0xc205_7a11) < style.cross {
        collect_npr_hatch_family(
            segments,
            viewport,
            points,
            style,
            style.angle_degrees + 78.0,
            1,
        );
    }
}

fn collect_npr_hatch_family(
    segments: &mut Vec<ProjectedHatchSegment>,
    viewport: &Viewport,
    points: [Vec2; 3],
    style: ProjectedTriangleHatch,
    angle_degrees: f32,
    family: u64,
) {
    let radians = angle_degrees.to_radians();
    let direction = Vec2::new(radians.cos(), radians.sin());
    let grid_normal = Vec2::new(-direction.y, direction.x);
    let pixel_points = points.map(|point| {
        Vec2::new(
            point.x * viewport.half_width,
            point.y * viewport.half_height,
        )
    });
    let projections = style.surface_coordinates.map(|point| point.x * grid_normal.x + point.y * grid_normal.y);
    let minimum = projections.into_iter().fold(f32::INFINITY, f32::min);
    let maximum = projections
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max);
    let spacing = style.spacing_pixels.max(4.0) * if family == 0 { 1.0 } else { 1.2 };
    let first = (minimum / spacing).ceil() as i32;
    let last = (maximum / spacing).floor() as i32;

    // The cap protects the frame budget from a single huge near-plane
    // triangle. Ordinary character triangles intersect zero or one grid line.
    for grid_index in first..=last.min(first + 15) {
        // One deterministic displacement per lattice line makes the field feel
        // hand-placed while the entity seed keeps shared triangle seams aligned.
        let offset = grid_index as f32 * spacing
            + npr_noise(style.seed ^ family.wrapping_mul(0x517c_c1b7), grid_index as u64)
                * style.wobble_pixels.min(spacing * 0.18);
        let mut intersections = [Vec2::ZERO; 3];
        let mut intersection_depths = [1.0; 3];
        let mut count = 0usize;
        for edge_index in 0..3 {
            let a = pixel_points[edge_index];
            let b = pixel_points[(edge_index + 1) % 3];
            let da = projections[edge_index] - offset;
            let db = projections[(edge_index + 1) % 3] - offset;
            if da * db > 0.0 || (da - db).abs() <= 1.0e-5 {
                continue;
            }
            let surface_t = da / (da - db);
            let za = style.depths[edge_index];
            let zb = style.depths[(edge_index + 1) % 3];
            let t = surface_t * zb / ((1.0 - surface_t) * za + surface_t * zb);
            let point = Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
            let previous = intersections[count.saturating_sub(1)];
            let separation =
                (point.x - previous.x).powi(2) + (point.y - previous.y).powi(2);
            if count == 0 || separation > 0.01 {
                intersections[count] = point;
                intersection_depths[count] = (1.0 - surface_t) * za + surface_t * zb;
                count += 1;
                if count == 2 {
                    break;
                }
            }
        }
        if count != 2 {
            continue;
        }
        let clip = [intersections[0], intersections[1]].map(|point| {
            Vec2::new(point.x / viewport.half_width, point.y / viewport.half_height)
        });
        let seed = style.seed
            ^ (grid_index as i64 as u64).wrapping_mul(0x9e37_79b9)
            ^ family.wrapping_mul(0xa076_1d64);
        // Preserve a phase for the whole model-space lattice line. Without
        // this, pressure/taper restart at 0..1 for every clipped triangle,
        // which makes a single hatch stroke visibly segmented by topology.
        // The wrapped phase is intentionally bounded because the stroke
        // noise functions use a normalized gesture domain.
        let lattice_phase = (grid_index.rem_euclid(64) as f32 + 0.5) / 64.0;
        let lattice_span = ((clip[1].x - clip[0].x).powi(2)
            + (clip[1].y - clip[0].y).powi(2))
            .sqrt()
            .clamp(0.01, 0.18);
        segments.push(ProjectedHatchSegment {
            points: clip,
            family,
            style: ProjectedInkEdge {
                depths: [intersection_depths[0], intersection_depths[1]],
                color: style.color,
                width_pixels: style.width_pixels,
                wobble_pixels: style.wobble_pixels,
                stroke_segments: style.stroke_segments.max(2),
                taper: 0.08,
                overstroke: 0.0,
                hardness: style.hardness,
                dryness: style.dryness,
                pencil_grain: style.pencil_grain,
                pressure_variation: style.pressure_variation,
                gesture_t0: lattice_phase,
                gesture_t1: (lattice_phase + lattice_span).min(0.995),
                persistent: false,
                stable_seed: seed,
                seed,
            },
        });
    }
}

fn push_npr_hatch_segments(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    segments: &mut [ProjectedHatchSegment],
) {
    // Segments from one lattice line are joined before tessellation. This
    // preserves one pressure/noise trajectory through adjacent triangles and
    // prevents visible resets at topology boundaries.
    // A city mesh can contain tens of thousands of clipped hatch segments.
    // Keep the work bounded; the common lattice phase still makes those
    // segments coherent without an O(n²) adjacency search.
    if segments.len() > 2048 {
        for segment in segments.iter() {
            push_npr_ink_edge(vertices, viewport, segment.points, segment.style);
        }
        return;
    }
    segments.sort_by_key(|segment| (segment.style.stable_seed, segment.family));
    let mut consumed = vec![false; segments.len()];
    let join_distance = 2.5 / viewport.half_width.min(viewport.half_height).max(1.0);
    for start in 0..segments.len() {
        if consumed[start] {
            continue;
        }
        consumed[start] = true;
        let mut chain = vec![start];
        let mut tail = segments[start].points[1];
        loop {
            let Some((next, reverse)) = (0..segments.len()).find_map(|index| {
                if consumed[index]
                    || segments[index].style.stable_seed != segments[start].style.stable_seed
                    || segments[index].family != segments[start].family
                {
                    return None;
                }
                let [a, b] = segments[index].points;
                let distance = |point: Vec2| {
                    ((point.x - tail.x).powi(2) + (point.y - tail.y).powi(2)).sqrt()
                };
                if distance(a) <= join_distance {
                    Some((index, false))
                } else if distance(b) <= join_distance {
                    Some((index, true))
                } else {
                    None
                }
            }) else {
                break;
            };
            consumed[next] = true;
            if reverse {
                segments[next].points.swap(0, 1);
                segments[next].style.depths.swap(0, 1);
            }
            tail = segments[next].points[1];
            chain.push(next);
        }
        let total = chain
            .iter()
            .map(|&index| {
                let [a, b] = segments[index].points;
                ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
            })
            .sum::<f32>()
            .max(1.0e-6);
        let mut travelled = 0.0;
        for index in chain {
            let segment = &segments[index];
            let length = ((segment.points[1].x - segment.points[0].x).powi(2)
                + (segment.points[1].y - segment.points[0].y).powi(2))
                .sqrt();
            let mut style = segment.style;
            let phase = style.gesture_t0;
            let available = (0.995 - phase).max(0.01);
            style.gesture_t0 = phase + available * travelled / total;
            style.gesture_t1 = phase + available * (travelled + length) / total;
            push_npr_ink_edge(vertices, viewport, segment.points, style);
            travelled += length;
        }
    }
}

fn push_npr_ink_edge(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    [a, b]: [Vec2; 2],
    style: ProjectedInkEdge,
) {
    let delta = Vec2::new(b.x - a.x, b.y - a.y);
    let pixel_delta = Vec2::new(
        delta.x * viewport.half_width,
        delta.y * viewport.half_height,
    );
    let length = (pixel_delta.x * pixel_delta.x + pixel_delta.y * pixel_delta.y).sqrt();
    if !length.is_finite() || length <= 0.001 {
        return;
    }
    let unit_normal_pixels = Vec2::new(-pixel_delta.y / length, pixel_delta.x / length);
    // Long strokes get more correlated samples; tiny edges remain cheap. The
    // samples are not independent noise: `npr_smooth_noise` gives the hand a
    // continuous gesture rather than a ruler edge with pixel jitter.
    let segments = ((length / 28.0).ceil() as usize)
        .clamp(2, style.stroke_segments.max(2) as usize);
    push_npr_ink_pass(
        vertices,
        viewport,
        a,
        b,
        unit_normal_pixels,
        segments,
        style,
        0,
    );
    if style.overstroke > 0.01
        && npr_unit_noise(style.stable_seed, 0x6eed_0e9d) < style.overstroke
    {
        push_npr_ink_pass(
            vertices,
            viewport,
            a,
            b,
            unit_normal_pixels,
            segments,
            style,
            1,
        );
    }
    // A soft graphite tool leaves a lighter, slightly displaced second trace.
    // It is spatially coherent within one artistic frame; the global redraw
    // epoch supplies a fresh realization for every mesh on the next frame.
    if style.pencil_grain > 0.0 && style.hardness < 0.5 && style.dryness > 0.08 {
        push_npr_ink_pass(
            vertices,
            viewport,
            a,
            b,
            unit_normal_pixels,
            segments,
            style,
            2,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_npr_ink_pass(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    a: Vec2,
    b: Vec2,
    normal_pixels: Vec2,
    segments: usize,
    style: ProjectedInkEdge,
    pass: u64,
) {
    let sample = |index: usize| {
        let t = index as f32 / segments as f32;
        let global_t = style.gesture_t0 + (style.gesture_t1 - style.gesture_t0) * t;
        let pass_seed = style.seed ^ pass.wrapping_mul(0x517c_c1b7);
        // Keep most of the hand bias stable, but let a smaller realization
        // component change on an artistic frame. This is correlated motion,
        // not independent per-vertex noise.
        let broad_noise = npr_smooth_noise_scaled(
            style.stable_seed ^ pass_seed,
            global_t,
            1.15,
        ) * 0.68
            + npr_smooth_noise_scaled(pass_seed, global_t, 1.15) * 0.32;
        let wrist_noise = npr_smooth_noise_scaled(
            style.stable_seed ^ pass_seed ^ 0x4a5f_1d2b,
            global_t,
            3.8,
        ) * 0.72
            + npr_smooth_noise_scaled(pass_seed ^ 0x4a5f_1d2b, global_t, 3.8) * 0.28;
        let tooth_noise = npr_smooth_noise_scaled(
            pass_seed ^ 0x9d2c_5680,
            global_t,
            11.0,
        );
        let normal_noise = broad_noise * 0.72 + wrist_noise * 0.22 + tooth_noise * 0.06;
        let tangent_noise = wrist_noise * 0.12 + tooth_noise * 0.035;
        // A hand does not hit the exact vertex at the end of every contour.
        // Only the outer ends of a joined gesture receive this aim error;
        // interior chain segments remain connected.
        let start_error = if style.gesture_t0 <= 0.001 {
            npr_noise(style.stable_seed ^ pass_seed ^ 0x1f12_9a43, 0) * 0.42
        } else {
            0.0
        };
        let end_error = if style.gesture_t1 >= 0.999 {
            npr_noise(style.stable_seed ^ pass_seed ^ 0x8b7d_31e1, 1) * 0.42
        } else {
            0.0
        };
        let endpoint_error = start_error * (1.0 - t).powi(2) + end_error * t.powi(2);
        let endpoint_overshoot = if style.gesture_t1 >= 0.999 {
            (t * t * (3.0 - 2.0 * t)) * npr_noise(style.stable_seed ^ 0x63d8_9b11, 2) * 0.18
        } else {
            0.0
        };
        let overstroke_offset = if pass == 0 {
            0.0
        } else if pass == 2 {
            -style.wobble_pixels.max(0.35) * 0.32
        } else {
            style.wobble_pixels.max(0.4) * 0.45
        };
        let offset_pixels = (normal_noise * style.wobble_pixels
            + endpoint_error
            + overstroke_offset)
            * (0.92 + 0.08 * (std::f32::consts::PI * t).sin());
        let tangent_pixels = (tangent_noise + endpoint_overshoot) * style.wobble_pixels * 0.35;
        let base = Vec2::new(
            a.x + (b.x - a.x) * t + (b.x - a.x) / length_or_one(a, b) * tangent_pixels / viewport.half_width,
            a.y + (b.y - a.y) * t + (b.y - a.y) / length_or_one(a, b) * tangent_pixels / viewport.half_height,
        );
        Vec2::new(
            base.x + normal_pixels.x * offset_pixels / viewport.half_width,
            base.y + normal_pixels.y * offset_pixels / viewport.half_height,
        )
    };
    let mut color = style.color;
    color.a *= if pass == 0 {
        0.68 + style.hardness * 0.32
    } else if pass == 2 {
        0.16 + style.hardness * 0.12
    } else {
        0.36 + style.hardness * 0.16
    };
    if segments == 1 {
        color.a *= 1.0 - style.dryness * 0.28;
    }
    for index in 0..segments {
        if pass != 0
            && segments > 1
            && npr_unit_noise(style.stable_seed ^ 0xa076_1d64 ^ pass, index as u64)
            < style.dryness * if pass == 2 { 0.28 } else { 0.14 }
        {
            continue;
        }
        let t0 = index as f32 / segments as f32;
        let t1 = (index + 1) as f32 / segments as f32;
        let gesture_t0 = style.gesture_t0 + (style.gesture_t1 - style.gesture_t0) * t0;
        let gesture_t1 = style.gesture_t0 + (style.gesture_t1 - style.gesture_t0) * t1;
        let taper = |t: f32| 1.0 - style.taper * (2.0 * (t - 0.5).abs()) * 0.45;
        let pressure0 = 1.0
            + style.pressure_variation
                * (npr_smooth_noise(style.stable_seed ^ 0xe703_7ed1, gesture_t0) * 0.72
                    + npr_smooth_noise(style.seed ^ 0xe703_7ed1, gesture_t0) * 0.28);
        let pressure1 = 1.0
            + style.pressure_variation
                * (npr_smooth_noise(style.stable_seed ^ 0xe703_7ed1, gesture_t1) * 0.72
                    + npr_smooth_noise(style.seed ^ 0xe703_7ed1, gesture_t1) * 0.28);
        let grain0 = 1.0 + style.pencil_grain * 0.18 * npr_smooth_noise(style.seed ^ 0x8d31_4c27, gesture_t0);
        let grain1 = 1.0 + style.pencil_grain * 0.18 * npr_smooth_noise(style.seed ^ 0x8d31_4c27, gesture_t1);
        let half0 = (style.width_pixels * taper(gesture_t0) * pressure0 * grain0).max(0.5) * 0.5;
        let half1 = (style.width_pixels * taper(gesture_t1) * pressure1 * grain1).max(0.5) * 0.5;
        let p0 = sample(index);
        let p1 = sample(index + 1);
        let n0 = Vec2::new(
            normal_pixels.x * half0 / viewport.half_width,
            normal_pixels.y * half0 / viewport.half_height,
        );
        let n1 = Vec2::new(
            normal_pixels.x * half1 / viewport.half_width,
            normal_pixels.y * half1 / viewport.half_height,
        );
        let mut segment_color = color;
        let segment_pressure = ((pressure0 + pressure1) * 0.5).clamp(0.55, 1.45);
        segment_color.a *= 0.82 + 0.18 * segment_pressure;
        let seed = npr_unit_noise(style.stable_seed, pass) * 256.0;
        let make_vertex = |point: Vec2, t: f32, lateral: f32, pressure: f32, width: f32| {
            let mut vertex = ColorVertex::new(point, segment_color);
            vertex.stroke = [t, lateral, pressure, seed];
            vertex.graphite = [style.pencil_grain, style.hardness, style.dryness, width];
            let local_t = (t - style.gesture_t0) / (style.gesture_t1 - style.gesture_t0).max(1.0e-6);
            vertex.depth = 0.001 * ((1.0 - local_t) / style.depths[0] + local_t / style.depths[1]) * 1.0005;
            vertex
        };
        let a = make_vertex(Vec2::new(p0.x + n0.x, p0.y + n0.y), gesture_t0, 1.0, pressure0, half0 * 2.0);
        let b = make_vertex(Vec2::new(p1.x + n1.x, p1.y + n1.y), gesture_t1, 1.0, pressure1, half1 * 2.0);
        let c = make_vertex(Vec2::new(p1.x - n1.x, p1.y - n1.y), gesture_t1, -1.0, pressure1, half1 * 2.0);
        let d = make_vertex(Vec2::new(p0.x - n0.x, p0.y - n0.y), gesture_t0, -1.0, pressure0, half0 * 2.0);
        vertices.extend_from_slice(&[a, b, c, a, c, d]);
    }
}

fn npr_unit_noise(seed: u64, index: u64) -> f32 {
    let mut value = seed ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    (value as u32) as f32 / u32::MAX as f32
}

fn npr_noise(seed: u64, index: u64) -> f32 {
    npr_unit_noise(seed, index) * 2.0 - 1.0
}

fn npr_smooth_noise(seed: u64, t: f32) -> f32 {
    let coordinate = t.clamp(0.0, 1.0) * 3.0;
    let index = coordinate.floor() as u64;
    let fraction = coordinate - index as f32;
    let smooth = fraction * fraction * (3.0 - 2.0 * fraction);
    let left = npr_noise(seed, index);
    let right = npr_noise(seed, index + 1);
    left + (right - left) * smooth
}

fn npr_smooth_noise_scaled(seed: u64, t: f32, scale: f32) -> f32 {
    let coordinate = (t.clamp(0.0, 1.0) * scale).max(0.0);
    let index = coordinate.floor() as u64;
    let fraction = coordinate - index as f32;
    let smooth = fraction * fraction * (3.0 - 2.0 * fraction);
    let left = npr_noise(seed, index);
    let right = npr_noise(seed, index + 1);
    left + (right - left) * smooth
}

fn length_or_one(a: Vec2, b: Vec2) -> f32 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt().max(1.0e-6)
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
                layer
                    .validate_mask_inputs(&command.packet)
                    .map_err(amigo_core::AmigoError::Message)?;
                let mask = renderer
                    .npr_pipelines
                    .mask_binding(&mut buffers, &target.device, &target.queue, &layer.mask)
                    .map_err(amigo_core::AmigoError::Message)?;
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

        let no_mask = renderer
            .npr_pipelines
            .mask_binding(
                &mut buffers,
                &target.device,
                &target.queue,
                &amigo_render_npr::CoverageMask::None,
            )
            .map_err(amigo_core::AmigoError::Message)?;
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
            pass.set_bind_group(0, &no_mask, &[]);
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
                pass.set_bind_group(0, &no_mask, &[]);
                pass.set_vertex_buffer(0, paper_buffer.slice(..));
                pass.draw(0..paper_vertices.len() as u32, 0..1);
            }
            for batch in &color_batches {
                match batch {
                    NprColorBatch::Paint {
                        buffer,
                        blend,
                        mask,
                    } => {
                        pass.set_pipeline(renderer.npr_pipelines.paint_for(*blend));
                        pass.set_bind_group(0, mask, &[]);
                        pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                        pass.draw(0..buffer.vertex_count, 0..1);
                    }
                    NprColorBatch::Fill {
                        buffer,
                        blend,
                        mask,
                    } => {
                        pass.set_pipeline(renderer.npr_pipelines.fill_for(*blend));
                        pass.set_bind_group(0, mask, &[]);
                        pass.set_vertex_buffer(0, buffer.buffer.slice(..));
                        pass.draw(0..buffer.vertex_count, 0..1);
                    }
                    NprColorBatch::Stroke {
                        buffer,
                        blend,
                        mask,
                    } => {
                        pass.set_pipeline(renderer.npr_pipelines.stroke_for(*blend));
                        pass.set_bind_group(0, mask, &[]);
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

        // Preserve painter ordering across materials without allocating one GPU
        // resource for every fill/stroke transition in a triangulated model.
        let color_vertices: Vec<_> = color_batches.iter()
            .flat_map(|batch| batch.vertices.iter().copied()).collect();
        let color_vertex_buffer = (!color_vertices.is_empty()).then(|| {
            target.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-offscreen-color-vertices"),
                contents: vertices_as_bytes(&color_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
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

        let first_world = color_batches.iter().position(|b| b.world_depth).unwrap_or(color_batches.len());
        let after_world = color_batches.iter().rposition(|b| b.world_depth).map_or(first_world, |i| i + 1);
        debug_assert!(color_batches[first_world..after_world].iter().all(|b| b.world_depth));
        let phases = [0..first_world, first_world..after_world, after_world..color_batches.len()];
        let mut first_vertex = 0;
        for (phase, range) in phases.into_iter().enumerate() {
            if phase == 1 && range.is_empty() {
                continue;
            }
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(match phase {
                    0 => "amigo-offscreen-background-pass",
                    1 => "amigo-offscreen-world-depth-pass",
                    _ => "amigo-offscreen-overlay-pass",
                }),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: if phase == 0 { load_op } else { wgpu::LoadOp::Load },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: (phase == 1).then(|| wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in texture_batches.iter().enumerate().filter(|_| phase == 0) {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            if let Some(buffer) = &color_vertex_buffer {
                pass.set_vertex_buffer(0, buffer.slice(..));
            }
            for batch in &color_batches[range] {
                if batch.world_depth {
                    pass.set_pipeline(self.pipeline(match (batch.pencil, batch.depth_write) {
                        (false, true) => "core.world-depth.color",
                        (true, true) => "core.world-depth.pencil",
                        (false, false) => "core.world-depth.color-read",
                        (true, false) => "core.world-depth.pencil-read",
                    }));
                } else {
                    pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                }
                let end_vertex = first_vertex + batch.vertices.len() as u32;
                pass.draw(first_vertex..end_vertex, 0..1);
                first_vertex = end_vertex;
            }

            for (index, batch) in ui_texture_batches.iter().enumerate().filter(|_| phase == 2) {
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
    use super::{npr_noise, ordered_npr_layer_ids};
    use amigo_render_npr::NprStyleLayers;

    #[test]
    fn world_depth_resolves_crossings_preserves_graphite_overlap_and_ui() {
        use super::*;
        let mut target = crate::WgpuRenderBackend::default().initialize_offscreen(64, 64).unwrap();
        let renderer = WgpuSceneRenderer::new_for_offscreen(&target);
        let quad = |color: ColorRgba, left_depth: f32, right_depth: f32| {
            [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0),
             (-1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(x, y)| {
                let mut vertex = ColorVertex::new(Vec2::new(x, y), color);
                vertex.depth = if x < 0.0 { left_depth } else { right_depth };
                vertex
            })
        };
        let render = |target: &mut WgpuOffscreenTarget, batches: &[ColorBatch]| {
            renderer.render_offscreen_batches(target, wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                &[], batches, &[]).unwrap();
            target.read_rgba8_blocking().unwrap()
        };
        let pixel = |image: &[u8], x: usize| -> [u8; 3] {
            image[(32 * 64 + x) * 4..(32 * 64 + x) * 4 + 3].try_into().unwrap()
        };
        let mut batches = Vec::new();
        let surfaces = world_depth_vertices(&mut batches, false, true);
        surfaces.extend(quad(ColorRgba::new(1.0, 0.0, 0.0, 1.0), 0.1, 0.9));
        surfaces.extend(quad(ColorRgba::new(0.0, 1.0, 0.0, 1.0), 0.5, 0.5));
        let pixels = render(&mut target, &batches);
        assert_eq!(pixel(&pixels, 16), [0, 255, 0], "left crossing must show green");
        assert_eq!(pixel(&pixels, 48), [255, 0, 0], "right crossing must show red despite draw order");

        batches.clear();
        world_depth_vertices(&mut batches, false, true)
            .extend(quad(ColorRgba::new(0.0, 0.0, 0.0, 1.0), 0.2, 0.2));
        let strokes = world_depth_vertices(&mut batches, false, false);
        strokes.extend(quad(ColorRgba::new(1.0, 0.0, 0.0, 0.5), 0.8, 0.8));
        strokes.extend(quad(ColorRgba::new(0.0, 1.0, 0.0, 0.5), 0.7, 0.7));
        let pixels = render(&mut target, &batches);
        let [red, green, _] = pixel(&pixels, 32);
        assert!(red > 100 && green > red, "strokes must blend without writing depth: {red}, {green}");

        color_batch_vertices(&mut batches, ParticleBlendMode2d::Alpha)
            .extend(quad(ColorRgba::new(0.0, 0.0, 1.0, 1.0), 0.0, 0.0));
        let pixels = render(&mut target, &batches);
        assert_eq!(pixel(&pixels, 32), [0, 0, 255], "UI must render without world depth");
        // Loading and pure UI frames also work after a frame using world depth.
        let pixels = render(&mut target, &batches[batches.len() - 1..]);
        assert_eq!(pixel(&pixels, 32), [0, 0, 255]);
    }

    #[test]
    fn graphite_coordinates_follow_stroke_when_projected_position_changes() {
        use super::*;
        let style = ProjectedInkEdge {
            depths: [3.0; 2],
            color: ColorRgba::new(0.1, 0.1, 0.1, 0.8), width_pixels: 3.0,
            wobble_pixels: 1.0, stroke_segments: 6, pressure_variation: 0.5,
            gesture_t0: 0.0, gesture_t1: 1.0, taper: 0.3, overstroke: 0.0,
            hardness: 0.3, dryness: 0.2, pencil_grain: 0.8,
            persistent: false, stable_seed: 41, seed: 41,
        };
        let viewport = Viewport::from_dimensions(512.0, 512.0);
        let mut first = Vec::new();
        let mut translated = Vec::new();
        push_npr_ink_edge(&mut first, &viewport, [Vec2::new(-0.5, 0.0), Vec2::new(0.5, 0.0)], style);
        push_npr_ink_edge(&mut translated, &viewport, [Vec2::new(-0.5, 0.25), Vec2::new(0.5, 0.25)], style);
        assert!(!first.is_empty());
        assert_eq!(first.len(), translated.len());
        for (a, b) in first.iter().zip(&translated) {
            assert_eq!(a.stroke, b.stroke, "graphite must move with the gesture");
            assert_eq!(a.graphite, b.graphite);
            assert!((b.position[1] - a.position[1] - 0.25).abs() < 1.0e-6);
        }
        assert!(first.iter().any(|v| v.stroke[1] == -1.0));
        assert!(first.iter().any(|v| v.stroke[1] == 1.0));
        let min_width = first.iter().map(|v| v.graphite[3]).fold(f32::INFINITY, f32::min);
        let max_width = first.iter().map(|v| v.graphite[3]).fold(0.0_f32, f32::max);
        assert!(max_width - min_width > 0.1, "pressure must affect the actual ribbon width");
    }

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

    #[test]
    fn hand_drawn_noise_is_stable_within_an_artistic_frame() {
        let first = npr_noise(41, 3);
        assert_eq!(first, npr_noise(41, 3));
        assert_ne!(first, npr_noise(42, 3));
        assert!((-1.0..=1.0).contains(&first));
    }

    #[test]
    fn pencil_gesture_does_not_snap_both_ends_to_the_mesh_edge() {
        use super::*;
        let style = ProjectedInkEdge {
            depths: [1.0; 2],
            color: ColorRgba::new(0.1, 0.1, 0.1, 0.8),
            width_pixels: 2.0,
            wobble_pixels: 1.0,
            stroke_segments: 6,
            pressure_variation: 0.35,
            gesture_t0: 0.0,
            gesture_t1: 1.0,
            taper: 0.2,
            overstroke: 0.0,
            hardness: 0.3,
            dryness: 0.2,
            pencil_grain: 0.8,
            persistent: false,
            stable_seed: 17,
            seed: 91,
        };
        let viewport = Viewport::from_dimensions(512.0, 512.0);
        let mut vertices = Vec::new();
        push_npr_ink_edge(
            &mut vertices,
            &viewport,
            [Vec2::new(-0.5, 0.0), Vec2::new(0.5, 0.0)],
            style,
        );
        assert!(!vertices.is_empty());
        let first_y = vertices[0].position[1];
        let last_y = vertices[vertices.len() - 1].position[1];
        assert!(first_y.abs() > 1.0e-7 || last_y.abs() > 1.0e-7);
    }

    #[test]
    fn hatch_segments_share_one_gesture_phase_when_they_touch() {
        use super::*;
        let style = |depths| ProjectedInkEdge {
            depths,
            color: ColorRgba::new(0.1, 0.1, 0.1, 0.8),
            width_pixels: 1.5,
            wobble_pixels: 0.3,
            stroke_segments: 3,
            pressure_variation: 0.2,
            gesture_t0: 0.25,
            gesture_t1: 0.40,
            taper: 0.08,
            overstroke: 0.0,
            hardness: 0.3,
            dryness: 0.2,
            pencil_grain: 0.8,
            persistent: false,
            stable_seed: 12,
            seed: 12,
        };
        let mut segments = vec![
            ProjectedHatchSegment {
                points: [Vec2::new(-0.5, 0.0), Vec2::new(0.0, 0.0)],
                style: style([1.0, 1.0]),
                family: 0,
            },
            ProjectedHatchSegment {
                points: [Vec2::new(0.0, 0.0), Vec2::new(0.5, 0.0)],
                style: style([1.0, 1.0]),
                family: 0,
            },
        ];
        let mut vertices = Vec::new();
        push_npr_hatch_segments(
            &mut vertices,
            &Viewport::from_dimensions(512.0, 512.0),
            &mut segments,
        );
        let max_phase = vertices
            .iter()
            .map(|vertex| vertex.stroke[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max_phase > 0.55, "joined hatch must advance its gesture phase");
    }
}
