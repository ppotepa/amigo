use crate::renderer::*;

#[derive(Debug, Clone)]
pub(crate) struct CachedMeshGeometry3d {
    vertices: Vec<Vec3>,
    triangles: Vec<MeshTriangle3d>,
    edges: Vec<MeshEdge3d>,
}

#[derive(Debug, Clone, Copy)]
struct MeshTriangle3d {
    indices: [usize; 3],
    normal: Vec3,
}

#[derive(Debug, Clone)]
struct MeshEdge3d {
    a: usize,
    b: usize,
    faces: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NprLineKind {
    Boundary,
    Silhouette,
    Feature,
}

#[derive(Debug, Clone, Copy)]
struct NprLineFragment {
    kind: NprLineKind,
    p0: Vec2,
    p1: Vec2,
}

#[derive(Debug, Clone)]
struct NprStrokePath {
    kind: NprLineKind,
    points: Vec<Vec2>,
}

#[derive(Debug, Clone)]
struct NprFaceVisibilityBuffer {
    width: usize,
    height: usize,
    face_id: Vec<Option<usize>>,
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

pub(crate) fn append_mesh_npr_line_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    if settings.passes == 0 || settings.width_px <= 0.0 {
        return;
    }

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
    let visibility = build_npr_face_visibility_buffer(geometry, &projected_vertices, viewport);
    let face_front = geometry
        .triangles
        .iter()
        .map(|triangle| {
            let world = triangle.indices.map(|index| world_vertices[index]);
            let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
            dot(normal, sub(camera.translation, triangle_center(world))) > 0.0
        })
        .collect::<Vec<_>>();
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

    let mut fragments = Vec::new();

    for edge in &geometry.edges {
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
        let feature = edge.faces.len() == 2
            && vis0
            && vis1
            && edge_angle_degrees(world_normals[edge.faces[0]], world_normals[edge.faces[1]])
                >= settings.feature_angle_degrees.max(0.0);
        let Some(kind) = npr_line_kind_for_edge(settings, boundary, silhouette, feature) else {
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

        fragments.extend(visible_npr_fragments_for_edge(
            &visibility,
            edge,
            kind,
            a,
            b,
            viewport,
            settings.min_screen_length_px,
        ));
    }

    let paths = build_npr_stroke_paths(
        &fragments,
        viewport,
        settings.endpoint_quant_px,
        settings.path_simplify_px,
    );
    for (path_index, path) in paths.iter().enumerate() {
        append_npr_styled_path_vertices(vertices, viewport, path, path_index, settings);
    }
}

impl WgpuSceneRenderer {
    pub(crate) fn mesh_geometry_3d(
        &mut self,
        assets: &dyn amigo_render_api::RenderAssetSource,
        mesh_asset: &amigo_assets::AssetKey,
    ) -> CachedMeshGeometry3d {
        let cache_key = mesh_asset.as_str().to_owned();
        if let Some(cached) = self.mesh_3d_geometry_cache.get(&cache_key) {
            return cached.clone();
        }

        let geometry = mesh_geometry_from_asset(assets, mesh_asset).unwrap_or_else(cube_geometry);
        self.mesh_3d_geometry_cache
            .insert(cache_key, geometry.clone());
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
) {
    let normal = normalize(cross(
        sub(vertices[indices[1]], vertices[indices[0]]),
        sub(vertices[indices[2]], vertices[indices[0]]),
    ));
    if normal == Vec3::ZERO {
        return;
    }
    triangles.push(MeshTriangle3d { indices, normal });
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
        .map(|((a, b), faces)| MeshEdge3d { a, b, faces })
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
        push_imported_triangle(&mut triangles, &vertices, indices);
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
) -> NprFaceVisibilityBuffer {
    let size = viewport.size();
    let max_dimension = size.x.max(size.y).max(1.0);
    let scale = (1024.0 / max_dimension).min(1.0);
    let width = (size.x * scale).round().max(8.0) as usize;
    let height = (size.y * scale).round().max(8.0) as usize;
    let mut depth = vec![f32::INFINITY; width * height];
    let mut face_id = vec![None; width * height];
    let mut face_visible = vec![false; geometry.triangles.len()];

    for (face_index, triangle) in geometry.triangles.iter().enumerate() {
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

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let w0 = npr_edge_function(b.x, b.y, c.x, c.y, px, py);
                let w1 = npr_edge_function(c.x, c.y, a.x, a.y, px, py);
                let w2 = npr_edge_function(a.x, a.y, b.x, b.y, px, py);
                let inside = if area >= 0.0 {
                    w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5
                } else {
                    w0 <= 1e-5 && w1 <= 1e-5 && w2 <= 1e-5
                };
                if !inside {
                    continue;
                }

                let l0 = w0 / area;
                let l1 = w1 / area;
                let l2 = w2 / area;
                let sample_depth = l0 * a.z + l1 * b.z + l2 * c.z;
                let index = y * width + x;
                if sample_depth < depth[index] {
                    depth[index] = sample_depth;
                    face_id[index] = Some(face_index);
                }
            }
        }
    }

