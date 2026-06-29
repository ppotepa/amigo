use amigo_math::{Vec2, Vec3};

use crate::renderer::{
    CachedMeshGeometry3d, MeshEdge3d, MeshTriangle3d, NprEdgeSampleResult3d,
    NprFaceVisibilityBuffer, NprLineFragment, NprLineKind, ProjectedPoint, Viewport, dot,
    normalize, screen_segment_length_px,
};

pub(crate) fn collect_npr_edge_fragments_for_mesh(
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
    if worker_count <= 1 || geometry.edges().len() < 4096 {
        return collect_npr_edge_fragments_for_chunk(
            geometry.edges(),
            geometry.triangles(),
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

    let chunk_size = geometry.edges().len().div_ceil(worker_count).max(1);
    let mut chunk_results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (chunk_index, chunk) in geometry.edges().chunks(chunk_size).enumerate() {
            handles.push(scope.spawn(move || {
                (
                    chunk_index,
                    collect_npr_edge_fragments_for_chunk(
                        chunk,
                        geometry.triangles(),
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
    triangles: &[MeshTriangle3d],
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
        let Some(kind) = npr_line_kind_for_edge(
            settings, boundary, silhouette, crease, seam, suggestive, contact,
        ) else {
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
        let min_screen_length_px = npr_edge_min_screen_length_px(settings, edge, triangles);
        if screen_length < min_screen_length_px {
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
            min_screen_length_px,
        ));
    }

    NprEdgeSampleResult3d {
        fragments,
        visible_edges,
    }
}

pub(crate) fn npr_edge_min_screen_length_px(
    settings: &amigo_render_api::NprLineSettings3d,
    edge: &MeshEdge3d,
    triangles: &[MeshTriangle3d],
) -> f32 {
    let base = settings.min_screen_length_px.max(0.0);
    if npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids) {
        base * 0.55
    } else {
        base
    }
}

fn npr_edge_touches_material_ids(
    edge: &MeshEdge3d,
    triangles: &[MeshTriangle3d],
    material_ids: &[u32],
) -> bool {
    !material_ids.is_empty()
        && edge.faces.iter().copied().any(|face| {
            triangles
                .get(face)
                .and_then(|triangle| triangle.material_id)
                .is_some_and(|material_id| material_ids.contains(&material_id))
        })
}

pub(crate) fn visible_npr_fragments_for_edge(
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

pub(crate) fn npr_line_kind_for_edge(
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

fn edge_angle_degrees(left: Vec3, right: Vec3) -> f32 {
    dot(normalize(left), normalize(right))
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

fn screen_segment_is_sane(start: Vec2, end: Vec2) -> bool {
    [start, end].iter().all(|point| {
        point.x.is_finite() && point.y.is_finite() && point.x.abs() < 8.0 && point.y.abs() < 8.0
    })
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
