use crate::renderer::*;
use std::sync::Arc;

const NPR_BRUSH_RESAMPLE_SPACING_PX: f32 = 2.5;

#[derive(Debug, Clone)]
pub(crate) struct CachedMeshGeometry3d {
    vertices: Vec<Vec3>,
    triangles: Vec<MeshTriangle3d>,
    edges: Vec<MeshEdge3d>,
}

impl CachedMeshGeometry3d {
    pub(crate) fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    pub(crate) fn triangles(&self) -> &[MeshTriangle3d] {
        &self.triangles
    }

    pub(crate) fn edges(&self) -> &[MeshEdge3d] {
        &self.edges
    }

    pub(crate) fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub(crate) fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MeshTriangle3d {
    pub(crate) indices: [usize; 3],
    pub(crate) normal: Vec3,
    pub(crate) material_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub(crate) struct MeshEdge3d {
    pub(crate) edge_id: u64,
    pub(crate) a: usize,
    pub(crate) b: usize,
    pub(crate) faces: Vec<usize>,
    pub(crate) material_seam: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NprLineKind {
    Boundary,
    Silhouette,
    Crease,
    Seam,
    Feature,
    Contact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NprDebugOverlay3d {
    LineKinds,
    RawPaths,
    Dropout,
    WidthAlpha,
}

impl NprDebugOverlay3d {
    pub(crate) fn from_camera_debug_view(view: &amigo_render_api::CameraDebugView2d) -> Option<Self> {
        match view.as_str() {
            "npr.line_kinds" | "npr.kinds" => Some(Self::LineKinds),
            "npr.raw_paths" | "npr.paths" => Some(Self::RawPaths),
            "npr.dropout" | "npr.breakup" => Some(Self::Dropout),
            "npr.width_alpha" | "npr.pressure" => Some(Self::WidthAlpha),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct NprLineFragment {
    source_edge_id: u64,
    kind: NprLineKind,
    p0: Vec2,
    p1: Vec2,
    t0: f32,
    t1: f32,
    tangent0: Vec2,
    tangent1: Vec2,
    avg_depth: f32,
}

#[derive(Debug, Clone)]
struct NprStrokePath {
    path_id: u64,
    kind: NprLineKind,
    points: Vec<Vec2>,
    source_edges: Vec<u64>,
    sorted_source_edges: Vec<u64>,
    arc_lengths_px: Vec<f32>,
    importance: f32,
    closed: bool,
}

#[derive(Debug, Clone, Copy)]
struct NprBrushSample {
    point: Vec2,
    arc_length_px: f32,
}

#[derive(Debug, Clone)]
struct NprStableBrushPath {
    path_id: u64,
    samples: Vec<NprBrushSample>,
    length_px: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NprStrokePassKind {
    Primary,
    Search,
}

#[derive(Debug, Clone, Copy)]
struct NprToolDynamics {
    base_width_px: f32,
    base_wobble_px: f32,
    effective_overshoot_px: f32,
    edge_complexity: f32,
    protected_silhouette: bool,
}

#[derive(Debug, Clone, Copy)]
struct NprStrokeGesture {
    path_seed: u64,
    path_length_px: f32,
    importance: f32,
    dynamics: NprToolDynamics,
    style: NprResolvedKindStyle,
}

#[derive(Debug, Clone, Copy)]
struct NprStrokePassPlan {
    kind: NprStrokePassKind,
    pass_index: u8,
    wobble_px: f32,
    width_multiplier: f32,
    color: ColorRgba,
    overshoot_px: f32,
}

#[derive(Debug, Clone, Copy)]
struct NprDropoutInterval {
    pass_index: u8,
    t0: f32,
    t1: f32,
}

#[derive(Debug, Clone)]
struct NprDropoutMask {
    intervals: Vec<NprDropoutInterval>,
}

#[derive(Debug, Clone)]
struct NprCachedStrokePlan {
    settings_signature: u64,
    length_bucket_px: u32,
    passes: Vec<NprStrokePassPlan>,
    dropout: NprDropoutMask,
}

impl NprCachedStrokePlan {
    fn is_compatible(
        &self,
        settings: &amigo_render_api::NprLineSettings3d,
        gesture: NprStrokeGesture,
    ) -> bool {
        self.settings_signature == npr_stroke_plan_settings_signature(settings)
            && self.length_bucket_px == npr_stroke_plan_length_bucket(gesture.path_length_px)
    }
}

#[derive(Debug, Clone, Copy)]
struct NprStrokeStripSample {
    point: Vec2,
    width_px: f32,
    offset_px: f32,
    overshoot_px: f32,
    color: ColorRgba,
}

#[derive(Debug, Clone, Copy)]
struct NprStrokeRail {
    left: Vec2,
    right: Vec2,
    color: ColorRgba,
}

#[derive(Debug, Clone)]
pub(crate) struct NprTemporalPathState3d {
    path: NprStrokePath,
    cached_plan: Option<NprCachedStrokePlan>,
    missing_frames: u8,
    last_seen_frame: u64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NprEntityPathHistory3d {
    paths: BTreeMap<u64, NprTemporalPathState3d>,
}

#[derive(Debug, Clone, Default)]
pub struct NprStrokeFrameStats3d {
    pub meshes: usize,
    pub gpu_realtime_meshes: usize,
    pub cpu_reference_meshes: usize,
    pub gpu_realtime_enqueued_edges: usize,
    pub gpu_realtime_enqueued_triangles: usize,
    pub gpu_realtime_topology_uploads: usize,
    pub gpu_realtime_buffer_capacity_bytes: u64,
    pub gpu_realtime_frame_jobs: usize,
    pub gpu_realtime_projected_vertices_capacity: usize,
    pub gpu_realtime_visible_segments_capacity: usize,
    pub gpu_realtime_endpoint_heads_capacity: usize,
    pub gpu_realtime_endpoint_entries_capacity: usize,
    pub gpu_realtime_path_links_capacity: usize,
    pub gpu_realtime_path_states_capacity: usize,
    pub gpu_realtime_path_segments_capacity: usize,
    pub gpu_realtime_stroke_segments_capacity: usize,
    pub gpu_realtime_debug_mode: String,
    pub paths: usize,
    pub boundary_paths: usize,
    pub silhouette_paths: usize,
    pub crease_paths: usize,
    pub seam_paths: usize,
    pub feature_paths: usize,
    pub contact_paths: usize,
    pub brush_samples: usize,
    pub strip_vertices: usize,
    pub primary_passes: usize,
    pub search_passes: usize,
    pub dropout_intervals: usize,
    pub cached_plan_hits: usize,
    pub cached_plan_misses: usize,
    pub path_build_us: f64,
    pub stabilize_us: f64,
    pub stroke_vertices_us: f64,
    pub path_project_us: f64,
    pub path_visibility_us: f64,
    pub path_edge_sample_us: f64,
    pub path_stitch_us: f64,
    pub path_visible_edges: usize,
    pub path_fragments: usize,
}

impl NprStrokeFrameStats3d {
    pub(crate) fn record_strategy(
        &mut self,
        strategy: amigo_render_api::NprRenderStrategy3d,
    ) {
        match strategy {
            amigo_render_api::NprRenderStrategy3d::GpuRealtime => self.gpu_realtime_meshes += 1,
            amigo_render_api::NprRenderStrategy3d::CpuReference => self.cpu_reference_meshes += 1,
        }
    }

    fn record_path_kind(&mut self, kind: NprLineKind) {
        match kind {
            NprLineKind::Boundary => self.boundary_paths += 1,
            NprLineKind::Silhouette => self.silhouette_paths += 1,
            NprLineKind::Crease => self.crease_paths += 1,
            NprLineKind::Seam => self.seam_paths += 1,
            NprLineKind::Feature => self.feature_paths += 1,
            NprLineKind::Contact => self.contact_paths += 1,
        }
    }

    fn record_pass(&mut self, pass: NprStrokePassPlan) {
        match pass.kind {
            NprStrokePassKind::Primary => self.primary_passes += 1,
            NprStrokePassKind::Search => self.search_passes += 1,
        }
    }

    pub(crate) fn add(&mut self, other: Self) {
        self.meshes += other.meshes;
        self.gpu_realtime_meshes += other.gpu_realtime_meshes;
        self.cpu_reference_meshes += other.cpu_reference_meshes;
        self.gpu_realtime_enqueued_edges += other.gpu_realtime_enqueued_edges;
        self.gpu_realtime_enqueued_triangles += other.gpu_realtime_enqueued_triangles;
        self.gpu_realtime_topology_uploads += other.gpu_realtime_topology_uploads;
        self.gpu_realtime_buffer_capacity_bytes += other.gpu_realtime_buffer_capacity_bytes;
        self.gpu_realtime_frame_jobs += other.gpu_realtime_frame_jobs;
        self.gpu_realtime_projected_vertices_capacity +=
            other.gpu_realtime_projected_vertices_capacity;
        self.gpu_realtime_visible_segments_capacity +=
            other.gpu_realtime_visible_segments_capacity;
        self.gpu_realtime_endpoint_heads_capacity +=
            other.gpu_realtime_endpoint_heads_capacity;
        self.gpu_realtime_endpoint_entries_capacity +=
            other.gpu_realtime_endpoint_entries_capacity;
        self.gpu_realtime_path_links_capacity += other.gpu_realtime_path_links_capacity;
        self.gpu_realtime_path_states_capacity += other.gpu_realtime_path_states_capacity;
        self.gpu_realtime_path_segments_capacity +=
            other.gpu_realtime_path_segments_capacity;
        self.gpu_realtime_stroke_segments_capacity +=
            other.gpu_realtime_stroke_segments_capacity;
        if self.gpu_realtime_debug_mode.is_empty() {
            self.gpu_realtime_debug_mode = other.gpu_realtime_debug_mode.clone();
        } else if !other.gpu_realtime_debug_mode.is_empty()
            && self.gpu_realtime_debug_mode != other.gpu_realtime_debug_mode
        {
            self.gpu_realtime_debug_mode = "mixed".to_owned();
        }
        self.paths += other.paths;
        self.boundary_paths += other.boundary_paths;
        self.silhouette_paths += other.silhouette_paths;
        self.crease_paths += other.crease_paths;
        self.seam_paths += other.seam_paths;
        self.feature_paths += other.feature_paths;
        self.contact_paths += other.contact_paths;
        self.brush_samples += other.brush_samples;
        self.strip_vertices += other.strip_vertices;
        self.primary_passes += other.primary_passes;
        self.search_passes += other.search_passes;
        self.dropout_intervals += other.dropout_intervals;
        self.cached_plan_hits += other.cached_plan_hits;
        self.cached_plan_misses += other.cached_plan_misses;
        self.path_build_us += other.path_build_us;
        self.stabilize_us += other.stabilize_us;
        self.stroke_vertices_us += other.stroke_vertices_us;
        self.path_project_us += other.path_project_us;
        self.path_visibility_us += other.path_visibility_us;
        self.path_edge_sample_us += other.path_edge_sample_us;
        self.path_stitch_us += other.path_stitch_us;
        self.path_visible_edges += other.path_visible_edges;
        self.path_fragments += other.path_fragments;
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct NprPathBuildStats3d {
    project_us: f64,
    visibility_us: f64,
    edge_sample_us: f64,
    stitch_us: f64,
    visible_edges: usize,
    fragments: usize,
}

#[derive(Debug, Clone)]
struct NprFaceVisibilityBuffer {
    width: usize,
    height: usize,
    face_id: Vec<usize>,
    face_visible: Vec<bool>,
}

pub(crate) fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    light_settings: amigo_render_api::Light3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    base_color: ColorRgba,
    render_order: i32,
    shading: amigo_render_api::Material3dShadingMode,
) {
    for triangle in &geometry.triangles {
        let world = triangle
            .indices
            .map(|index| transform_point_3d(geometry.vertices[index], transform));
        let projected = world.map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        });
        let [Some(a), Some(b), Some(c)] = projected else {
            continue;
        };
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        if dot(normal, sub(camera.translation, center)) <= 0.0 {
            continue;
        }
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }
        if !projected_triangle_overlaps_viewport([a.position, b.position, c.position]) {
            continue;
        }
        let shaded = match shading {
            amigo_render_api::Material3dShadingMode::Lit => {
                let light_dir = normalize(Vec3::new(
                    -light_settings.direction.x,
                    -light_settings.direction.y,
                    -light_settings.direction.z,
                ));
                let lit = dot(normal, light_dir).max(0.0) * light_settings.intensity.max(0.0);
                let brightness: f32 = (light_settings.ambient.max(0.0) + lit).clamp(0.0, 1.25);
                multiply_color(
                    force_opaque(modulate_color(base_color, brightness)),
                    light_settings.color,
                )
            }
            amigo_render_api::Material3dShadingMode::Unlit => force_opaque(base_color),
        };
        triangles.push(ProjectedTriangle {
            points: [a.position, b.position, c.position],
            color: shaded,
            depth: (a.depth + b.depth + c.depth) / 3.0,
            render_order,
        });
    }
}

#[cfg(test)]
pub(crate) fn append_mesh_npr_line_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    let paths = build_npr_stroke_paths_for_mesh(
        viewport,
        camera,
        camera_settings,
        geometry,
        transform,
        settings,
    )
    .paths;
    append_npr_paths_as_vertices(vertices, viewport, &paths, settings);
}

pub(crate) fn append_mesh_npr_line_vertices_with_history_and_stats(
    history: &mut BTreeMap<String, NprEntityPathHistory3d>,
    frame_index: u64,
    entity_name: &str,
    vertices: &mut Vec<ColorVertex>,
    npr_stroke_segments: Option<&mut Vec<NprStrokeSegmentVertex>>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprStrokeFrameStats3d {
    let path_build_start = std::time::Instant::now();
    let path_build_result = build_npr_stroke_paths_for_mesh(
        viewport,
        camera,
        camera_settings,
        geometry,
        transform,
        settings,
    );
    let paths = path_build_result.paths;
    let path_build_us = path_build_start.elapsed().as_secs_f64() * 1_000_000.0;
    let stabilize_start = std::time::Instant::now();
    let stabilized = stabilize_npr_paths_for_entity(
        history,
        frame_index,
        entity_name,
        settings,
        paths,
    );
    let stabilize_us = stabilize_start.elapsed().as_secs_f64() * 1_000_000.0;
    let mut stats = NprStrokeFrameStats3d {
        paths: stabilized.len(),
        path_build_us,
        path_project_us: path_build_result.stats.project_us,
        path_visibility_us: path_build_result.stats.visibility_us,
        path_edge_sample_us: path_build_result.stats.edge_sample_us,
        path_stitch_us: path_build_result.stats.stitch_us,
        path_visible_edges: path_build_result.stats.visible_edges,
        path_fragments: path_build_result.stats.fragments,
        stabilize_us,
        ..NprStrokeFrameStats3d::default()
    };
    let stroke_vertices_start = std::time::Instant::now();
    let entity_history = history
        .get_mut(entity_name)
        .expect("stabilized NPR paths should initialize entity history");
    let mut npr_stroke_segments = npr_stroke_segments;
    for path in &stabilized {
        stats.record_path_kind(path.kind);
        let cached_plan = entity_history
            .paths
            .get(&path.path_id)
            .and_then(|state| state.cached_plan.as_ref());
        let used_plan = append_npr_styled_path_vertices(
            vertices,
            npr_stroke_segments.as_deref_mut(),
            viewport,
            path,
            settings,
            cached_plan,
            &mut stats,
        );
        if let Some(state) = entity_history.paths.get_mut(&path.path_id) {
            state.cached_plan = Some(used_plan);
        }
    }
    stats.stroke_vertices_us = stroke_vertices_start.elapsed().as_secs_f64() * 1_000_000.0;
    stats.meshes = 1;
    stats
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_mesh_npr_debug_overlay_vertices_with_history(
    history: &BTreeMap<String, NprEntityPathHistory3d>,
    entity_name: &str,
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
    overlay: NprDebugOverlay3d,
) {
    let mut paths = history
        .get(entity_name)
        .map(|entity| {
            entity
                .paths
                .values()
                .map(|state| state.path.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if paths.is_empty() {
        paths = build_npr_stroke_paths_for_mesh(
            viewport,
            camera,
            camera_settings,
            geometry,
            transform,
            settings,
        )
        .paths;
    }

    for path in &paths {
        append_npr_debug_path_vertices(vertices, viewport, path, settings, overlay);
    }
}

struct NprPathBuildResult3d {
    paths: Vec<NprStrokePath>,
    stats: NprPathBuildStats3d,
}

fn build_npr_stroke_paths_for_mesh(
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprPathBuildResult3d {
    if settings.passes == 0 || settings.width_px <= 0.0 {
        return NprPathBuildResult3d {
            paths: Vec::new(),
            stats: NprPathBuildStats3d::default(),
        };
    }

    let project_start = std::time::Instant::now();
    let world_vertices = geometry
        .vertices
        .iter()
        .map(|vertex| transform_point_3d(*vertex, transform))
        .collect::<Vec<_>>();
    let projected_vertices = world_vertices
        .iter()
        .map(|vertex| {
            project_point_with_camera(
                *vertex,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        })
        .collect::<Vec<_>>();
    let project_us = project_start.elapsed().as_secs_f64() * 1_000_000.0;
    let visibility_start = std::time::Instant::now();
    let mut face_front = Vec::with_capacity(geometry.triangles.len());
    let mut face_view_alignment = Vec::with_capacity(geometry.triangles.len());
    for triangle in &geometry.triangles {
        let world = triangle.indices.map(|index| world_vertices[index]);
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        let to_camera = normalize(sub(camera.translation, center));
        let view_dot = dot(normal, to_camera);
        face_front.push(view_dot > 0.0);
        face_view_alignment.push(view_dot.abs());
    }
    let visibility = build_npr_face_visibility_buffer(
        geometry,
        &projected_vertices,
        viewport,
        &face_front,
        settings.visibility_max_dimension_px,
    );
    let face_visible = face_front
        .iter()
        .enumerate()
        .map(|(index, front)| {
            *front && visibility.face_visible.get(index).copied().unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let world_normals = geometry
        .triangles
        .iter()
        .map(|triangle| transform_direction_3d(triangle.normal, transform))
        .collect::<Vec<_>>();
    let visibility_us = visibility_start.elapsed().as_secs_f64() * 1_000_000.0;

    let edge_sample_start = std::time::Instant::now();
    let edge_sample_result = collect_npr_edge_fragments_for_mesh(
        geometry,
        viewport,
        settings,
        &visibility,
        &world_vertices,
        &projected_vertices,
        &face_visible,
        &face_front,
        &face_view_alignment,
        &world_normals,
    );
    let fragments = edge_sample_result.fragments;
    let visible_edges = edge_sample_result.visible_edges;
    let edge_sample_us = edge_sample_start.elapsed().as_secs_f64() * 1_000_000.0;

    let stitch_start = std::time::Instant::now();
    let fragment_count = fragments.len();
    let paths = build_npr_stroke_paths(
        &fragments,
        viewport,
        settings.endpoint_snap_px,
        settings.path_simplify_px,
    );
    let stitch_us = stitch_start.elapsed().as_secs_f64() * 1_000_000.0;
    NprPathBuildResult3d {
        paths,
        stats: NprPathBuildStats3d {
            project_us,
            visibility_us,
            edge_sample_us,
            stitch_us,
            visible_edges,
            fragments: fragment_count,
        },
    }
}

struct NprEdgeSampleResult3d {
    fragments: Vec<NprLineFragment>,
    visible_edges: usize,
}

fn collect_npr_edge_fragments_for_mesh(
    geometry: &CachedMeshGeometry3d,
    viewport: &Viewport,
    settings: &amigo_render_api::NprLineSettings3d,
    visibility: &NprFaceVisibilityBuffer,
    world_vertices: &[Vec3],
    projected_vertices: &[Option<ProjectedPoint>],
    face_visible: &[bool],
    face_front: &[bool],
    face_view_alignment: &[f32],
    world_normals: &[Vec3],
) -> NprEdgeSampleResult3d {
    let worker_count = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .min(8);
    if worker_count <= 1 || geometry.edges.len() < 4096 {
        return collect_npr_edge_fragments_for_chunk(
            &geometry.edges,
            viewport,
            settings,
            visibility,
            world_vertices,
            projected_vertices,
            face_visible,
            face_front,
            face_view_alignment,
            world_normals,
        );
    }

    let chunk_size = geometry.edges.len().div_ceil(worker_count).max(1);
    let mut chunk_results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (chunk_index, chunk) in geometry.edges.chunks(chunk_size).enumerate() {
            handles.push(scope.spawn(move || {
                (
                    chunk_index,
                    collect_npr_edge_fragments_for_chunk(
                        chunk,
                        viewport,
                        settings,
                        visibility,
                        world_vertices,
                        projected_vertices,
                        face_visible,
                        face_front,
                        face_view_alignment,
                        world_normals,
                    ),
                )
            }));
        }
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .expect("NPR edge sampling worker should not panic")
            })
            .collect::<Vec<_>>()
    });
    chunk_results.sort_by_key(|(chunk_index, _)| *chunk_index);

    let visible_edges = chunk_results
        .iter()
        .map(|(_, result)| result.visible_edges)
        .sum();
    let fragment_count = chunk_results
        .iter()
        .map(|(_, result)| result.fragments.len())
        .sum();
    let mut fragments = Vec::with_capacity(fragment_count);
    for (_, result) in chunk_results {
        fragments.extend(result.fragments);
    }
    NprEdgeSampleResult3d {
        fragments,
        visible_edges,
    }
}

fn collect_npr_edge_fragments_for_chunk(
    edges: &[MeshEdge3d],
    viewport: &Viewport,
    settings: &amigo_render_api::NprLineSettings3d,
    visibility: &NprFaceVisibilityBuffer,
    world_vertices: &[Vec3],
    projected_vertices: &[Option<ProjectedPoint>],
    face_visible: &[bool],
    face_front: &[bool],
    face_view_alignment: &[f32],
    world_normals: &[Vec3],
) -> NprEdgeSampleResult3d {
    let mut fragments = Vec::new();
    let mut visible_edges = 0usize;

    for edge in edges {
        let f0 = edge.faces.first().copied();
        let f1 = edge.faces.get(1).copied();
        let vis0 = f0
            .and_then(|face| face_visible.get(face))
            .copied()
            .unwrap_or(false);
        let vis1 = f1
            .and_then(|face| face_visible.get(face))
            .copied()
            .unwrap_or(false);
        let front0 = f0
            .and_then(|face| face_front.get(face))
            .copied()
            .unwrap_or(false);
        let front1 = f1
            .and_then(|face| face_front.get(face))
            .copied()
            .unwrap_or(false);
        let boundary = edge.faces.len() == 1 && vis0;
        let silhouette = edge.faces.len() == 2 && front0 != front1 && (vis0 || vis1);
        let crease = edge.faces.len() == 2
            && vis0
            && vis1
            && edge_angle_degrees(world_normals[edge.faces[0]], world_normals[edge.faces[1]])
                >= settings.feature_angle_degrees.max(0.0);
        let seam = edge.faces.len() == 2 && vis0 && vis1 && edge.material_seam;
        let contact =
            (vis0 || vis1) && edge_endpoints_near_contact_ground(edge, world_vertices, settings);
        let suggestive = edge.faces.len() == 2
            && vis0
            && vis1
            && front0
            && front1
            && !crease
            && !seam
            && !silhouette
            && edge
                .faces
                .iter()
                .copied()
                .filter_map(|face| face_view_alignment.get(face).copied())
                .fold(f32::INFINITY, f32::min)
                <= 0.35;
        let Some(kind) =
            npr_line_kind_for_edge(settings, boundary, silhouette, crease, seam, suggestive, contact)
        else {
            continue;
        };
        let Some(a) = projected_vertices.get(edge.a).and_then(|point| *point) else {
            continue;
        };
        let Some(b) = projected_vertices.get(edge.b).and_then(|point| *point) else {
            continue;
        };
        if !screen_segment_is_sane(a.position, b.position) {
            continue;
        }
        let screen_length = screen_segment_length_px(a.position, b.position, viewport);
        if screen_length < settings.min_screen_length_px.max(0.0) {
            continue;
        }
        visible_edges += 1;

        fragments.extend(visible_npr_fragments_for_edge(
            visibility,
            edge,
            kind,
            a,
            b,
            viewport,
            settings.min_screen_length_px,
        ));
    }

    NprEdgeSampleResult3d {
        fragments,
        visible_edges,
    }
}

#[cfg(test)]
fn append_npr_paths_as_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    paths: &[NprStrokePath],
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprStrokeFrameStats3d {
    let mut stats = NprStrokeFrameStats3d {
        paths: paths.len(),
        ..NprStrokeFrameStats3d::default()
    };
    for path in paths {
        stats.record_path_kind(path.kind);
        let _ =
            append_npr_styled_path_vertices(vertices, None, viewport, path, settings, None, &mut stats);
    }
    stats
}

fn stabilize_npr_paths_for_entity(
    history: &mut BTreeMap<String, NprEntityPathHistory3d>,
    frame_index: u64,
    entity_name: &str,
    settings: &amigo_render_api::NprLineSettings3d,
    fresh_paths: Vec<NprStrokePath>,
) -> Vec<NprStrokePath> {
    let temporal_path_smoothing = settings.temporal_path_smoothing;
    let hysteresis = if temporal_path_smoothing {
        settings.visibility_hysteresis_frames.max(1)
    } else {
        1
    };
    let history = history.entry(entity_name.to_owned()).or_default();
    let fresh_keys = fresh_paths
        .iter()
        .map(|path| path.path_id)
        .collect::<BTreeSet<_>>();
    let mut output = Vec::with_capacity(fresh_paths.len());
    let mut consumed_previous_ids = BTreeSet::new();

    for path in fresh_paths {
        let path_id = path.path_id;
        let matched_previous_id = if history.paths.contains_key(&path_id) {
            Some(path_id)
        } else {
            best_npr_previous_path_match(&history.paths, &consumed_previous_ids, &path)
        };
        let blended = if let (true, Some(previous_id)) =
            (temporal_path_smoothing, matched_previous_id)
        {
            let previous = history
                .paths
                .get(&previous_id)
                .expect("matched NPR history key should exist");
            blend_npr_stroke_path(&previous.path, path, settings.temporal_stability)
        } else {
            path
        };
        let cached_plan = matched_previous_id
            .and_then(|previous_id| history.paths.get(&previous_id))
            .and_then(|state| state.cached_plan.clone());
        if let Some(previous_id) = matched_previous_id {
            consumed_previous_ids.insert(previous_id);
            if previous_id != path_id {
                history.paths.remove(&previous_id);
            }
        }
        history.paths.insert(
            path_id,
            NprTemporalPathState3d {
                path: blended.clone(),
                cached_plan,
                missing_frames: 0,
                last_seen_frame: frame_index,
            },
        );
        output.push(blended);
    }

    let stale_keys = history
        .paths
        .keys()
        .filter(|key| !fresh_keys.contains(*key) && !consumed_previous_ids.contains(*key))
        .copied()
        .collect::<Vec<_>>();
    for key in stale_keys {
        let mut remove = false;
        if let Some(state) = history.paths.get_mut(&key) {
            let next_missing = state.missing_frames.saturating_add(1);
            if next_missing < hysteresis {
                state.missing_frames = next_missing;
                output.push(state.path.clone());
            } else {
                remove = true;
            }
        }
        if remove {
            history.paths.remove(&key);
        }
    }

    prune_stale_npr_history(&mut history.paths, frame_index);
    let mut keyed_output = output
        .into_iter()
        .map(|path| (npr_path_average_y(&path), path))
        .collect::<Vec<_>>();
    keyed_output.sort_by(|(left_y, _), (right_y, _)| {
        left_y
            .partial_cmp(right_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    keyed_output
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>()
}

fn best_npr_previous_path_match(
    history: &BTreeMap<u64, NprTemporalPathState3d>,
    consumed_previous_ids: &BTreeSet<u64>,
    path: &NprStrokePath,
) -> Option<u64> {
    history
        .iter()
        .filter(|(path_id, _)| !consumed_previous_ids.contains(*path_id))
        .filter_map(|(path_id, state)| {
            npr_previous_path_match_score(&state.path, path)
                .filter(|score| *score <= 0.12)
                .map(|score| (*path_id, score))
        })
        .min_by(|(_, left), (_, right)| {
            left.partial_cmp(right)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(path_id, _)| path_id)
}

fn sorted_npr_source_edges(source_edges: &[u64]) -> Vec<u64> {
    let mut sorted = source_edges.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}

fn npr_source_edge_overlap_count(left: &[u64], right: &[u64]) -> usize {
    let mut left_index = 0;
    let mut right_index = 0;
    let mut overlap = 0;
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                overlap += 1;
                left_index += 1;
                right_index += 1;
            }
        }
    }
    overlap
}

fn npr_previous_path_match_score(previous: &NprStrokePath, current: &NprStrokePath) -> Option<f32> {
    if previous.kind != current.kind || previous.points.is_empty() || current.points.is_empty() {
        return None;
    }

    let endpoint_score = npr_path_endpoint_distance_score(previous, current);
    if endpoint_score > 0.48 {
        return None;
    }

    let overlap =
        npr_source_edge_overlap_count(&previous.sorted_source_edges, &current.sorted_source_edges);
    let overlap_ratio = overlap as f32
        / previous
            .source_edges
            .len()
            .max(current.source_edges.len())
            .max(1) as f32;
    if overlap_ratio > 0.0 {
        Some(endpoint_score * (1.0 - overlap_ratio * 0.75))
    } else {
        (endpoint_score <= 0.05).then_some(endpoint_score + 0.05)
    }
}

fn npr_path_endpoint_distance_score(previous: &NprStrokePath, current: &NprStrokePath) -> f32 {
    let Some(previous_start) = previous.points.first().copied() else {
        return f32::INFINITY;
    };
    let Some(previous_end) = previous.points.last().copied() else {
        return f32::INFINITY;
    };
    let Some(current_start) = current.points.first().copied() else {
        return f32::INFINITY;
    };
    let Some(current_end) = current.points.last().copied() else {
        return f32::INFINITY;
    };
    let forward = distance_vec2(previous_start, current_start)
        + distance_vec2(previous_end, current_end);
    let reversed = distance_vec2(previous_start, current_end)
        + distance_vec2(previous_end, current_start);
    forward.min(reversed) * 0.5
}

fn blend_npr_stroke_path(
    previous: &NprStrokePath,
    current: NprStrokePath,
    temporal_stability: f32,
) -> NprStrokePath {
    let stability = temporal_stability.clamp(0.0, 1.0);
    if stability <= 0.0 || previous.points.len() != current.points.len() {
        return current;
    }

    let hold = (stability * 0.55).clamp(0.0, 0.85);
    let points = previous
        .points
        .iter()
        .zip(current.points.iter())
        .map(|(prev, curr)| Vec2::new(curr.x * (1.0 - hold) + prev.x * hold, curr.y * (1.0 - hold) + prev.y * hold))
        .collect::<Vec<_>>();
    let arc_lengths_px = current.arc_lengths_px.clone();
    NprStrokePath {
        points,
        arc_lengths_px,
        importance: current.importance * (1.0 - hold) + previous.importance * hold,
        ..current
    }
}

fn prune_stale_npr_history(
    history: &mut BTreeMap<u64, NprTemporalPathState3d>,
    frame_index: u64,
) {
    let stale = history
        .iter()
        .filter_map(|(key, state)| {
            (frame_index.saturating_sub(state.last_seen_frame) > 24).then_some(key.clone())
        })
        .collect::<Vec<_>>();
    for key in stale {
        history.remove(&key);
    }
}

impl WgpuSceneRenderer {
    pub(crate) fn mesh_geometry_3d(
        &mut self,
        assets: &dyn amigo_render_api::RenderAssetSource,
        mesh_asset: &amigo_assets::AssetKey,
    ) -> Arc<CachedMeshGeometry3d> {
        let cache_key = mesh_asset.as_str().to_owned();
        if let Some(cached) = self.mesh_3d_geometry_cache.get(&cache_key) {
            return Arc::clone(cached);
        }

        let geometry = Arc::new(mesh_geometry_from_asset(assets, mesh_asset).unwrap_or_else(cube_geometry));
        self.mesh_3d_geometry_cache
            .insert(cache_key, Arc::clone(&geometry));
        geometry
    }
}

fn mesh_geometry_from_asset(
    assets: &dyn amigo_render_api::RenderAssetSource,
    mesh_asset: &amigo_assets::AssetKey,
) -> Option<CachedMeshGeometry3d> {
    let prepared = assets.prepared_asset(mesh_asset)?;
    if !matches!(prepared.kind, amigo_assets::PreparedAssetKind::Mesh3d) {
        return None;
    }
    let source_file = prepared.metadata.get("source.file")?;
    let mesh_path = prepared.resolved_path.parent()?.join(source_file);
    if prepared.format.as_deref() != Some("glb") && mesh_path.extension()?.to_str()? != "glb" {
        return None;
    }
    load_glb_geometry(&mesh_path).ok().filter(|geometry| {
        !geometry.vertices.is_empty()
            && !geometry.triangles.is_empty()
            && !geometry.edges.is_empty()
    })
}

fn load_glb_geometry(path: &std::path::Path) -> Result<CachedMeshGeometry3d, gltf::Error> {
    let (document, buffers, _) = gltf::import(path)?;
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let material_id = primitive.material().index().map(|index| index as u32);
            let reader = primitive
                .reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));
            let Some(positions) = reader.read_positions() else {
                continue;
            };
            let base_index = vertices.len();
            vertices
                .extend(positions.map(|position| Vec3::new(position[0], position[1], position[2])));
            if let Some(indices) = reader.read_indices() {
                let indices = indices.into_u32().collect::<Vec<_>>();
                for chunk in indices.chunks_exact(3) {
                    push_imported_triangle(
                        &mut triangles,
                        &vertices,
                        [
                            base_index + chunk[0] as usize,
                            base_index + chunk[1] as usize,
                            base_index + chunk[2] as usize,
                        ],
                        material_id,
                    );
                }
            } else {
                let count = vertices.len() - base_index;
                for chunk_start in (0..count).step_by(3) {
                    if chunk_start + 2 >= count {
                        break;
                    }
                    push_imported_triangle(
                        &mut triangles,
                        &vertices,
                        [
                            base_index + chunk_start,
                            base_index + chunk_start + 1,
                            base_index + chunk_start + 2,
                        ],
                        material_id,
                    );
                }
            }
        }
    }

    normalize_geometry(&mut vertices);
    rebuild_triangle_normals(&mut triangles, &vertices);
    let edges = build_edges(&triangles);
    Ok(CachedMeshGeometry3d {
        vertices,
        triangles,
        edges,
    })
}

fn push_imported_triangle(
    triangles: &mut Vec<MeshTriangle3d>,
    vertices: &[Vec3],
    indices: [usize; 3],
    material_id: Option<u32>,
) {
    let normal = normalize(cross(
        sub(vertices[indices[1]], vertices[indices[0]]),
        sub(vertices[indices[2]], vertices[indices[0]]),
    ));
    if normal == Vec3::ZERO {
        return;
    }
    triangles.push(MeshTriangle3d {
        indices,
        normal,
        material_id,
    });
}

fn normalize_geometry(vertices: &mut [Vec3]) {
    if vertices.is_empty() {
        return;
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for vertex in vertices.iter().copied() {
        min.x = min.x.min(vertex.x);
        min.y = min.y.min(vertex.y);
        min.z = min.z.min(vertex.z);
        max.x = max.x.max(vertex.x);
        max.y = max.y.max(vertex.y);
        max.z = max.z.max(vertex.z);
    }
    let center = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let extent = (max.x - min.x).max(max.y - min.y).max(max.z - min.z);
    let scale = if extent <= f32::EPSILON {
        1.0
    } else {
        1.8 / extent
    };
    for vertex in vertices {
        *vertex = Vec3::new(
            (vertex.x - center.x) * scale,
            (vertex.y - center.y) * scale,
            (vertex.z - center.z) * scale,
        );
    }
}

fn rebuild_triangle_normals(triangles: &mut [MeshTriangle3d], vertices: &[Vec3]) {
    for triangle in triangles {
        triangle.normal = normalize(cross(
            sub(vertices[triangle.indices[1]], vertices[triangle.indices[0]]),
            sub(vertices[triangle.indices[2]], vertices[triangle.indices[0]]),
        ));
    }
}

fn build_edges(triangles: &[MeshTriangle3d]) -> Vec<MeshEdge3d> {
    let mut edge_faces = BTreeMap::<(usize, usize), Vec<usize>>::new();
    for (face_index, triangle) in triangles.iter().enumerate() {
        let [a, b, c] = triangle.indices;
        for (left, right) in [(a, b), (b, c), (c, a)] {
            let key = if left <= right {
                (left, right)
            } else {
                (right, left)
            };
            edge_faces.entry(key).or_default().push(face_index);
        }
    }
    edge_faces
        .into_iter()
        .map(|((a, b), faces)| {
            let material_seam = faces.len() == 2
                && triangles[faces[0]].material_id != triangles[faces[1]].material_id;
            MeshEdge3d {
                edge_id: stable_mesh_edge_id(a, b, &faces),
                a,
                b,
                faces,
                material_seam,
            }
        })
        .collect()
}

fn cube_geometry() -> CachedMeshGeometry3d {
    let vertices = vec![
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
    ];
    let face_triangles = [
        [0usize, 2usize, 1usize],
        [0usize, 3usize, 2usize],
        [4usize, 5usize, 6usize],
        [4usize, 6usize, 7usize],
        [0usize, 1usize, 5usize],
        [0usize, 5usize, 4usize],
        [2usize, 3usize, 7usize],
        [2usize, 7usize, 6usize],
        [1usize, 2usize, 6usize],
        [1usize, 6usize, 5usize],
        [3usize, 0usize, 4usize],
        [3usize, 4usize, 7usize],
    ];
    let mut triangles = Vec::new();
    for indices in face_triangles {
        push_imported_triangle(&mut triangles, &vertices, indices, None);
    }
    let edges = build_edges(&triangles);
    CachedMeshGeometry3d {
        vertices,
        triangles,
        edges,
    }
}

fn build_npr_face_visibility_buffer(
    geometry: &CachedMeshGeometry3d,
    projected_vertices: &[Option<ProjectedPoint>],
    viewport: &Viewport,
    face_front: &[bool],
    max_visibility_dimension_px: f32,
) -> NprFaceVisibilityBuffer {
    let size = viewport.size();
    let max_dimension = size.x.max(size.y).max(1.0);
    let target_dimension = max_visibility_dimension_px.clamp(128.0, 4096.0);
    let scale = (target_dimension / max_dimension).min(1.0);
    let width = (size.x * scale).round().max(8.0) as usize;
    let height = (size.y * scale).round().max(8.0) as usize;
    let mut depth = vec![f32::INFINITY; width * height];
    let mut face_id = vec![usize::MAX; width * height];
    let mut face_visible = vec![false; geometry.triangles.len()];

    for (face_index, triangle) in geometry.triangles.iter().enumerate() {
        if !face_front.get(face_index).copied().unwrap_or(false) {
            continue;
        }
        let Some(a) = projected_vertices
            .get(triangle.indices[0])
            .and_then(|point| *point)
        else {
            continue;
        };
        let Some(b) = projected_vertices
            .get(triangle.indices[1])
            .and_then(|point| *point)
        else {
            continue;
        };
        let Some(c) = projected_vertices
            .get(triangle.indices[2])
            .and_then(|point| *point)
        else {
            continue;
        };
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }

        let a = npr_projected_point_to_buffer(a, width, height);
        let b = npr_projected_point_to_buffer(b, width, height);
        let c = npr_projected_point_to_buffer(c, width, height);
        let area = npr_edge_function(a.x, a.y, b.x, b.y, c.x, c.y);
        if area.abs() <= f32::EPSILON {
            continue;
        }

        let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
        let max_x = a.x.max(b.x).max(c.x).ceil().min((width - 1) as f32) as usize;
        let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as usize;
        let max_y = a.y.max(b.y).max(c.y).ceil().min((height - 1) as f32) as usize;
        if min_x > max_x || min_y > max_y {
            continue;
        }

        let w0_step_x = c.y - b.y;
        let w1_step_x = a.y - c.y;
        let w2_step_x = b.y - a.y;
        let w0_step_y = -(c.x - b.x);
        let w1_step_y = -(a.x - c.x);
        let w2_step_y = -(b.x - a.x);
        let start_px = min_x as f32 + 0.5;
        let start_py = min_y as f32 + 0.5;
        let row_start_w0 = npr_edge_function(b.x, b.y, c.x, c.y, start_px, start_py);
        let row_start_w1 = npr_edge_function(c.x, c.y, a.x, a.y, start_px, start_py);
        let row_start_w2 = npr_edge_function(a.x, a.y, b.x, b.y, start_px, start_py);
        let inv_area = 1.0 / area;

        for y in min_y..=max_y {
            let row_offset = (y - min_y) as f32;
            let mut w0 = row_start_w0 + row_offset * w0_step_y;
            let mut w1 = row_start_w1 + row_offset * w1_step_y;
            let mut w2 = row_start_w2 + row_offset * w2_step_y;
            for x in min_x..=max_x {
                let inside = if area >= 0.0 {
                    w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5
                } else {
                    w0 <= 1e-5 && w1 <= 1e-5 && w2 <= 1e-5
                };
                if !inside {
                    w0 += w0_step_x;
                    w1 += w1_step_x;
                    w2 += w2_step_x;
                    continue;
                }

                let l0 = w0 * inv_area;
                let l1 = w1 * inv_area;
                let l2 = w2 * inv_area;
                let sample_depth = l0 * a.z + l1 * b.z + l2 * c.z;
                let index = y * width + x;
                if sample_depth < depth[index] {
                    depth[index] = sample_depth;
                    face_id[index] = face_index;
                }
                w0 += w0_step_x;
                w1 += w1_step_x;
                w2 += w2_step_x;
            }
        }
    }

    for face in face_id.iter().copied().filter(|face| *face != usize::MAX) {
        if let Some(visible) = face_visible.get_mut(face) {
            *visible = true;
        }
    }

    NprFaceVisibilityBuffer {
        width,
        height,
        face_id,
        face_visible,
    }
}

fn npr_projected_point_to_buffer(point: ProjectedPoint, width: usize, height: usize) -> Vec3 {
    Vec3::new(
        (point.position.x * 0.5 + 0.5) * width as f32,
        (1.0 - (point.position.y * 0.5 + 0.5)) * height as f32,
        point.depth,
    )
}

fn npr_edge_function(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn visible_npr_fragments_for_edge(
    visibility: &NprFaceVisibilityBuffer,
    edge: &MeshEdge3d,
    kind: NprLineKind,
    a: ProjectedPoint,
    b: ProjectedPoint,
    viewport: &Viewport,
    min_segment_px: f32,
) -> Vec<NprLineFragment> {
    let length = screen_segment_length_px(a.position, b.position, viewport);
    if length < min_segment_px.max(0.5) {
        return Vec::new();
    }

    let samples = (length / 4.0).ceil().clamp(7.0, 96.0) as usize;
    let mut fragments = Vec::new();
    let mut run_start = None;
    let mut previous_point = None;
    let mut previous_visible = false;

    for sample in 0..=samples {
        let t = sample as f32 / samples as f32;
        let point = Vec2::new(
            a.position.x + (b.position.x - a.position.x) * t,
            a.position.y + (b.position.y - a.position.y) * t,
        );
        let depth = a.depth + (b.depth - a.depth) * t;
        let visible = npr_projected_point_in_clip(point)
            && sample_npr_owned_face(visibility, point, depth, edge);

        if visible && !previous_visible {
            run_start = Some((point, t));
        }
        if !visible && previous_visible {
            if let (Some((start, start_t)), Some((end, end_t))) = (run_start, previous_point) {
                push_visible_npr_fragment(
                    &mut fragments,
                    edge.edge_id,
                    kind,
                    start,
                    end,
                    start_t,
                    end_t,
                    a.depth + (b.depth - a.depth) * ((start_t + end_t) * 0.5),
                    viewport,
                    min_segment_px,
                );
            }
            run_start = None;
        }

        previous_visible = visible;
        previous_point = Some((point, t));
    }

    if previous_visible {
        if let (Some((start, start_t)), Some((end, end_t))) = (run_start, previous_point) {
            push_visible_npr_fragment(
                &mut fragments,
                edge.edge_id,
                kind,
                start,
                end,
                start_t,
                end_t,
                a.depth + (b.depth - a.depth) * ((start_t + end_t) * 0.5),
                viewport,
                min_segment_px,
            );
        }
    }

    fragments
}

fn push_visible_npr_fragment(
    fragments: &mut Vec<NprLineFragment>,
    source_edge_id: u64,
    kind: NprLineKind,
    start: Vec2,
    end: Vec2,
    t0: f32,
    t1: f32,
    avg_depth: f32,
    viewport: &Viewport,
    min_segment_px: f32,
) {
    if screen_segment_length_px(start, end, viewport) >= min_segment_px.max(0.5) {
        let tangent = normalize_screen_vector(start, end, viewport);
        fragments.push(NprLineFragment {
            source_edge_id,
            kind,
            p0: start,
            p1: end,
            t0,
            t1,
            tangent0: tangent,
            tangent1: tangent,
            avg_depth,
        });
    }
}

fn npr_projected_point_in_clip(point: Vec2) -> bool {
    point.x >= -1.0 && point.x <= 1.0 && point.y >= -1.0 && point.y <= 1.0
}

fn sample_npr_owned_face(
    visibility: &NprFaceVisibilityBuffer,
    point: Vec2,
    _depth: f32,
    edge: &MeshEdge3d,
) -> bool {
    let x = ((point.x * 0.5 + 0.5) * visibility.width as f32).floor() as isize;
    let y = ((1.0 - (point.y * 0.5 + 0.5)) * visibility.height as f32).floor() as isize;

    for dy in -1..=1 {
        for dx in -1..=1 {
            let sx = x + dx;
            let sy = y + dy;
            if sx < 0
                || sy < 0
                || sx >= visibility.width as isize
                || sy >= visibility.height as isize
            {
                continue;
            }
            let index = sy as usize * visibility.width + sx as usize;
            let face = visibility.face_id[index];
            if face == usize::MAX {
                continue;
            };
            if edge.faces.contains(&face) {
                return true;
            }
        }
    }

    false
}

fn npr_line_kind_for_edge(
    settings: &amigo_render_api::NprLineSettings3d,
    boundary: bool,
    silhouette: bool,
    crease: bool,
    seam: bool,
    suggestive: bool,
    contact: bool,
) -> Option<NprLineKind> {
    if contact && settings.contact {
        Some(NprLineKind::Contact)
    } else if boundary && settings.boundary {
        Some(NprLineKind::Boundary)
    } else if silhouette && settings.silhouette {
        Some(NprLineKind::Silhouette)
    } else if crease && settings.feature {
        Some(NprLineKind::Crease)
    } else if seam && settings.feature {
        Some(NprLineKind::Seam)
    } else if suggestive && settings.suggestive {
        Some(NprLineKind::Feature)
    } else {
        None
    }
}

fn edge_endpoints_near_contact_ground(
    edge: &MeshEdge3d,
    world_vertices: &[Vec3],
    settings: &amigo_render_api::NprLineSettings3d,
) -> bool {
    if !settings.contact {
        return false;
    }
    let Some(a) = world_vertices.get(edge.a).copied() else {
        return false;
    };
    let Some(b) = world_vertices.get(edge.b).copied() else {
        return false;
    };
    let threshold = settings.contact_threshold.max(0.0);
    (a.y - settings.contact_ground_y).abs() <= threshold
        && (b.y - settings.contact_ground_y).abs() <= threshold
}

#[derive(Debug, Clone, Copy)]
struct NprFragmentEndpoint {
    fragment_index: usize,
    endpoint: u8,
}

#[derive(Debug, Clone, Copy)]
struct NprPathFragment {
    fragment: NprLineFragment,
    k0: (i32, i32),
    k1: (i32, i32),
}

fn build_npr_stroke_paths(
    fragments: &[NprLineFragment],
    viewport: &Viewport,
    endpoint_quant_px: f32,
    simplify_px: f32,
) -> Vec<NprStrokePath> {
    let mut paths = Vec::new();
    for kind in [
        NprLineKind::Silhouette,
        NprLineKind::Crease,
        NprLineKind::Seam,
        NprLineKind::Feature,
        NprLineKind::Contact,
        NprLineKind::Boundary,
    ] {
        let typed = fragments
            .iter()
            .copied()
            .filter(|fragment| fragment.kind == kind)
            .collect::<Vec<_>>();
        paths.extend(build_npr_stroke_paths_for_kind(
            kind,
            &typed,
            viewport,
            endpoint_quant_px,
            simplify_px,
        ));
    }
    paths.sort_by(|left, right| {
        npr_path_average_y(left)
            .partial_cmp(&npr_path_average_y(right))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    paths
}

fn build_npr_stroke_paths_for_kind(
    kind: NprLineKind,
    fragments: &[NprLineFragment],
    viewport: &Viewport,
    endpoint_snap_px: f32,
    simplify_px: f32,
) -> Vec<NprStrokePath> {
    if fragments.is_empty() {
        return Vec::new();
    }

    let nodes = fragments
        .iter()
        .copied()
        .map(|fragment| NprPathFragment {
            fragment,
            k0: npr_point_key(fragment.p0, viewport, endpoint_snap_px),
            k1: npr_point_key(fragment.p1, viewport, endpoint_snap_px),
        })
        .collect::<Vec<_>>();
    let mut adjacency = BTreeMap::<(i32, i32), Vec<NprFragmentEndpoint>>::new();
    for (fragment_index, node) in nodes.iter().enumerate() {
        adjacency
            .entry(node.k0)
            .or_default()
            .push(NprFragmentEndpoint {
                fragment_index,
                endpoint: 0,
            });
        adjacency
            .entry(node.k1)
            .or_default()
            .push(NprFragmentEndpoint {
                fragment_index,
                endpoint: 1,
            });
    }

    let mut visited = vec![false; nodes.len()];
    let mut paths = Vec::new();

    for entries in adjacency.values() {
        if entries.len() == 2 {
            continue;
        }
        for endpoint in entries {
            if visited[endpoint.fragment_index] {
                continue;
            }
            let fragments = walk_npr_path(&nodes, &adjacency, &mut visited, *endpoint, viewport);
            push_npr_stroke_path(&mut paths, kind, fragments, viewport, simplify_px);
        }
    }

    for fragment_index in 0..nodes.len() {
        if visited[fragment_index] {
            continue;
        }
        let fragments = walk_npr_path(
            &nodes,
            &adjacency,
            &mut visited,
            NprFragmentEndpoint {
                fragment_index,
                endpoint: 0,
            },
            viewport,
        );
        push_npr_stroke_path(&mut paths, kind, fragments, viewport, simplify_px);
    }

    paths
}

fn walk_npr_path(
    nodes: &[NprPathFragment],
    adjacency: &BTreeMap<(i32, i32), Vec<NprFragmentEndpoint>>,
    visited: &mut [bool],
    start: NprFragmentEndpoint,
    viewport: &Viewport,
) -> Vec<NprLineFragment> {
    let mut path = Vec::new();
    let mut current = start;
    let mut guard = 0usize;

    while !visited[current.fragment_index] && guard < 20_000 {
        guard += 1;
        visited[current.fragment_index] = true;
        let node = nodes[current.fragment_index];
        let (fragment, next_key, entry_tangent) = if current.endpoint == 0 {
            (node.fragment, node.k1, node.fragment.tangent1)
        } else {
            (
                NprLineFragment {
                    p0: node.fragment.p1,
                    p1: node.fragment.p0,
                    t0: node.fragment.t1,
                    t1: node.fragment.t0,
                    tangent0: mul_vec2(node.fragment.tangent1, -1.0),
                    tangent1: mul_vec2(node.fragment.tangent0, -1.0),
                    ..node.fragment
                },
                node.k0,
                mul_vec2(node.fragment.tangent0, -1.0),
            )
        };
        path.push(fragment);

        let Some(entries) = adjacency.get(&next_key) else {
            break;
        };
        let Some(next) =
            best_npr_path_continuation(nodes, entries, visited, next_key, entry_tangent, viewport)
        else {
            break;
        };
        current = next;
    }

    path
}

fn push_npr_stroke_path(
    paths: &mut Vec<NprStrokePath>,
    kind: NprLineKind,
    fragments: Vec<NprLineFragment>,
    viewport: &Viewport,
    simplify_px: f32,
) {
    if fragments.is_empty() {
        return;
    }
    let mut points = Vec::with_capacity(fragments.len() + 1);
    points.push(fragments[0].p0);
    points.extend(fragments.iter().map(|fragment| fragment.p1));
    let points = simplify_npr_path(&points, viewport, simplify_px);
    if points.len() > 1 {
        let source_edges = fragments
            .iter()
            .map(|fragment| fragment.source_edge_id)
            .collect::<Vec<_>>();
        let sorted_source_edges = sorted_npr_source_edges(&source_edges);
        let path_id = stable_path_id(kind, &source_edges);
        let avg_depth = fragments.iter().map(|fragment| fragment.avg_depth).sum::<f32>()
            / fragments.len() as f32;
        paths.push(NprStrokePath {
            path_id,
            kind,
            arc_lengths_px: npr_path_arc_lengths(&points, viewport),
            importance: npr_path_importance(kind, avg_depth),
            closed: npr_path_is_closed(&points, viewport),
            points,
            source_edges,
            sorted_source_edges,
        });
    }
}

fn npr_point_key(point: Vec2, viewport: &Viewport, endpoint_quant_px: f32) -> (i32, i32) {
    let quant = endpoint_quant_px.max(0.5);
    (
        ((point.x * viewport.half_width) / quant).round() as i32,
        ((point.y * viewport.half_height) / quant).round() as i32,
    )
}

fn simplify_npr_path(points: &[Vec2], viewport: &Viewport, epsilon_px: f32) -> Vec<Vec2> {
    if epsilon_px <= 0.0 || points.len() <= 2 {
        return points.to_vec();
    }

    let mut max_distance = -1.0f32;
    let mut split_index = 0usize;
    for index in 1..points.len() - 1 {
        let distance = npr_perpendicular_distance_px(
            points[index],
            points[0],
            points[points.len() - 1],
            viewport,
        );
        if distance > max_distance {
            max_distance = distance;
            split_index = index;
        }
    }

    if max_distance > epsilon_px {
        let mut left = simplify_npr_path(&points[..=split_index], viewport, epsilon_px);
        let right = simplify_npr_path(&points[split_index..], viewport, epsilon_px);
        left.pop();
        left.extend(right);
        left
    } else {
        vec![points[0], points[points.len() - 1]]
    }
}

fn npr_perpendicular_distance_px(point: Vec2, start: Vec2, end: Vec2, viewport: &Viewport) -> f32 {
    let px = point.x * viewport.half_width;
    let py = point.y * viewport.half_height;
    let ax = start.x * viewport.half_width;
    let ay = start.y * viewport.half_height;
    let bx = end.x * viewport.half_width;
    let by = end.y * viewport.half_height;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= f32::EPSILON {
        ((px - ax).powi(2) + (py - ay).powi(2)).sqrt()
    } else {
        (dy * px - dx * py + bx * ay - by * ax).abs() / len
    }
}

fn npr_path_average_y(path: &NprStrokePath) -> f32 {
    path.points.iter().map(|point| point.y).sum::<f32>() / path.points.len() as f32
}

fn best_npr_path_continuation(
    nodes: &[NprPathFragment],
    entries: &[NprFragmentEndpoint],
    visited: &[bool],
    join_key: (i32, i32),
    entry_tangent: Vec2,
    viewport: &Viewport,
) -> Option<NprFragmentEndpoint> {
    entries
        .iter()
        .copied()
        .filter(|entry| !visited[entry.fragment_index])
        .min_by(|left, right| {
            let left_score =
                npr_path_join_score(nodes[left.fragment_index].fragment, left.endpoint, join_key, entry_tangent, viewport);
            let right_score =
                npr_path_join_score(nodes[right.fragment_index].fragment, right.endpoint, join_key, entry_tangent, viewport);
            left_score
                .partial_cmp(&right_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn npr_path_join_score(
    fragment: NprLineFragment,
    endpoint: u8,
    join_key: (i32, i32),
    entry_tangent: Vec2,
    viewport: &Viewport,
) -> f32 {
    let (start, tangent) = if endpoint == 0 {
        (fragment.p0, fragment.tangent0)
    } else {
        (fragment.p1, mul_vec2(fragment.tangent1, -1.0))
    };
    let key = npr_point_key(start, viewport, 1.0);
    let gap = ((key.0 - join_key.0).abs() + (key.1 - join_key.1).abs()) as f32;
    let tangent_mismatch = 1.0 - dot_vec2(normalize_vec2(entry_tangent), normalize_vec2(tangent));
    gap * 0.75 + tangent_mismatch * 12.0 + fragment.avg_depth.abs() * 0.025
}

fn npr_path_arc_lengths(points: &[Vec2], viewport: &Viewport) -> Vec<f32> {
    let mut result = Vec::with_capacity(points.len());
    let mut total = 0.0;
    result.push(0.0);
    for index in 1..points.len() {
        total += screen_segment_length_px(points[index - 1], points[index], viewport);
        result.push(total);
    }
    result
}

fn build_npr_stable_brush_path(path: &NprStrokePath, viewport: &Viewport) -> NprStableBrushPath {
    if path.points.len() < 2 {
        return NprStableBrushPath {
            path_id: path.path_id,
            samples: path
                .points
                .iter()
                .copied()
                .map(|point| NprBrushSample {
                    point,
                    arc_length_px: 0.0,
                })
                .collect(),
            length_px: 0.0,
        };
    }

    let estimated_samples = path
        .arc_lengths_px
        .last()
        .copied()
        .map(|length| (length / NPR_BRUSH_RESAMPLE_SPACING_PX).ceil() as usize + 1)
        .unwrap_or(path.points.len())
        .max(path.points.len());
    let mut samples = Vec::with_capacity(estimated_samples);
    let mut total = 0.0;
    samples.push(NprBrushSample {
        point: path.points[0],
        arc_length_px: 0.0,
    });

    for index in 1..path.points.len() {
        let start = path.points[index - 1];
        let end = path.points[index];
        let segment_length = screen_segment_length_px(start, end, viewport);
        if segment_length <= f32::EPSILON {
            continue;
        }

        let steps = (segment_length / NPR_BRUSH_RESAMPLE_SPACING_PX).floor() as usize;
        for step in 1..=steps {
            let local_t = (step as f32 * NPR_BRUSH_RESAMPLE_SPACING_PX / segment_length)
                .clamp(0.0, 1.0);
            if local_t >= 1.0 {
                continue;
            }
            samples.push(NprBrushSample {
                point: Vec2::new(
                    start.x + (end.x - start.x) * local_t,
                    start.y + (end.y - start.y) * local_t,
                ),
                arc_length_px: total + segment_length * local_t,
            });
        }

        total += segment_length;
        samples.push(NprBrushSample {
            point: end,
            arc_length_px: total,
        });
    }

    NprStableBrushPath {
        path_id: path.path_id,
        samples,
        length_px: total.max(1.0),
    }
}

fn npr_path_is_closed(points: &[Vec2], viewport: &Viewport) -> bool {
    if points.len() < 3 {
        return false;
    }
    screen_segment_length_px(points[0], points[points.len() - 1], viewport) <= 3.0
}

fn npr_path_importance(kind: NprLineKind, avg_depth: f32) -> f32 {
    let depth_factor = (1.18 - avg_depth.abs() * 0.08).clamp(0.72, 1.18);
    let kind_factor = match kind {
        NprLineKind::Silhouette => 1.08,
        NprLineKind::Boundary => 0.96,
        NprLineKind::Crease => 0.88,
        NprLineKind::Seam => 0.82,
        NprLineKind::Feature => 0.88,
        NprLineKind::Contact => 0.92,
    };
    depth_factor * kind_factor
}

fn npr_distance_width_multiplier(
    importance: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    crate::renderer::npr_distance_width_multiplier(importance, settings)
}

fn npr_depth_alpha_multiplier(
    importance: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    crate::renderer::npr_depth_alpha_multiplier(importance, settings)
}

fn npr_pressure_multiplier(
    t: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    crate::renderer::npr_pressure_multiplier(t, settings)
}

fn npr_alpha_pressure_multiplier(
    t: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    crate::renderer::npr_alpha_pressure_multiplier(t, settings)
}

fn npr_straightness_wobble_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    crate::renderer::npr_straightness_wobble_multiplier(settings)
}

fn npr_tool_width_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    crate::renderer::npr_tool_width_multiplier(settings)
}

fn npr_tool_alpha_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    crate::renderer::npr_tool_alpha_multiplier(settings)
}

fn npr_tool_pressure_jitter_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    crate::renderer::npr_tool_pressure_jitter_multiplier(settings)
}

fn npr_tool_dropout_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    crate::renderer::npr_tool_dropout_multiplier(settings)
}

fn npr_tool_search_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    crate::renderer::npr_tool_search_multiplier(settings)
}

fn npr_endpoint_lock(
    t: f32,
    path_length_px: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let start_t = (settings.endpoint_lock_start_px.max(0.0) / path_length_px.max(1.0))
        .clamp(0.0, 0.45);
    let end_t = (settings.endpoint_lock_end_px.max(0.0) / path_length_px.max(1.0))
        .clamp(0.0, 0.45);
    if start_t > 0.0 && t <= start_t {
        (t / start_t).clamp(0.0, 1.0)
    } else if end_t > 0.0 && t >= 1.0 - end_t {
        ((1.0 - t) / end_t).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn npr_pass_offset_px(
    path_seed: u64,
    arc_t: f32,
    settings: &amigo_render_api::NprLineSettings3d,
    pass: u8,
) -> f32 {
    if settings.pass_offset_px <= 0.0 {
        return 0.0;
    }
    coherent_signed_noise_1d(
        settings.seed,
        path_seed,
        pass as u64,
        arc_t * 12.0 + pass as f32,
        631,
    ) * settings.pass_offset_px
}

fn coherent_signed_noise_1d(seed: u64, edge: u64, pass: u64, position: f32, salt: u64) -> f32 {
    let base = position.floor();
    let frac = (position - base).clamp(0.0, 1.0);
    let smooth = frac * frac * (3.0 - 2.0 * frac);
    let left = deterministic_signed_noise(seed, edge, pass, salt.wrapping_add(base as u64));
    let right = deterministic_signed_noise(
        seed,
        edge,
        pass,
        salt.wrapping_add(base as u64).wrapping_add(1),
    );
    left + (right - left) * smooth
}

fn build_npr_stroke_gesture(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprStrokeGesture {
    let style = resolve_npr_kind_style(path.kind, settings);
    let importance = path.importance.clamp(0.55, 1.35);
    let dynamics = NprToolDynamics {
        base_width_px: settings.width_px
            * style.width_multiplier
            * importance
            * npr_tool_width_multiplier(settings),
        base_wobble_px: style.wobble_px
            * settings.humanization
            * npr_straightness_wobble_multiplier(settings),
        effective_overshoot_px: if path.closed { 0.0 } else { style.overshoot_px },
        edge_complexity: path.source_edges.len().max(1) as f32,
        protected_silhouette: path.kind == NprLineKind::Silhouette && path.importance >= 0.9,
    };

    NprStrokeGesture {
        path_seed: path.path_id,
        path_length_px: path.arc_lengths_px.last().copied().unwrap_or(0.0).max(1.0),
        importance,
        dynamics,
        style,
    }
}

fn build_npr_stroke_pass_plan(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
) -> Vec<NprStrokePassPlan> {
    let primary_passes = settings.passes.min(8);
    let mut passes = Vec::with_capacity(
        primary_passes as usize + settings.search_line_count as usize,
    );

    for pass in 0..primary_passes {
        passes.push(NprStrokePassPlan {
            kind: NprStrokePassKind::Primary,
            pass_index: pass,
            wobble_px: gesture.dynamics.base_wobble_px
                * npr_pass_jitter_multiplier(primary_passes, pass),
            width_multiplier: npr_pass_width_multiplier(primary_passes, pass),
            color: npr_pass_color(
                settings.ink_color,
                primary_passes,
                pass,
                gesture.style.alpha_multiplier,
            ),
            overshoot_px: gesture.dynamics.effective_overshoot_px,
        });
    }

    let search_count = if path.kind == NprLineKind::Silhouette {
        0
    } else {
        ((settings.search_line_count as f32) * npr_tool_search_multiplier(settings))
            .round()
            .clamp(0.0, 8.0) as u8
    };
    for search_pass in 0..search_count {
        passes.push(NprStrokePassPlan {
            kind: NprStrokePassKind::Search,
            pass_index: primary_passes.saturating_add(search_pass),
            wobble_px: gesture.dynamics.base_wobble_px * 1.18,
            width_multiplier: 0.78,
            color: ColorRgba::new(
                settings.ink_color.r,
                settings.ink_color.g,
                settings.ink_color.b,
                (settings.ink_color.a
                    * settings.search_line_alpha
                    * npr_tool_alpha_multiplier(settings))
                .clamp(0.0, 1.0),
            ),
            overshoot_px: gesture
                .dynamics
                .effective_overshoot_px
                .max(settings.undershoot_px),
        });
    }

    passes
}

fn build_npr_dropout_mask(
    gesture: NprStrokeGesture,
    settings: &amigo_render_api::NprLineSettings3d,
    passes: &[NprStrokePassPlan],
) -> NprDropoutMask {
    let mut intervals = Vec::new();
    if !gesture.dynamics.protected_silhouette && gesture.style.dropout > 0.0 {
        let complexity_multiplier =
            (1.0 - (gesture.dynamics.edge_complexity.min(12.0) - 1.0) * 0.01).max(0.0);
        let effective_dropout = (gesture.style.dropout
            * npr_tool_dropout_multiplier(settings)
            * complexity_multiplier)
            .clamp(0.0, 0.85);
        let path_length = gesture.path_length_px.max(1.0);
        let interval_count = (effective_dropout * path_length / 64.0).ceil() as usize;
        let interval_count = interval_count.min(8);
        let min_gap_t = (settings.dropout_segment_min_px.max(1.0) / path_length).clamp(0.01, 0.25);

        for pass in passes
            .iter()
            .copied()
            .filter(|pass| pass.kind == NprStrokePassKind::Primary)
        {
            for interval_index in 0..interval_count {
                let center = deterministic_noise(
                    settings.seed,
                    gesture.path_seed,
                    pass.pass_index as u64,
                    751 + interval_index as u64,
                );
                let width = (min_gap_t
                    + deterministic_noise(
                        settings.seed,
                        gesture.path_seed,
                        pass.pass_index as u64,
                        811 + interval_index as u64,
                    ) * min_gap_t)
                    .clamp(0.01, 0.25);
                let t0 = (center - width * 0.5).clamp(0.08, 0.92);
                let t1 = (center + width * 0.5).clamp(0.08, 0.92);
                if t1 > t0 {
                    intervals.push(NprDropoutInterval {
                        pass_index: pass.pass_index,
                        t0,
                        t1,
                    });
                }
            }
        }
    }

    NprDropoutMask { intervals }
}

fn build_npr_cached_stroke_plan(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
) -> NprCachedStrokePlan {
    let passes = build_npr_stroke_pass_plan(path, settings, gesture);
    let dropout = build_npr_dropout_mask(gesture, settings, &passes);
    NprCachedStrokePlan {
        settings_signature: npr_stroke_plan_settings_signature(settings),
        length_bucket_px: npr_stroke_plan_length_bucket(gesture.path_length_px),
        passes,
        dropout,
    }
}

fn build_empty_npr_cached_stroke_plan(
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprCachedStrokePlan {
    NprCachedStrokePlan {
        settings_signature: npr_stroke_plan_settings_signature(settings),
        length_bucket_px: 0,
        passes: Vec::new(),
        dropout: NprDropoutMask {
            intervals: Vec::new(),
        },
    }
}

fn npr_stroke_plan_length_bucket(length_px: f32) -> u32 {
    (length_px.max(0.0) / 8.0).round() as u32
}

fn npr_stroke_plan_settings_signature(settings: &amigo_render_api::NprLineSettings3d) -> u64 {
    let mut hash = 0x9E37_79B9_7F4A_7C15_u64;
    hash = mix_u64(hash, settings.stroke_tool as u64);
    hash = mix_u64(hash, settings.suggestive as u64);
    hash = mix_u64(hash, settings.contact as u64);
    hash = mix_u64(hash, settings.contact_ground_y.to_bits() as u64);
    hash = mix_u64(hash, settings.contact_threshold.to_bits() as u64);
    hash = mix_u64(hash, settings.passes as u64);
    hash = mix_u64(hash, settings.search_line_count as u64);
    hash = mix_u64(hash, settings.search_line_alpha.to_bits() as u64);
    hash = mix_u64(hash, settings.dropout.to_bits() as u64);
    hash = mix_u64(hash, settings.dropout_segment_min_px.to_bits() as u64);
    hash = mix_u64(hash, settings.tool_dropout_multiplier.to_bits() as u64);
    hash = mix_u64(hash, settings.tool_search_multiplier.to_bits() as u64);
    hash = mix_u64(hash, settings.tool_alpha_multiplier.to_bits() as u64);
    hash = mix_u64(hash, settings.ink_color.a.to_bits() as u64);
    hash = mix_u64(hash, settings.seed);
    hash
}

fn mix_u64(current: u64, value: u64) -> u64 {
    current
        .wrapping_mul(0x100_0000_01B3)
        .wrapping_add(value ^ 0xA53A_9E37_1337_5EED)
}

impl NprDropoutMask {
    fn keeps_segment(
        &self,
        pass: NprStrokePassPlan,
        segment_t0: f32,
        segment_t1: f32,
        segment_length_px: f32,
    ) -> bool {
        if pass.kind == NprStrokePassKind::Search || segment_length_px <= f32::EPSILON {
            return true;
        }
        !self.intervals.iter().any(|interval| {
            interval.pass_index == pass.pass_index
                && segment_t1 >= interval.t0
                && segment_t0 <= interval.t1
        })
    }
}

fn append_npr_styled_path_vertices(
    vertices: &mut Vec<ColorVertex>,
    mut npr_stroke_segments: Option<&mut Vec<NprStrokeSegmentVertex>>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    cached_plan: Option<&NprCachedStrokePlan>,
    stats: &mut NprStrokeFrameStats3d,
) -> NprCachedStrokePlan {
    if path.points.len() < 2 {
        return build_empty_npr_cached_stroke_plan(settings);
    }

    let gesture = build_npr_stroke_gesture(path, settings);
    let brush_path = build_npr_stable_brush_path(path, viewport);
    if brush_path.samples.len() < 2 {
        return build_empty_npr_cached_stroke_plan(settings);
    }
    stats.brush_samples += brush_path.samples.len();
    let plan = if let Some(plan) = cached_plan.filter(|plan| plan.is_compatible(settings, gesture)) {
        stats.cached_plan_hits += 1;
        plan.clone()
    } else {
        stats.cached_plan_misses += 1;
        build_npr_cached_stroke_plan(path, settings, gesture)
    };
    stats.dropout_intervals += plan.dropout.intervals.len();

    for pass in plan.passes.iter().copied() {
        stats.record_pass(pass);
        let mut strip_samples = Vec::with_capacity(brush_path.samples.len());
        for point_index in 1..brush_path.samples.len() {
            let segment_length = screen_segment_length_px(
                brush_path.samples[point_index - 1].point,
                brush_path.samples[point_index].point,
                viewport,
            );
            let segment_t0 = brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
            let segment_t1 = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
            if !plan
                .dropout
                .keeps_segment(pass, segment_t0, segment_t1, segment_length)
            {
                let before_vertices = vertices.len();
                if let Some(segments) = npr_stroke_segments.as_deref_mut() {
                    append_npr_stroke_strip_segments(segments, viewport, &strip_samples);
                    stats.strip_vertices += strip_samples.len().saturating_sub(1) * 6;
                } else {
                    append_npr_stroke_strip_vertices(vertices, viewport, &strip_samples);
                    stats.strip_vertices += vertices.len().saturating_sub(before_vertices);
                }
                strip_samples.clear();
                continue;
            }

            if strip_samples.is_empty() {
                let distance_t =
                    brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
                strip_samples.push(npr_stroke_strip_sample(
                    &brush_path,
                    point_index - 1,
                    settings,
                    gesture,
                    pass,
                    distance_t,
                    viewport,
                ));
            }
            let distance_t = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
            strip_samples.push(npr_stroke_strip_sample(
                &brush_path,
                point_index,
                settings,
                gesture,
                pass,
                distance_t,
                viewport,
            ));
        }
        let before_vertices = vertices.len();
        if let Some(segments) = npr_stroke_segments.as_deref_mut() {
            append_npr_stroke_strip_segments(segments, viewport, &strip_samples);
            stats.strip_vertices += strip_samples.len().saturating_sub(1) * 6;
        } else {
            append_npr_stroke_strip_vertices(vertices, viewport, &strip_samples);
            stats.strip_vertices += vertices.len().saturating_sub(before_vertices);
        }
    }
    plan
}

fn append_npr_debug_path_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    overlay: NprDebugOverlay3d,
) {
    match overlay {
        NprDebugOverlay3d::LineKinds => {
            append_npr_debug_polyline(vertices, viewport, &path.points, npr_line_kind_debug_color(path.kind), 2.0);
        }
        NprDebugOverlay3d::RawPaths => {
            append_npr_debug_polyline(vertices, viewport, &path.points, npr_path_id_debug_color(path.path_id), 1.5);
        }
        NprDebugOverlay3d::Dropout => {
            append_npr_dropout_debug_vertices(vertices, viewport, path, settings);
        }
        NprDebugOverlay3d::WidthAlpha => {
            append_npr_width_alpha_debug_vertices(vertices, viewport, path, settings);
        }
    }
}

fn append_npr_dropout_debug_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    if path.points.len() < 2 {
        return;
    }

    let gesture = build_npr_stroke_gesture(path, settings);
    let brush_path = build_npr_stable_brush_path(path, viewport);
    let passes = build_npr_stroke_pass_plan(path, settings, gesture);
    let dropout = build_npr_dropout_mask(gesture, settings, &passes);
    let Some(primary) = passes
        .iter()
        .copied()
        .find(|pass| pass.kind == NprStrokePassKind::Primary)
    else {
        return;
    };

    for point_index in 1..brush_path.samples.len() {
        let segment_t0 = brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
        let segment_t1 = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
        let segment_length = screen_segment_length_px(
            brush_path.samples[point_index - 1].point,
            brush_path.samples[point_index].point,
            viewport,
        );
        if dropout.keeps_segment(primary, segment_t0, segment_t1, segment_length) {
            continue;
        }
        append_npr_debug_segment(
            vertices,
            viewport,
            brush_path.samples[point_index - 1].point,
            brush_path.samples[point_index].point,
            ColorRgba::new(1.0, 0.12, 0.05, 0.95),
            4.0,
        );
    }
}

fn append_npr_width_alpha_debug_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    if path.points.len() < 2 {
        return;
    }

    let gesture = build_npr_stroke_gesture(path, settings);
    let brush_path = build_npr_stable_brush_path(path, viewport);
    let Some(primary) = build_npr_stroke_pass_plan(path, settings, gesture)
        .into_iter()
        .find(|pass| pass.kind == NprStrokePassKind::Primary)
    else {
        return;
    };

    for point_index in 1..brush_path.samples.len() {
        let t0 = brush_path.samples[point_index - 1].arc_length_px / brush_path.length_px;
        let t1 = brush_path.samples[point_index].arc_length_px / brush_path.length_px;
        let sample0 = npr_stroke_strip_sample(
            &brush_path,
            point_index - 1,
            settings,
            gesture,
            primary,
            t0,
            viewport,
        );
        let sample1 = npr_stroke_strip_sample(
            &brush_path,
            point_index,
            settings,
            gesture,
            primary,
            t1,
            viewport,
        );
        let width01 = ((sample0.width_px + sample1.width_px) * 0.5 / 8.0).clamp(0.0, 1.0);
        let alpha01 = ((sample0.color.a + sample1.color.a) * 0.5).clamp(0.0, 1.0);
        append_npr_debug_segment(
            vertices,
            viewport,
            sample0.point,
            sample1.point,
            ColorRgba::new(width01, alpha01, 1.0 - width01, 0.9),
            3.0,
        );
    }
}

fn append_npr_debug_polyline(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    points: &[Vec2],
    color: ColorRgba,
    width_px: f32,
) {
    for window in points.windows(2) {
        append_npr_debug_segment(vertices, viewport, window[0], window[1], color, width_px);
    }
}

fn append_npr_debug_segment(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    a: Vec2,
    b: Vec2,
    color: ColorRgba,
    width_px: f32,
) {
    let delta_px = Vec2::new(
        (b.x - a.x) * viewport.half_width,
        (b.y - a.y) * viewport.half_height,
    );
    let length = (delta_px.x * delta_px.x + delta_px.y * delta_px.y).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let normal_px = Vec2::new(-delta_px.y / length, delta_px.x / length);
    let half = width_px.max(0.5) * 0.5;
    let offset = Vec2::new(
        normal_px.x * half / viewport.half_width,
        normal_px.y * half / viewport.half_height,
    );
    push_quad(
        vertices,
        Vec2::new(a.x + offset.x, a.y + offset.y),
        Vec2::new(b.x + offset.x, b.y + offset.y),
        Vec2::new(b.x - offset.x, b.y - offset.y),
        Vec2::new(a.x - offset.x, a.y - offset.y),
        color,
    );
}

fn npr_line_kind_debug_color(kind: NprLineKind) -> ColorRgba {
    match kind {
        NprLineKind::Boundary => ColorRgba::new(0.15, 0.45, 1.0, 0.95),
        NprLineKind::Silhouette => ColorRgba::new(1.0, 0.88, 0.1, 0.95),
        NprLineKind::Crease => ColorRgba::new(1.0, 0.22, 0.12, 0.9),
        NprLineKind::Seam => ColorRgba::new(0.2, 1.0, 0.45, 0.9),
        NprLineKind::Feature => ColorRgba::new(0.85, 0.35, 1.0, 0.9),
        NprLineKind::Contact => ColorRgba::new(0.05, 0.05, 0.05, 0.95),
    }
}

fn npr_path_id_debug_color(path_id: u64) -> ColorRgba {
    let r = deterministic_noise(path_id, 11, 0, 0);
    let g = deterministic_noise(path_id, 23, 0, 0);
    let b = deterministic_noise(path_id, 37, 0, 0);
    ColorRgba::new(0.25 + r * 0.75, 0.25 + g * 0.75, 0.25 + b * 0.75, 0.88)
}

fn npr_stroke_strip_sample(
    brush_path: &NprStableBrushPath,
    point_index: usize,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
    pass: NprStrokePassPlan,
    distance_t: f32,
    viewport: &Viewport,
) -> NprStrokeStripSample {
    let point = humanize_npr_brush_sample(
        brush_path,
        point_index,
        settings,
        pass.pass_index,
        pass.wobble_px,
        viewport,
    );
    let width_noise = coherent_signed_noise_1d(
        settings.seed,
        gesture.path_seed,
        pass.pass_index as u64,
        distance_t * settings.stroke_wobble_frequency.max(0.01) * 100.0 + 7.0,
        503,
    );
    let width_px = (gesture.dynamics.base_width_px
        * pass.width_multiplier
        * npr_pressure_multiplier(distance_t, settings)
        * npr_taper_multiplier(distance_t, gesture.style.taper)
        * npr_distance_width_multiplier(gesture.importance, settings)
        + width_noise
            * settings.pressure_jitter
            * npr_tool_pressure_jitter_multiplier(settings))
        .max(0.25);
    let pass_offset = npr_pass_offset_px(brush_path.path_id, distance_t, settings, pass.pass_index);
    let color = ColorRgba::new(
        pass.color.r,
        pass.color.g,
        pass.color.b,
        (pass.color.a
            * npr_alpha_pressure_multiplier(distance_t, settings)
            * npr_tool_alpha_multiplier(settings)
            * npr_depth_alpha_multiplier(gesture.importance, settings))
            .clamp(0.0, 1.0),
    );

    NprStrokeStripSample {
        point,
        width_px,
        offset_px: pass_offset,
        overshoot_px: pass.overshoot_px,
        color,
    }
}

fn append_npr_stroke_strip_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    samples: &[NprStrokeStripSample],
) {
    if samples.len() < 2 {
        return;
    }

    let mut previous_rail = npr_stroke_rail(samples, viewport, 0);
    for index in 1..samples.len() {
        let Some(a) = previous_rail else {
            previous_rail = npr_stroke_rail(samples, viewport, index);
            continue;
        };
        let Some(b) = npr_stroke_rail(samples, viewport, index) else {
            continue;
        };
        let color = ColorRgba::new(
            (a.color.r + b.color.r) * 0.5,
            (a.color.g + b.color.g) * 0.5,
            (a.color.b + b.color.b) * 0.5,
            (a.color.a + b.color.a) * 0.5,
        );
        vertices.push(ColorVertex::new(a.left, color));
        vertices.push(ColorVertex::new(b.left, color));
        vertices.push(ColorVertex::new(b.right, color));
        vertices.push(ColorVertex::new(a.left, color));
        vertices.push(ColorVertex::new(b.right, color));
        vertices.push(ColorVertex::new(a.right, color));
        previous_rail = Some(b);
    }
}

fn append_npr_stroke_strip_segments(
    segments: &mut Vec<NprStrokeSegmentVertex>,
    viewport: &Viewport,
    samples: &[NprStrokeStripSample],
) {
    if samples.len() < 2 {
        return;
    }

    for index in 1..samples.len() {
        let start = samples[index - 1];
        let end = samples[index];
        let dx = (end.point.x - start.point.x) * viewport.half_width;
        let dy = (end.point.y - start.point.y) * viewport.half_height;
        if dx * dx + dy * dy <= f32::EPSILON {
            continue;
        }
        let color = ColorRgba::new(
            (start.color.r + end.color.r) * 0.5,
            (start.color.g + end.color.g) * 0.5,
            (start.color.b + end.color.b) * 0.5,
            (start.color.a + end.color.a) * 0.5,
        );
        segments.push(NprStrokeSegmentVertex {
            start: [
                start.point.x * viewport.half_width,
                start.point.y * viewport.half_height,
            ],
            end: [
                end.point.x * viewport.half_width,
                end.point.y * viewport.half_height,
            ],
            color: [color.r, color.g, color.b, color.a],
            width_px: (start.width_px + end.width_px) * 0.5,
            offset_px: (start.offset_px + end.offset_px) * 0.5,
            overshoot_start_px: if index == 1 { start.overshoot_px } else { 0.0 },
            overshoot_end_px: if index + 1 == samples.len() {
                end.overshoot_px
            } else {
                0.0
            },
            viewport_half: [viewport.half_width, viewport.half_height],
            end_width_px: end.width_px,
            end_alpha: end.color.a,
        });
    }
}

fn npr_stroke_rail(
    samples: &[NprStrokeStripSample],
    viewport: &Viewport,
    index: usize,
) -> Option<NprStrokeRail> {
    let sample = samples.get(index)?;
    let previous = samples[index.saturating_sub(1)].point;
    let next = samples[(index + 1).min(samples.len() - 1)].point;
    let tangent_px = Vec2::new(
        (next.x - previous.x) * viewport.half_width,
        (next.y - previous.y) * viewport.half_height,
    );
    let length = (tangent_px.x * tangent_px.x + tangent_px.y * tangent_px.y).sqrt();
    if length <= f32::EPSILON {
        return None;
    }

    let direction = Vec2::new(tangent_px.x / length, tangent_px.y / length);
    let normal = Vec2::new(-direction.y, direction.x);
    let endpoint_sign = if index == 0 {
        -1.0
    } else if index + 1 == samples.len() {
        1.0
    } else {
        0.0
    };
    let center_px = Vec2::new(
        sample.point.x * viewport.half_width
            + direction.x * sample.overshoot_px * endpoint_sign
            + normal.x * sample.offset_px,
        sample.point.y * viewport.half_height
            + direction.y * sample.overshoot_px * endpoint_sign
            + normal.y * sample.offset_px,
    );
    let half_width = sample.width_px * 0.5;
    let left = Vec2::new(
        (center_px.x + normal.x * half_width) / viewport.half_width,
        (center_px.y + normal.y * half_width) / viewport.half_height,
    );
    let right = Vec2::new(
        (center_px.x - normal.x * half_width) / viewport.half_width,
        (center_px.y - normal.y * half_width) / viewport.half_height,
    );
    Some(NprStrokeRail {
        left,
        right,
        color: sample.color,
    })
}

fn humanize_npr_brush_sample(
    brush_path: &NprStableBrushPath,
    index: usize,
    settings: &amigo_render_api::NprLineSettings3d,
    pass: u8,
    wobble_px: f32,
    viewport: &Viewport,
) -> Vec2 {
    let micro_wobble_px = settings.micro_wobble_px
        * settings.humanization
        * npr_straightness_wobble_multiplier(settings);
    if wobble_px <= 0.0 && micro_wobble_px <= 0.0 {
        return brush_path.samples[index].point;
    }

    let prev = brush_path.samples[index.saturating_sub(1)].point;
    let next = brush_path.samples[(index + 1).min(brush_path.samples.len() - 1)].point;
    let tx = (next.x - prev.x) * viewport.half_width;
    let ty = (next.y - prev.y) * viewport.half_height;
    let length = (tx * tx + ty * ty).sqrt();
    if length <= f32::EPSILON {
        return brush_path.samples[index].point;
    }

    let normal = Vec2::new(-ty / length, tx / length);
    let point = brush_path.samples[index].point;
    let arc_t = brush_path.samples[index].arc_length_px / brush_path.length_px;
    let endpoint_lock = npr_endpoint_lock(arc_t, brush_path.length_px, settings);
    let primary = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.stroke_wobble_frequency.max(0.01) * 100.0,
        919,
    );
    let drift = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.stroke_wobble_frequency.max(0.01) * 37.0 + 3.7,
        977,
    );
    let micro = coherent_signed_noise_1d(
        settings.seed,
        brush_path.path_id,
        pass as u64,
        arc_t * settings.micro_wobble_frequency.max(0.01) * 100.0 + 13.0,
        991,
    );
    let tangent_scale = settings.local_angular_drift_degrees.to_radians().sin() * settings.humanization;
    let px = point.x * viewport.half_width
        + normal.x * primary * wobble_px * endpoint_lock
        + normal.x * micro * micro_wobble_px * endpoint_lock
        + (tx / length) * drift * wobble_px * tangent_scale * endpoint_lock;
    let py = point.y * viewport.half_height
        + normal.y * primary * wobble_px * endpoint_lock
        + normal.y * micro * micro_wobble_px * endpoint_lock
        + (ty / length) * drift * wobble_px * tangent_scale * endpoint_lock;
    Vec2::new(px / viewport.half_width, py / viewport.half_height)
}

fn npr_pass_jitter_multiplier(passes: u8, pass: u8) -> f32 {
    if passes >= 3 {
        1.0 + pass as f32 * 0.55
    } else if passes == 2 {
        if pass == 0 { 1.1 } else { 0.35 }
    } else {
        0.35
    }
}

fn npr_pass_width_multiplier(passes: u8, pass: u8) -> f32 {
    if passes >= 3 {
        0.9
    } else if passes == 2 {
        if pass == 0 { 1.6 } else { 0.85 }
    } else {
        0.75
    }
}

fn npr_pass_color(color: ColorRgba, passes: u8, pass: u8, alpha_multiplier: f32) -> ColorRgba {
    let alpha = if passes >= 3 {
        0.18
    } else if passes == 2 {
        if pass == 0 { 0.28 } else { 0.75 }
    } else {
        0.92
    };
    ColorRgba::new(
        color.r,
        color.g,
        color.b,
        (color.a * alpha * alpha_multiplier).clamp(0.0, 1.0),
    )
}

fn npr_taper_multiplier(t: f32, taper: f32) -> f32 {
    let endpoint_weight = (t.min(1.0 - t) * 2.0).clamp(0.0, 1.0);
    1.0 - taper.clamp(0.0, 1.0) * (1.0 - endpoint_weight.max(0.35))
}

fn edge_angle_degrees(left: Vec3, right: Vec3) -> f32 {
    dot(normalize(left), normalize(right))
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

fn screen_segment_length_px(start: Vec2, end: Vec2, viewport: &Viewport) -> f32 {
    let dx = (end.x - start.x) * viewport.half_width;
    let dy = (end.y - start.y) * viewport.half_height;
    (dx * dx + dy * dy).sqrt()
}

fn screen_segment_is_sane(start: Vec2, end: Vec2) -> bool {
    [start, end].iter().all(|point| {
        point.x.is_finite() && point.y.is_finite() && point.x.abs() < 8.0 && point.y.abs() < 8.0
    })
}

fn triangle_center(points: [Vec3; 3]) -> Vec3 {
    Vec3::new(
        (points[0].x + points[1].x + points[2].x) / 3.0,
        (points[0].y + points[1].y + points[2].y) / 3.0,
        (points[0].z + points[1].z + points[2].z) / 3.0,
    )
}

fn projected_triangle_is_sane(points: [Vec2; 3]) -> bool {
    points.iter().all(|point| {
        point.x.is_finite() && point.y.is_finite() && point.x.abs() < 8.0 && point.y.abs() < 8.0
    })
}

fn projected_triangle_overlaps_viewport(points: [Vec2; 3]) -> bool {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    max_x >= -1.0 && min_x <= 1.0 && max_y >= -1.0 && min_y <= 1.0
}

fn force_opaque(color: ColorRgba) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, 1.0)
}

fn transform_direction_3d(point: Vec3, transform: Transform3) -> Vec3 {
    let scaled = Vec3::new(
        point.x * transform.scale.x.signum(),
        point.y * transform.scale.y.signum(),
        point.z * transform.scale.z.signum(),
    );
    let rotated_x = rotate_x(scaled, transform.rotation_euler.x);
    let rotated_y = rotate_y(rotated_x, transform.rotation_euler.y);
    normalize(rotate_z(rotated_y, transform.rotation_euler.z))
}

fn stable_mesh_edge_id(a: usize, b: usize, faces: &[usize]) -> u64 {
    let first_face = faces.first().copied().unwrap_or_default() as u64;
    let second_face = faces.get(1).copied().unwrap_or_default() as u64;
    ((a as u64) << 32)
        ^ (b as u64)
        ^ first_face.wrapping_mul(0x9E37_79B9)
        ^ second_face.wrapping_mul(0x85EB_CA77)
}

fn stable_path_id(kind: NprLineKind, source_edges: &[u64]) -> u64 {
    let mut value = match kind {
        NprLineKind::Boundary => 11u64,
        NprLineKind::Silhouette => 17u64,
        NprLineKind::Crease => 19u64,
        NprLineKind::Seam => 29u64,
        NprLineKind::Feature => 23u64,
        NprLineKind::Contact => 31u64,
    };

    let reversed = source_edges.iter().rev().copied().collect::<Vec<_>>();
    let canonical_edges = if reversed.as_slice() < source_edges {
        reversed.as_slice()
    } else {
        source_edges
    };
    for edge in canonical_edges {
        value = value.rotate_left(7) ^ edge.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
    value
}

fn normalize_screen_vector(start: Vec2, end: Vec2, viewport: &Viewport) -> Vec2 {
    let dx = (end.x - start.x) * viewport.half_width;
    let dy = (end.y - start.y) * viewport.half_height;
    normalize_vec2(Vec2::new(dx, dy))
}

fn normalize_vec2(value: Vec2) -> Vec2 {
    let len = (value.x * value.x + value.y * value.y).sqrt();
    if len <= f32::EPSILON {
        Vec2::ZERO
    } else {
        Vec2::new(value.x / len, value.y / len)
    }
}

fn dot_vec2(left: Vec2, right: Vec2) -> f32 {
    left.x * right.x + left.y * right.y
}

fn distance_vec2(left: Vec2, right: Vec2) -> f32 {
    ((left.x - right.x).powi(2) + (left.y - right.y).powi(2)).sqrt()
}

fn mul_vec2(value: Vec2, scalar: f32) -> Vec2 {
    Vec2::new(value.x * scalar, value.y * scalar)
}

fn deterministic_noise(seed: u64, edge: u64, pass: u64, salt: u64) -> f32 {
    let mut value = seed
        ^ edge.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ pass.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ salt.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    ((value >> 40) as f32) / ((1u64 << 24) as f32)
}

fn deterministic_signed_noise(seed: u64, edge: u64, pass: u64, salt: u64) -> f32 {
    deterministic_noise(seed, edge, pass, salt) * 2.0 - 1.0
}

pub(crate) fn append_text_3d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    content: &str,
    transform: Transform3,
    size: f32,
    color: ColorRgba,
) {
    let pixel_size = (size * 0.18).max(0.05);
    let advance = 6.0 * pixel_size;
    let text_width = content.chars().count() as f32 * advance;
    let start_x = -text_width * 0.5;
    let start_y = -3.5 * pixel_size;

    for (index, ch) in content.chars().enumerate() {
        let rows = glyph_rows(ch);
        let glyph_origin_x = start_x + index as f32 * advance;
        for (row_index, row_bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if row_bits & (1 << (4 - column)) == 0 {
                    continue;
                }

                let min = Vec3::new(
                    glyph_origin_x + column as f32 * pixel_size,
                    start_y + (6 - row_index) as f32 * pixel_size,
                    0.0,
                );
                let max = Vec3::new(min.x + pixel_size, min.y + pixel_size, 0.0);
                let quad = [
                    transform_point_3d(min, transform),
                    transform_point_3d(Vec3::new(max.x, min.y, 0.0), transform),
                    transform_point_3d(max, transform),
                    transform_point_3d(Vec3::new(min.x, max.y, 0.0), transform),
                ];
                let [Some(a), Some(b), Some(c), Some(d)] = quad.map(|point| {
                    project_point(point, camera, *viewport).map(|projected| projected.position)
                }) else {
                    continue;
                };
                push_quad(vertices, a, b, c, d, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex_signature(vertices: &[ColorVertex]) -> Vec<[i32; 6]> {
        vertices
            .iter()
            .map(|vertex| {
                [
                    (vertex.position[0] * 10_000.0).round() as i32,
                    (vertex.position[1] * 10_000.0).round() as i32,
                    (vertex.color[0] * 10_000.0).round() as i32,
                    (vertex.color[1] * 10_000.0).round() as i32,
                    (vertex.color[2] * 10_000.0).round() as i32,
                    (vertex.color[3] * 10_000.0).round() as i32,
                ]
            })
            .collect()
    }

    fn test_npr_path(id: u64, points: &[(f32, f32)]) -> NprStrokePath {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let points = points
            .iter()
            .map(|(x, y)| Vec2::new(*x, *y))
            .collect::<Vec<_>>();
        NprStrokePath {
            path_id: id,
            kind: NprLineKind::Silhouette,
            source_edges: vec![id],
            sorted_source_edges: vec![id],
            arc_lengths_px: npr_path_arc_lengths(&points, &viewport),
            importance: 1.0,
            closed: false,
            points,
        }
    }

    fn test_npr_path_with_kind(
        id: u64,
        kind: NprLineKind,
        points: &[(f32, f32)],
    ) -> NprStrokePath {
        let mut path = test_npr_path(id, points);
        path.kind = kind;
        path
    }

    fn test_npr_path_with_edges(
        id: u64,
        edges: Vec<u64>,
        points: &[(f32, f32)],
    ) -> NprStrokePath {
        let mut path = test_npr_path(id, points);
        path.sorted_source_edges = sorted_npr_source_edges(&edges);
        path.source_edges = edges;
        path
    }

    #[test]
    fn npr_debug_overlay_maps_camera_debug_views() {
        assert_eq!(
            NprDebugOverlay3d::from_camera_debug_view(&amigo_render_api::CameraDebugView2d::new(
                "npr.line_kinds"
            )),
            Some(NprDebugOverlay3d::LineKinds)
        );
        assert_eq!(
            NprDebugOverlay3d::from_camera_debug_view(&amigo_render_api::CameraDebugView2d::new(
                "npr.dropout"
            )),
            Some(NprDebugOverlay3d::Dropout)
        );
        assert_eq!(
            NprDebugOverlay3d::from_camera_debug_view(&amigo_render_api::CameraDebugView2d::final_output()),
            None
        );
    }

    #[test]
    fn npr_debug_overlay_emits_line_kind_vertices_without_changing_style_settings() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = test_npr_path_with_kind(
            17,
            NprLineKind::Crease,
            &[(-0.5, -0.25), (0.0, 0.1), (0.5, -0.25)],
        );
        let settings = amigo_render_api::NprLineSettings3d::default();
        let mut vertices = Vec::new();

        append_npr_debug_path_vertices(
            &mut vertices,
            &viewport,
            &path,
            &settings,
            NprDebugOverlay3d::LineKinds,
        );

        assert!(!vertices.is_empty());
        assert!(vertices.iter().any(|vertex| vertex.color[0] > 0.9));
    }

    #[test]
    fn npr_suggestive_edges_require_explicit_authoring() {
        let disabled = amigo_render_api::NprLineSettings3d {
            suggestive: false,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let enabled = amigo_render_api::NprLineSettings3d {
            suggestive: true,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_line_kind_for_edge(&disabled, false, false, false, false, true, false),
            None
        );
        assert_eq!(
            npr_line_kind_for_edge(&enabled, false, false, false, false, true, false),
            Some(NprLineKind::Feature)
        );
    }

    #[test]
    fn npr_contact_edges_require_explicit_authoring() {
        let disabled = amigo_render_api::NprLineSettings3d {
            contact: false,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let enabled = amigo_render_api::NprLineSettings3d {
            contact: true,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_line_kind_for_edge(&disabled, false, false, false, false, false, true),
            None
        );
        assert_eq!(
            npr_line_kind_for_edge(&enabled, false, false, false, false, false, true),
            Some(NprLineKind::Contact)
        );
    }

    #[test]
    fn npr_contact_authoring_reaches_backend_stats_and_vertices() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            boundary: true,
            silhouette: true,
            feature: true,
            suggestive: false,
            contact: true,
            contact_ground_y: 0.0,
            contact_threshold: 1.0,
            min_screen_length_px: 0.0,
            width_px: 2.0,
            pressure_jitter: 0.0,
            stroke_wobble_px: 0.0,
            overshoot_px: 0.0,
            dropout: 0.0,
            passes: 1,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut vertices = Vec::new();
        let mut history = BTreeMap::new();
        let stats = append_mesh_npr_line_vertices_with_history_and_stats(
            &mut history,
            1,
            "contact-cube",
            &mut vertices,
            None,
            &viewport,
            Transform3 {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform3::default()
            },
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );

        assert!(
            stats.contact_paths > 0,
            "expected visible contact paths, got stats={stats:?}"
        );
        assert_eq!(stats.paths, stats.contact_paths);
        assert_eq!(stats.boundary_paths, 0);
        assert_eq!(stats.silhouette_paths, 0);
        assert_eq!(stats.crease_paths, 0);
        assert_eq!(stats.seam_paths, 0);
        assert_eq!(stats.feature_paths, 0);
        assert_eq!(stats.strip_vertices, vertices.len());
        assert!(!vertices.is_empty());
    }

    fn y_span(vertices: &[ColorVertex]) -> f32 {
        let min_y = vertices
            .iter()
            .map(|vertex| vertex.position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = vertices
            .iter()
            .map(|vertex| vertex.position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        max_y - min_y
    }

    #[test]
    fn cube_geometry_exposes_edges_for_npr_lines() {
        let geometry = cube_geometry();
        assert_eq!(geometry.vertices.len(), 8);
        assert_eq!(geometry.triangles.len(), 12);
        assert!(!geometry.edges.is_empty());
    }

    #[test]
    fn loads_playground_npr_box_glb_geometry() {
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();
        let path = workspace_root.join("mods/playground-npr/source-models/khronos/Box.glb");
        let geometry = load_glb_geometry(&path).expect("playground npr box glb should import");
        assert!(!geometry.vertices.is_empty());
        assert!(!geometry.triangles.is_empty());
        assert!(!geometry.edges.is_empty());
    }

    #[test]
    #[ignore = "manual NPR benchmark; run with --ignored --nocapture"]
    fn benchmark_playground_npr_soldier_rotation_cpu_stroke_workload() {
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();
        let path = workspace_root.join("mods/playground-npr/source-models/threejs/Soldier.glb");
        let geometry = load_glb_geometry(&path).expect("playground npr soldier glb should import");
        let viewport = Viewport::from_dimensions(1280.0, 720.0);
        let camera = Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.2),
            ..Transform3::default()
        };
        let cinematic_settings = amigo_render_api::NprLineSettings3d {
            style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
            stroke_tool: amigo_render_api::NprStrokeTool3d::InkPen,
            boundary: true,
            silhouette: true,
            feature: true,
            suggestive: true,
            contact: true,
            contact_ground_y: -0.5,
            contact_threshold: 0.08,
            min_screen_length_px: 2.0,
            feature_angle_degrees: 32.0,
            width_px: 2.35,
            passes: 2,
            search_line_count: 1,
            temporal_stability: 0.92,
            visibility_hysteresis_frames: 3,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let target_60fps_settings = amigo_render_api::NprLineSettings3d {
            style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
            stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
            boundary: true,
            silhouette: true,
            feature: true,
            suggestive: false,
            contact: false,
            contact_ground_y: -0.5,
            contact_threshold: 0.06,
            min_screen_length_px: 3.4,
            feature_angle_degrees: 42.0,
            width_px: 2.0,
            silhouette_width_multiplier: 1.50,
            boundary_width_multiplier: 0.92,
            feature_width_multiplier: 0.48,
            depth_pressure: 0.08,
            humanization: 0.22,
            line_confidence: 0.88,
            temporal_stability: 0.94,
            visibility_hysteresis_frames: 2,
            visibility_max_dimension_px: 720.0,
            endpoint_snap_px: 2.4,
            path_simplify_px: 1.45,
            passes: 1,
            search_line_count: 0,
            dropout: 0.0,
            dropout_segment_min_px: 12.0,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let low_120fps_settings = amigo_render_api::NprLineSettings3d {
            style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
            stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
            boundary: true,
            silhouette: true,
            feature: true,
            suggestive: false,
            contact: false,
            contact_ground_y: -0.5,
            contact_threshold: 0.04,
            min_screen_length_px: 5.0,
            feature_angle_degrees: 52.0,
            width_px: 1.85,
            silhouette_width_multiplier: 1.55,
            boundary_width_multiplier: 0.85,
            feature_width_multiplier: 0.32,
            depth_pressure: 0.04,
            humanization: 0.08,
            line_confidence: 0.95,
            temporal_stability: 0.96,
            visibility_hysteresis_frames: 1,
            visibility_max_dimension_px: 512.0,
            endpoint_snap_px: 3.0,
            path_simplify_px: 2.0,
            passes: 1,
            search_line_count: 0,
            dropout: 0.0,
            dropout_segment_min_px: 16.0,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let cases = [
            ("cinematic_12fps", cinematic_settings),
            ("target_60fps", target_60fps_settings),
            ("low_120fps", low_120fps_settings),
        ];

        let frames = 120u64;
        let warmup = 12usize;
        println!(
            "NPR soldier rotation benchmark: frames={} warmup={} measured={} viewport=1280x720 vertices={} triangles={} edges={}",
            frames,
            warmup,
            frames as usize - warmup,
            geometry.vertices.len(),
            geometry.triangles.len(),
            geometry.edges.len()
        );
        for (case_name, settings) in cases {
            let mut history = BTreeMap::new();
            let mut timings_us = Vec::new();
            let mut vertices = Vec::new();
            let mut npr_stroke_segments = Vec::new();
            let mut totals = NprStrokeFrameStats3d::default();

            for frame in 0..frames {
                vertices.clear();
                npr_stroke_segments.clear();
                let angle = frame as f32 / frames as f32 * std::f32::consts::TAU;
                let transform = Transform3 {
                    rotation_euler: Vec3::new(-std::f32::consts::FRAC_PI_2, angle, 0.0),
                    ..Transform3::default()
                };
                let start = std::time::Instant::now();
                let stats = append_mesh_npr_line_vertices_with_history_and_stats(
                    &mut history,
                    frame + 1,
                    "playground-npr-model-1-soldier",
                    &mut vertices,
                    Some(&mut npr_stroke_segments),
                    &viewport,
                    camera,
                    amigo_render_api::Camera3dRenderSettings::default(),
                    &geometry,
                    transform,
                    &settings,
                );
                let elapsed = start.elapsed();
                if frame as usize >= warmup {
                    timings_us.push(elapsed.as_secs_f64() * 1_000_000.0);
                    totals.add(stats);
                }
            }

            timings_us.sort_by(|left, right| {
                left.partial_cmp(right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let measured = timings_us.len().max(1);
            let measured_f64 = measured as f64;
            let mean_us = timings_us.iter().sum::<f64>() / measured_f64;
            let median_us = timings_us[measured / 2];
            let p95_us =
                timings_us[((measured as f32 * 0.95).floor() as usize).min(measured - 1)];
            let avg_paths = totals.paths as f64 / measured_f64;
            let avg_samples = totals.brush_samples as f64 / measured_f64;
            let avg_vertices = totals.strip_vertices as f64 / measured_f64;
            let avg_search_passes = totals.search_passes as f64 / measured_f64;
            let avg_path_build_us = totals.path_build_us / measured_f64;
            let avg_stabilize_us = totals.stabilize_us / measured_f64;
            let avg_stroke_vertices_us = totals.stroke_vertices_us / measured_f64;
            let avg_path_project_us = totals.path_project_us / measured_f64;
            let avg_path_visibility_us = totals.path_visibility_us / measured_f64;
            let avg_path_edge_sample_us = totals.path_edge_sample_us / measured_f64;
            let avg_path_stitch_us = totals.path_stitch_us / measured_f64;
            let avg_visible_edges = totals.path_visible_edges as f64 / measured_f64;
            let avg_fragments = totals.path_fragments as f64 / measured_f64;
            let cache_total = totals.cached_plan_hits + totals.cached_plan_misses;
            let cache_hit_rate = if cache_total == 0 {
                0.0
            } else {
                totals.cached_plan_hits as f64 / cache_total as f64 * 100.0
            };

            println!(
                "case={case_name} cpu_stroke_us mean={mean_us:.2} median={median_us:.2} p95={p95_us:.2}"
            );
            println!(
                "case={case_name} workload avg_paths={avg_paths:.2} avg_samples={avg_samples:.2} avg_vertices={avg_vertices:.2} avg_search_passes={avg_search_passes:.2} cache_hit_rate={cache_hit_rate:.1}% visibility_max_dimension_px={:.0}",
                settings.visibility_max_dimension_px
            );
            println!(
                "case={case_name} stage_cpu_us path_build={avg_path_build_us:.2} stabilize={avg_stabilize_us:.2} stroke_vertices={avg_stroke_vertices_us:.2}"
            );
            println!(
                "case={case_name} path_build_breakdown_us project={avg_path_project_us:.2} visibility={avg_path_visibility_us:.2} edge_sample={avg_path_edge_sample_us:.2} stitch={avg_path_stitch_us:.2} visible_edges={avg_visible_edges:.2} fragments={avg_fragments:.2}"
            );
            println!(
                "case={case_name} path_mix boundary={} silhouette={} crease={} seam={} feature={} contact={}",
                totals.boundary_paths,
                totals.silhouette_paths,
                totals.crease_paths,
                totals.seam_paths,
                totals.feature_paths,
                totals.contact_paths
            );

            assert!(mean_us.is_finite());
            assert!(avg_paths > 0.0);
            assert!(avg_vertices > 0.0);
        }
    }

    #[test]
    fn npr_line_segment_uses_pixel_width() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let mut vertices = Vec::new();
        append_npr_stroke_strip_vertices(
            &mut vertices,
            &viewport,
            &[
                NprStrokeStripSample {
                    point: Vec2::new(-0.5, 0.0),
                    width_px: 4.0,
                    offset_px: 0.0,
                    overshoot_px: 0.0,
                    color: ColorRgba::WHITE,
                },
                NprStrokeStripSample {
                    point: Vec2::new(0.5, 0.0),
                    width_px: 4.0,
                    offset_px: 0.0,
                    overshoot_px: 0.0,
                    color: ColorRgba::WHITE,
                },
            ],
        );
        assert_eq!(vertices.len(), 6);
        let height_ndc = (vertices[0].position[1] - vertices[2].position[1]).abs();
        assert!((height_ndc - (4.0 / viewport.half_height)).abs() < 0.0001);
    }

    #[test]
    fn npr_stroke_strip_connects_multiple_samples() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let mut vertices = Vec::new();
        let samples = [
            NprStrokeStripSample {
                point: Vec2::new(-0.5, 0.0),
                width_px: 4.0,
                offset_px: 0.0,
                overshoot_px: 0.0,
                color: ColorRgba::WHITE,
            },
            NprStrokeStripSample {
                point: Vec2::new(0.0, 0.1),
                width_px: 4.0,
                offset_px: 0.0,
                overshoot_px: 0.0,
                color: ColorRgba::WHITE,
            },
            NprStrokeStripSample {
                point: Vec2::new(0.5, 0.0),
                width_px: 4.0,
                offset_px: 0.0,
                overshoot_px: 0.0,
                color: ColorRgba::WHITE,
            },
        ];

        append_npr_stroke_strip_vertices(&mut vertices, &viewport, &samples);

        assert_eq!(vertices.len(), 12);
    }

    #[test]
    fn npr_fragments_join_into_quantized_stroke_path() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let fragments = vec![
            NprLineFragment {
                source_edge_id: 11,
                kind: NprLineKind::Feature,
                p0: Vec2::new(-0.2, 0.0),
                p1: Vec2::new(0.0, 0.0),
                t0: 0.0,
                t1: 0.5,
                tangent0: Vec2::new(1.0, 0.0),
                tangent1: Vec2::new(1.0, 0.0),
                avg_depth: 1.0,
            },
            NprLineFragment {
                source_edge_id: 11,
                kind: NprLineKind::Feature,
                p0: Vec2::new(0.002, 0.001),
                p1: Vec2::new(0.2, 0.0),
                t0: 0.5,
                t1: 1.0,
                tangent0: Vec2::new(1.0, 0.0),
                tangent1: Vec2::new(1.0, 0.0),
                avg_depth: 1.0,
            },
        ];

        let paths = build_npr_stroke_paths(&fragments, &viewport, 2.5, 0.0);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].kind, NprLineKind::Feature);
        assert_eq!(paths[0].points.len(), 3);
    }

    #[test]
    fn npr_stable_path_id_uses_source_edges_not_projection() {
        let forward = stable_path_id(NprLineKind::Silhouette, &[11, 22, 33]);
        let reversed = stable_path_id(NprLineKind::Silhouette, &[33, 22, 11]);
        let crease = stable_path_id(NprLineKind::Crease, &[11, 22, 33]);

        assert_eq!(forward, reversed);
        assert_ne!(forward, crease);
    }

    #[test]
    fn npr_dihedral_feature_edges_classify_as_crease_lines() {
        let settings = amigo_render_api::NprLineSettings3d {
            feature: true,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_line_kind_for_edge(&settings, false, false, true, false, false, false),
            Some(NprLineKind::Crease)
        );
    }

    #[test]
    fn npr_material_seam_edges_classify_as_seam_lines() {
        let settings = amigo_render_api::NprLineSettings3d {
            feature: true,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(
            npr_line_kind_for_edge(&settings, false, false, false, true, false, false),
            Some(NprLineKind::Seam)
        );
    }

    #[test]
    fn npr_path_simplification_removes_nearly_collinear_points() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let points = vec![
            Vec2::new(-0.5, 0.0),
            Vec2::new(0.0, 0.0005),
            Vec2::new(0.5, 0.0),
        ];

        let simplified = simplify_npr_path(&points, &viewport, 1.0);

        assert_eq!(simplified, vec![points[0], points[2]]);
    }

    #[test]
    fn npr_visibility_fragments_require_owned_face_samples() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let edge = MeshEdge3d {
            edge_id: 11,
            a: 0,
            b: 1,
            faces: vec![0],
            material_seam: false,
        };
        let a = ProjectedPoint {
            position: Vec2::new(-0.25, 0.0),
            depth: 2.0,
        };
        let b = ProjectedPoint {
            position: Vec2::new(0.25, 0.0),
            depth: 2.0,
        };
        let owned_visibility = NprFaceVisibilityBuffer {
            width: 16,
            height: 16,
            face_id: vec![0; 16 * 16],
            face_visible: vec![true],
        };
        let hidden_visibility = NprFaceVisibilityBuffer {
            width: 16,
            height: 16,
            face_id: vec![1; 16 * 16],
            face_visible: vec![true, true],
        };

        let visible = visible_npr_fragments_for_edge(
            &owned_visibility,
            &edge,
            NprLineKind::Boundary,
            a,
            b,
            &viewport,
            1.0,
        );
        let hidden = visible_npr_fragments_for_edge(
            &hidden_visibility,
            &edge,
            NprLineKind::Boundary,
            a,
            b,
            &viewport,
            1.0,
        );

        assert_eq!(visible.len(), 1);
        assert!(hidden.is_empty());
    }

    #[test]
    fn npr_lines_skip_back_facing_feature_edges() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            boundary: false,
            silhouette: false,
            feature: true,
            feature_angle_degrees: 1.0,
            min_screen_length_px: 0.0,
            width_px: 2.0,
            pressure_jitter: 0.0,
            stroke_wobble_px: 0.0,
            overshoot_px: 0.0,
            dropout: 0.0,
            passes: 1,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut vertices = Vec::new();
        append_mesh_npr_line_vertices(
            &mut vertices,
            &viewport,
            Transform3 {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform3::default()
            },
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );
        assert!(
            vertices.is_empty(),
            "feature-only pass should not draw back-facing cube edges"
        );

        let mut silhouette_vertices = Vec::new();
        append_mesh_npr_line_vertices(
            &mut silhouette_vertices,
            &viewport,
            Transform3 {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform3::default()
            },
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &amigo_render_api::NprLineSettings3d {
                boundary: false,
                silhouette: true,
                feature: false,
                min_screen_length_px: 0.0,
                width_px: 2.0,
                pressure_jitter: 0.0,
                stroke_wobble_px: 0.0,
                overshoot_px: 0.0,
                dropout: 0.0,
                passes: 1,
                ..amigo_render_api::NprLineSettings3d::default()
            },
        );
        assert!(!silhouette_vertices.is_empty());
    }

    #[test]
    fn npr_line_generation_is_deterministic_for_same_seed() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            seed: 2222,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let camera = Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        };
        let mut first = Vec::new();
        let mut second = Vec::new();

        append_mesh_npr_line_vertices(
            &mut first,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );
        append_mesh_npr_line_vertices(
            &mut second,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );

        assert_eq!(vertex_signature(&first), vertex_signature(&second));
    }

    #[test]
    fn npr_backend_stats_report_generated_stroke_work() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            boundary: false,
            silhouette: true,
            feature: false,
            min_screen_length_px: 0.0,
            width_px: 2.0,
            pressure_jitter: 0.0,
            stroke_wobble_px: 0.0,
            overshoot_px: 0.0,
            dropout: 0.0,
            passes: 1,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut vertices = Vec::new();
        let mut history = BTreeMap::new();
        let stats = append_mesh_npr_line_vertices_with_history_and_stats(
            &mut history,
            1,
            "cube",
            &mut vertices,
            None,
            &viewport,
            Transform3 {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform3::default()
            },
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );

        assert_eq!(stats.meshes, 1);
        assert!(stats.paths > 0);
        assert_eq!(stats.silhouette_paths, stats.paths);
        assert_eq!(stats.boundary_paths, 0);
        assert_eq!(stats.crease_paths, 0);
        assert_eq!(stats.seam_paths, 0);
        assert_eq!(stats.feature_paths, 0);
        assert!(stats.brush_samples >= stats.paths * 2);
        assert!(stats.primary_passes >= stats.paths);
        assert_eq!(stats.search_passes, 0);
        assert_eq!(stats.cached_plan_hits, 0);
        assert_eq!(stats.cached_plan_misses, stats.paths);
        assert_eq!(stats.strip_vertices, vertices.len());
        assert!(!vertices.is_empty());
    }

    #[test]
    fn npr_line_generation_changes_with_seed_without_dropping_paths() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let camera = Transform3 {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform3::default()
        };
        let mut first = Vec::new();
        let mut second = Vec::new();

        append_mesh_npr_line_vertices(
            &mut first,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &amigo_render_api::NprLineSettings3d {
                seed: 1001,
                ..amigo_render_api::NprLineSettings3d::default()
            },
        );
        append_mesh_npr_line_vertices(
            &mut second,
            &viewport,
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &amigo_render_api::NprLineSettings3d {
                seed: 1002,
                ..amigo_render_api::NprLineSettings3d::default()
            },
        );

        assert!(!first.is_empty());
        assert_eq!(first.len(), second.len());
        assert_ne!(vertex_signature(&first), vertex_signature(&second));
    }

    #[test]
    fn npr_straightness_controls_humanized_path_deviation() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = test_npr_path(808, &[(-0.55, 0.0), (-0.15, 0.0), (0.2, 0.0), (0.55, 0.0)]);
        let loose = amigo_render_api::NprLineSettings3d {
            straightness: 0.0,
            stroke_wobble_px: 1.2,
            micro_wobble_px: 0.35,
            humanization: 1.0,
            passes: 1,
            search_line_count: 0,
            dropout: 0.0,
            pressure_jitter: 0.0,
            seed: 909,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let straight = amigo_render_api::NprLineSettings3d {
            straightness: 1.0,
            ..loose.clone()
        };
        let mut loose_vertices = Vec::new();
        let mut straight_vertices = Vec::new();

        append_npr_styled_path_vertices(
            &mut loose_vertices,
            None,
            &viewport,
            &path,
            &loose,
            None,
            &mut NprStrokeFrameStats3d::default(),
        );
        append_npr_styled_path_vertices(
            &mut straight_vertices,
            None,
            &viewport,
            &path,
            &straight,
            None,
            &mut NprStrokeFrameStats3d::default(),
        );

        let loose_span = y_span(&loose_vertices);
        let straight_span = y_span(&straight_vertices);
        assert!(
            loose_span > straight_span,
            "lower straightness should increase gesture deviation"
        );
    }

    #[test]
    fn npr_alpha_pressure_curve_fades_stroke_along_arc_length() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = test_npr_path(909, &[(-0.6, 0.0), (-0.2, 0.0), (0.2, 0.0), (0.6, 0.0)]);
        let settings = amigo_render_api::NprLineSettings3d {
            alpha_pressure_curve: [1.0, 1.0, 0.25, 0.2],
            stroke_wobble_px: 0.0,
            micro_wobble_px: 0.0,
            pressure_jitter: 0.0,
            dropout: 0.0,
            passes: 1,
            search_line_count: 0,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut vertices = Vec::new();

        append_npr_styled_path_vertices(
            &mut vertices,
            None,
            &viewport,
            &path,
            &settings,
            None,
            &mut NprStrokeFrameStats3d::default(),
        );

        let max_alpha = vertices
            .iter()
            .map(|vertex| vertex.color[3])
            .fold(0.0, f32::max);
        let min_alpha = vertices
            .iter()
            .map(|vertex| vertex.color[3])
            .fold(1.0, f32::min);
        assert!(min_alpha < max_alpha * 0.35);
    }

    #[test]
    fn npr_depth_alpha_modulates_stroke_opacity() {
        let settings = amigo_render_api::NprLineSettings3d {
            depth_alpha: 0.5,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let near = npr_depth_alpha_multiplier(1.2, &settings);
        let far = npr_depth_alpha_multiplier(0.2, &settings);

        assert!(near > far);
    }

    #[test]
    fn npr_stroke_tool_profiles_change_drawing_dynamics() {
        let ink = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::InkPen,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let pencil = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let brush = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::Brush,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let technical = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert!(npr_tool_search_multiplier(&pencil) > npr_tool_search_multiplier(&ink));
        assert_eq!(npr_tool_search_multiplier(&technical), 0.0);
        assert!(npr_tool_dropout_multiplier(&pencil) > npr_tool_dropout_multiplier(&ink));
        assert_eq!(npr_tool_dropout_multiplier(&technical), 0.0);
        assert!(npr_tool_width_multiplier(&brush) > npr_tool_width_multiplier(&ink));
        assert!(npr_tool_alpha_multiplier(&pencil) < npr_tool_alpha_multiplier(&ink));
        assert!(
            npr_tool_pressure_jitter_multiplier(&technical)
                < npr_tool_pressure_jitter_multiplier(&ink)
        );
    }

    #[test]
    fn npr_tool_override_multipliers_author_tool_dynamics() {
        let settings = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
            tool_search_multiplier: 0.0,
            tool_dropout_multiplier: 0.0,
            tool_alpha_multiplier: 0.5,
            tool_width_multiplier: 2.0,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(npr_tool_search_multiplier(&settings), 0.0);
        assert_eq!(npr_tool_dropout_multiplier(&settings), 0.0);
        assert!(npr_tool_alpha_multiplier(&settings) < 0.5);
        assert!(npr_tool_width_multiplier(&settings) > 1.5);
    }

    #[test]
    fn npr_stable_brush_path_resamples_by_arc_length() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = test_npr_path(913, &[(-0.5, 0.0), (0.5, 0.0)]);

        let brush_path = build_npr_stable_brush_path(&path, &viewport);

        assert!(brush_path.samples.len() > path.points.len());
        assert_eq!(brush_path.samples[0].point, path.points[0]);
        assert_eq!(brush_path.samples.last().expect("last sample").point, path.points[1]);
        assert!(brush_path
            .samples
            .windows(2)
            .all(|window| window[1].arc_length_px >= window[0].arc_length_px));
    }

    #[test]
    fn npr_stroke_pass_plan_does_not_search_duplicate_silhouettes() {
        let silhouette = test_npr_path(910, &[(-0.5, 0.0), (0.5, 0.0)]);
        let feature = test_npr_path_with_kind(
            911,
            NprLineKind::Crease,
            &[(-0.5, 0.0), (0.5, 0.0)],
        );
        let settings = amigo_render_api::NprLineSettings3d {
            style_preset: amigo_render_api::NprStylePreset3d::RoughComicInk,
            stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
            passes: 1,
            search_line_count: 2,
            gpu_realtime_tuning: amigo_render_api::NprGpuRealtimeTuning3d {
                search_enabled: true,
                ..amigo_render_api::NprGpuRealtimeTuning3d::default()
            },
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let silhouette_plan = build_npr_stroke_pass_plan(
            &silhouette,
            &settings,
            build_npr_stroke_gesture(&silhouette, &settings),
        );
        let feature_plan = build_npr_stroke_pass_plan(
            &feature,
            &settings,
            build_npr_stroke_gesture(&feature, &settings),
        );

        assert_eq!(silhouette_plan.len(), 1);
        assert!(silhouette_plan
            .iter()
            .all(|pass| pass.kind == NprStrokePassKind::Primary));
        assert_eq!(feature_plan.len(), 4);
        assert_eq!(
            feature_plan
                .iter()
                .filter(|pass| pass.kind == NprStrokePassKind::Search)
                .count(),
            3
        );
    }

    #[test]
    fn npr_dropout_mask_protects_primary_silhouette_segments() {
        let path = test_npr_path(912, &[(-0.5, 0.0), (0.0, 0.0), (0.5, 0.0)]);
        let settings = amigo_render_api::NprLineSettings3d {
            dropout: 1.0,
            dropout_segment_min_px: 1.0,
            passes: 1,
            search_line_count: 0,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let gesture = build_npr_stroke_gesture(&path, &settings);
        let pass = build_npr_stroke_pass_plan(&path, &settings, gesture)[0];
        let dropout = build_npr_dropout_mask(gesture, &settings, &[pass]);

        assert!(dropout.keeps_segment(pass, 0.0, 0.5, 100.0));
        assert!(dropout.keeps_segment(pass, 0.5, 1.0, 100.0));
    }

    #[test]
    fn npr_cached_stroke_plan_reuses_dropout_for_stable_path() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let path = test_npr_path_with_kind(
            920,
            NprLineKind::Crease,
            &[(-0.5, 0.0), (0.0, 0.0), (0.5, 0.0)],
        );
        let settings = amigo_render_api::NprLineSettings3d {
            dropout: 0.8,
            dropout_segment_min_px: 1.0,
            passes: 1,
            search_line_count: 0,
            seed: 99,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut vertices = Vec::new();
        let mut stats = NprStrokeFrameStats3d::default();
        let first_plan = append_npr_styled_path_vertices(
            &mut vertices,
            None,
            &viewport,
            &path,
            &settings,
            None,
            &mut stats,
        );
        let mut second_vertices = Vec::new();
        let mut second_stats = NprStrokeFrameStats3d::default();
        let second_plan = append_npr_styled_path_vertices(
            &mut second_vertices,
            None,
            &viewport,
            &path,
            &settings,
            Some(&first_plan),
            &mut second_stats,
        );

        assert_eq!(first_plan.settings_signature, second_plan.settings_signature);
        assert_eq!(first_plan.length_bucket_px, second_plan.length_bucket_px);
        assert_eq!(
            first_plan.dropout.intervals.len(),
            second_plan.dropout.intervals.len()
        );
        assert_eq!(vertex_signature(&vertices), vertex_signature(&second_vertices));
    }

    #[test]
    fn npr_backend_stats_report_cached_plan_hits_on_second_frame() {
        let geometry = cube_geometry();
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            boundary: false,
            silhouette: true,
            feature: false,
            min_screen_length_px: 0.0,
            width_px: 2.0,
            pressure_jitter: 0.0,
            stroke_wobble_px: 0.0,
            overshoot_px: 0.0,
            dropout: 0.0,
            passes: 1,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut history = BTreeMap::new();
        let mut first_vertices = Vec::new();
        let first = append_mesh_npr_line_vertices_with_history_and_stats(
            &mut history,
            1,
            "cube",
            &mut first_vertices,
            None,
            &viewport,
            Transform3 {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform3::default()
            },
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );
        let mut second_vertices = Vec::new();
        let second = append_mesh_npr_line_vertices_with_history_and_stats(
            &mut history,
            2,
            "cube",
            &mut second_vertices,
            None,
            &viewport,
            Transform3 {
                translation: Vec3::new(0.0, 0.0, 4.0),
                ..Transform3::default()
            },
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            Transform3::default(),
            &settings,
        );

        assert!(first.cached_plan_misses > 0);
        assert_eq!(first.cached_plan_hits, 0);
        assert!(second.cached_plan_hits > 0);
        assert!(second.cached_plan_misses < first.cached_plan_misses);
    }

    #[test]
    fn npr_cached_stroke_plan_invalidates_when_seed_changes() {
        let path = test_npr_path_with_kind(
            921,
            NprLineKind::Crease,
            &[(-0.5, 0.0), (0.0, 0.0), (0.5, 0.0)],
        );
        let settings = amigo_render_api::NprLineSettings3d {
            dropout: 0.8,
            dropout_segment_min_px: 1.0,
            passes: 1,
            search_line_count: 0,
            seed: 99,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let changed_seed = amigo_render_api::NprLineSettings3d {
            seed: 100,
            ..settings.clone()
        };
        let gesture = build_npr_stroke_gesture(&path, &settings);
        let cached = build_npr_cached_stroke_plan(&path, &settings, gesture);

        assert!(!cached.is_compatible(
            &changed_seed,
            build_npr_stroke_gesture(&path, &changed_seed)
        ));
    }

    #[test]
    fn npr_temporal_history_retains_path_for_hysteresis_window() {
        let mut history = BTreeMap::new();
        let settings = amigo_render_api::NprLineSettings3d {
            visibility_hysteresis_frames: 3,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let first = stabilize_npr_paths_for_entity(
            &mut history,
            1,
            "entity",
            &settings,
            vec![test_npr_path(77, &[(-0.2, 0.0), (0.2, 0.0)])],
        );
        let retained = stabilize_npr_paths_for_entity(&mut history, 2, "entity", &settings, vec![]);
        let retained_again =
            stabilize_npr_paths_for_entity(&mut history, 3, "entity", &settings, vec![]);
        let dropped = stabilize_npr_paths_for_entity(&mut history, 4, "entity", &settings, vec![]);

        assert_eq!(first.len(), 1);
        assert_eq!(retained.len(), 1);
        assert_eq!(retained_again.len(), 1);
        assert!(dropped.is_empty());
    }

    #[test]
    fn npr_temporal_history_blends_returning_path_points() {
        let mut history = BTreeMap::new();
        let settings = amigo_render_api::NprLineSettings3d {
            temporal_stability: 0.9,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let _ = stabilize_npr_paths_for_entity(
            &mut history,
            1,
            "entity",
            &settings,
            vec![test_npr_path(99, &[(-0.2, 0.0), (0.2, 0.0)])],
        );
        let blended = stabilize_npr_paths_for_entity(
            &mut history,
            2,
            "entity",
            &settings,
            vec![test_npr_path(99, &[(-0.2, 0.1), (0.2, 0.1)])],
        );

        assert_eq!(blended.len(), 1);
        assert!(blended[0].points[0].y > 0.0);
        assert!(blended[0].points[0].y < 0.1);
    }

    #[test]
    fn npr_temporal_history_matches_changed_path_id_by_source_overlap() {
        let mut history = BTreeMap::new();
        let settings = amigo_render_api::NprLineSettings3d {
            temporal_stability: 0.9,
            visibility_hysteresis_frames: 4,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let _ = stabilize_npr_paths_for_entity(
            &mut history,
            1,
            "entity",
            &settings,
            vec![test_npr_path_with_edges(
                100,
                vec![10, 20, 30],
                &[(-0.2, 0.0), (0.2, 0.0)],
            )],
        );
        let blended = stabilize_npr_paths_for_entity(
            &mut history,
            2,
            "entity",
            &settings,
            vec![test_npr_path_with_edges(
                101,
                vec![20, 30, 40],
                &[(-0.2, 0.08), (0.2, 0.08)],
            )],
        );

        assert_eq!(blended.len(), 1);
        assert_eq!(history.len(), 1);
        assert!(blended[0].points[0].y > 0.0);
        assert!(blended[0].points[0].y < 0.08);
    }
}