    for face in face_id.iter().flatten() {
        if let Some(visible) = face_visible.get_mut(*face) {
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
            run_start = Some(point);
        }
        if !visible && previous_visible {
            if let (Some(start), Some(end)) = (run_start, previous_point) {
                push_visible_npr_fragment(
                    &mut fragments,
                    kind,
                    start,
                    end,
                    viewport,
                    min_segment_px,
                );
            }
            run_start = None;
        }

        previous_visible = visible;
        previous_point = Some(point);
    }

    if previous_visible {
        if let (Some(start), Some(end)) = (run_start, previous_point) {
            push_visible_npr_fragment(&mut fragments, kind, start, end, viewport, min_segment_px);
        }
    }

    fragments
}

fn push_visible_npr_fragment(
    fragments: &mut Vec<NprLineFragment>,
    kind: NprLineKind,
    start: Vec2,
    end: Vec2,
    viewport: &Viewport,
    min_segment_px: f32,
) {
    if screen_segment_length_px(start, end, viewport) >= min_segment_px.max(0.5) {
        fragments.push(NprLineFragment {
            kind,
            p0: start,
            p1: end,
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
            let Some(face) = visibility.face_id[index] else {
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
    feature: bool,
) -> Option<NprLineKind> {
    if boundary && settings.boundary {
        Some(NprLineKind::Boundary)
    } else if silhouette && settings.silhouette {
        Some(NprLineKind::Silhouette)
    } else if feature && settings.feature {
        Some(NprLineKind::Feature)
    } else {
        None
    }
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
        NprLineKind::Feature,
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
    endpoint_quant_px: f32,
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
            k0: npr_point_key(fragment.p0, viewport, endpoint_quant_px),
            k1: npr_point_key(fragment.p1, viewport, endpoint_quant_px),
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
            let points = walk_npr_path(&nodes, &adjacency, &mut visited, *endpoint);
            push_npr_stroke_path(&mut paths, kind, points, viewport, simplify_px);
        }
    }

    for fragment_index in 0..nodes.len() {
        if visited[fragment_index] {
            continue;
        }
        let points = walk_npr_path(
            &nodes,
            &adjacency,
            &mut visited,
            NprFragmentEndpoint {
                fragment_index,
                endpoint: 0,
            },
        );
        push_npr_stroke_path(&mut paths, kind, points, viewport, simplify_px);
    }

    paths
}

fn walk_npr_path(
    nodes: &[NprPathFragment],
    adjacency: &BTreeMap<(i32, i32), Vec<NprFragmentEndpoint>>,
    visited: &mut [bool],
    start: NprFragmentEndpoint,
) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut current = start;
    let mut guard = 0usize;

    while !visited[current.fragment_index] && guard < 20_000 {
        guard += 1;
        visited[current.fragment_index] = true;
        let node = nodes[current.fragment_index];
        let (first, second, next_key) = if current.endpoint == 0 {
            (node.fragment.p0, node.fragment.p1, node.k1)
        } else {
            (node.fragment.p1, node.fragment.p0, node.k0)
        };
        if points.is_empty() {
            points.push(first);
        }
        points.push(second);

        let Some(next) = adjacency
            .get(&next_key)
            .and_then(|entries| entries.iter().find(|entry| !visited[entry.fragment_index]))
            .copied()
        else {
            break;
        };
        current = next;
    }

    points
}

fn push_npr_stroke_path(
    paths: &mut Vec<NprStrokePath>,
    kind: NprLineKind,
    points: Vec<Vec2>,
    viewport: &Viewport,
    simplify_px: f32,
) {
    let points = simplify_npr_path(&points, viewport, simplify_px);
    if points.len() > 1 {
        paths.push(NprStrokePath { kind, points });
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

fn append_npr_styled_path_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    path: &NprStrokePath,
    path_index: usize,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    if path.points.len() < 2 {
        return;
    }

    let passes = settings.passes.min(8);
    let base_width = settings.width_px * npr_kind_width_multiplier(path.kind);
    let base_jitter = settings.path_jitter_px * npr_kind_jitter_multiplier(path.kind);
    let path_seed = path_index as u64;

    for pass in 0..passes {
        let pass_jitter = base_jitter * npr_pass_jitter_multiplier(passes, pass);
        let width_mul = npr_pass_width_multiplier(passes, pass);
        let color = npr_pass_color(settings.ink_color, passes, pass);
        for point_index in 1..path.points.len() {
            if deterministic_noise(
                settings.seed,
                path_seed,
                pass as u64,
                101 + point_index as u64,
            ) < settings.dropout
            {
                continue;
            }

            let p0 = jitter_npr_path_point(
                &path.points,
                point_index - 1,
                pass_jitter,
                settings.seed + pass as u64 * 11,
                viewport,
            );
            let p1 = jitter_npr_path_point(
                &path.points,
                point_index,
                pass_jitter,
                settings.seed + pass as u64 * 11 + 3,
                viewport,
            );
            let t = point_index as f32 / (path.points.len() - 1) as f32;
            let width_noise = deterministic_signed_noise(
                settings.seed,
                path_seed,
                pass as u64,
                503 + point_index as u64,
            );
            let width_px = (base_width * width_mul * npr_taper_multiplier(t, settings.taper)
                + width_noise * settings.width_jitter_px)
                .max(0.25);

            append_npr_line_segment_vertices(
                vertices,
                viewport,
                p0,
                p1,
                width_px,
                0.0,
                settings.overshoot_px,
                color,
            );
        }
    }
}

fn jitter_npr_path_point(
    points: &[Vec2],
    index: usize,
    amount_px: f32,
    seed: u64,
    viewport: &Viewport,
) -> Vec2 {
    if amount_px <= 0.0 {
        return points[index];
    }

    let prev = points[index.saturating_sub(1)];
    let next = points[(index + 1).min(points.len() - 1)];
    let tx = (next.x - prev.x) * viewport.half_width;
    let ty = (next.y - prev.y) * viewport.half_height;
    let length = (tx * tx + ty * ty).sqrt();
    if length <= f32::EPSILON {
        return points[index];
    }

    let normal = Vec2::new(-ty / length, tx / length);
    let point = points[index];
    let noise = deterministic_signed_noise(
        seed,
        index as u64,
        ((point.x.abs() * 1000.0) as u64).wrapping_add((point.y.abs() * 1000.0) as u64),
        919,
    );
    let px = point.x * viewport.half_width + normal.x * noise * amount_px;
    let py = point.y * viewport.half_height + normal.y * noise * amount_px;
    Vec2::new(px / viewport.half_width, py / viewport.half_height)
}

fn npr_kind_width_multiplier(kind: NprLineKind) -> f32 {
    match kind {
        NprLineKind::Silhouette => 1.0,
        NprLineKind::Feature => 0.72,
        NprLineKind::Boundary => 0.82,
    }
}

fn npr_kind_jitter_multiplier(kind: NprLineKind) -> f32 {
    match kind {
        NprLineKind::Feature => 0.8,
        NprLineKind::Silhouette | NprLineKind::Boundary => 1.0,
    }
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

fn npr_pass_color(color: ColorRgba, passes: u8, pass: u8) -> ColorRgba {
    let alpha = if passes >= 3 {
        0.18
    } else if passes == 2 {
        if pass == 0 { 0.28 } else { 0.75 }
    } else {
        0.92
    };
    ColorRgba::new(color.r, color.g, color.b, (color.a * alpha).clamp(0.0, 1.0))
}

fn npr_taper_multiplier(t: f32, taper: f32) -> f32 {
    let endpoint_weight = (t.min(1.0 - t) * 2.0).clamp(0.0, 1.0);
    1.0 - taper.clamp(0.0, 1.0) * (1.0 - endpoint_weight.max(0.35))
}

fn append_npr_line_segment_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    start: Vec2,
    end: Vec2,
    width_px: f32,
    jitter_px: f32,
    overshoot_px: f32,
    color: ColorRgba,
) {
    let dx_px = (end.x - start.x) * viewport.half_width;
    let dy_px = (end.y - start.y) * viewport.half_height;
    let length = (dx_px * dx_px + dy_px * dy_px).sqrt();
    if length <= f32::EPSILON {
        return;
    }
    let dir_px = Vec2::new(dx_px / length, dy_px / length);
    let normal_px = Vec2::new(-dir_px.y, dir_px.x);
    let half_width = width_px * 0.5;
    let start_px = Vec2::new(
        start.x * viewport.half_width - dir_px.x * overshoot_px + normal_px.x * jitter_px,
        start.y * viewport.half_height - dir_px.y * overshoot_px + normal_px.y * jitter_px,
    );
    let end_px = Vec2::new(
        end.x * viewport.half_width + dir_px.x * overshoot_px + normal_px.x * jitter_px,
        end.y * viewport.half_height + dir_px.y * overshoot_px + normal_px.y * jitter_px,
    );
    let offset = Vec2::new(normal_px.x * half_width, normal_px.y * half_width);
    let a = Vec2::new(
        (start_px.x + offset.x) / viewport.half_width,
        (start_px.y + offset.y) / viewport.half_height,
    );
    let b = Vec2::new(
        (end_px.x + offset.x) / viewport.half_width,
        (end_px.y + offset.y) / viewport.half_height,
    );
    let c = Vec2::new(
        (end_px.x - offset.x) / viewport.half_width,
        (end_px.y - offset.y) / viewport.half_height,
    );
    let d = Vec2::new(
        (start_px.x - offset.x) / viewport.half_width,
        (start_px.y - offset.y) / viewport.half_height,
    );
    push_quad(vertices, a, b, c, d, color);
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
    fn npr_line_segment_uses_pixel_width() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let mut vertices = Vec::new();
        append_npr_line_segment_vertices(
            &mut vertices,
            &viewport,
            Vec2::new(-0.5, 0.0),
            Vec2::new(0.5, 0.0),
            4.0,
            0.0,
            0.0,
            ColorRgba::WHITE,
        );
        assert_eq!(vertices.len(), 6);
        let height_ndc = (vertices[0].position[1] - vertices[2].position[1]).abs();
        assert!((height_ndc - (4.0 / viewport.half_height)).abs() < 0.0001);
    }

    #[test]
    fn npr_fragments_join_into_quantized_stroke_path() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let fragments = vec![
            NprLineFragment {
                kind: NprLineKind::Feature,
                p0: Vec2::new(-0.2, 0.0),
                p1: Vec2::new(0.0, 0.0),
            },
            NprLineFragment {
                kind: NprLineKind::Feature,
                p0: Vec2::new(0.002, 0.001),
                p1: Vec2::new(0.2, 0.0),
            },
        ];

        let paths = build_npr_stroke_paths(&fragments, &viewport, 2.5, 0.0);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].kind, NprLineKind::Feature);
        assert_eq!(paths[0].points.len(), 3);
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
            a: 0,
            b: 1,
            faces: vec![0],
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
            face_id: vec![Some(0); 16 * 16],
            face_visible: vec![true],
        };
        let hidden_visibility = NprFaceVisibilityBuffer {
            width: 16,
            height: 16,
            face_id: vec![Some(1); 16 * 16],
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
            width_jitter_px: 0.0,
            path_jitter_px: 0.0,
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
                width_jitter_px: 0.0,
                path_jitter_px: 0.0,
                overshoot_px: 0.0,
                dropout: 0.0,
                passes: 1,
                ..amigo_render_api::NprLineSettings3d::default()
            },
        );
        assert!(!silhouette_vertices.is_empty());
    }
}
