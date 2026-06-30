use amigo_math::{Vec2, Vec3};

use crate::renderer::{
    CachedMeshGeometry3d, MeshEdge3d, MeshTriangle3d, NprEdgeSampleResult3d,
    NprFaceVisibilityBuffer, NprLineFragment, NprLineKind, ProjectedPoint, Viewport, dot,
    normalize, screen_segment_length_px,
};

use super::{
    NprLineCandidateTraits, deterministic_noise, npr_line_family_role_for_kind,
    npr_line_kind_enabled, npr_min_screen_length_px_with_traits,
    npr_preferred_stroke_length_px_with_traits, npr_technical_detail_keep_with_traits,
};
use super::types::NprRejectedTechnicalCandidate;

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
    let rejected_count = chunk_results
        .iter()
        .map(|(_, result)| result.rejected_technical.len())
        .sum();
    let mut rejected_technical = Vec::with_capacity(rejected_count);
    for (_, result) in chunk_results {
        fragments.extend(result.fragments);
        rejected_technical.extend(result.rejected_technical);
    }
    NprEdgeSampleResult3d {
        fragments,
        visible_edges,
        rejected_technical,
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
    let mut rejected_technical = Vec::new();

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
        if !npr_line_kind_enabled(kind, settings) {
            continue;
        }
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
        let min_screen_length_px = npr_edge_min_screen_length_px(settings, kind, edge, triangles);
        if screen_length < min_screen_length_px {
            continue;
        }
        let candidate_importance = npr_edge_candidate_importance(
            settings,
            edge,
            triangles,
            kind,
            screen_length,
            face_view_alignment,
            projected_avg_depth(a, b),
        );
        let material_detail =
            npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids);
        let material_seam = edge.material_seam;
        if !npr_edge_survives_author_policy(
            settings,
            edge,
            kind,
            candidate_importance,
            material_detail,
        ) {
            if npr_kind_is_technical_detail(kind) {
                rejected_technical.push(NprRejectedTechnicalCandidate {
                    source_edge_id: edge.edge_id,
                    kind,
                    candidate_importance,
                    p0: a.position,
                    p1: b.position,
                });
            }
            continue;
        }
        visible_edges += 1;

        let require_outer_contour = npr_requires_screen_outer_contour(settings, kind);
        fragments.extend(visible_npr_fragments_for_edge(
            visibility,
            edge,
            kind,
            a,
            b,
            viewport,
            min_screen_length_px,
            candidate_importance,
            material_detail,
            material_seam,
            require_outer_contour,
        ));
    }

    NprEdgeSampleResult3d {
        fragments,
        visible_edges,
        rejected_technical,
    }
}

pub(crate) fn npr_edge_min_screen_length_px(
    settings: &amigo_render_api::NprLineSettings3d,
    kind: NprLineKind,
    edge: &MeshEdge3d,
    triangles: &[MeshTriangle3d],
) -> f32 {
    let material_detail =
        npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids);
    let material_seam = edge.material_seam;
    let base = npr_min_screen_length_px_with_traits(
        kind,
        NprLineCandidateTraits {
            technical_detail: npr_kind_is_technical_detail(kind),
            material_detail,
            material_seam,
        },
        settings,
    )
    .max(0.0);
    if material_detail {
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

fn npr_edge_candidate_importance(
    settings: &amigo_render_api::NprLineSettings3d,
    edge: &MeshEdge3d,
    triangles: &[MeshTriangle3d],
    kind: NprLineKind,
    screen_length: f32,
    face_view_alignment: &[f32],
    avg_depth: f32,
) -> f32 {
    if !npr_kind_is_technical_detail(kind) {
        return 1.0;
    }

    let material_detail =
        npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids);
    let material_seam = edge.material_seam;
    let role = npr_line_family_role_for_kind(kind, settings);
    let traits = NprLineCandidateTraits {
        technical_detail: true,
        material_detail,
        material_seam,
    };
    let detail_keep = npr_technical_detail_keep_with_traits(kind, traits, settings).clamp(0.0, 1.0);
    let angle_score = npr_edge_angle_score(settings, edge, triangles);
    let length_span = npr_preferred_stroke_length_px_with_traits(kind, traits, settings)
        .max(settings.min_screen_length_px * 6.0)
        .max(1.0);
    let length_score = (screen_length / length_span).clamp(0.0, 1.0);
    let view_score = edge
        .faces
        .iter()
        .copied()
        .filter_map(|face| face_view_alignment.get(face).copied())
        .map(|value| 1.0 - value.clamp(0.0, 1.0))
        .fold(0.5, f32::max);
    let depth_score = (1.0 - avg_depth.abs() * 0.12).clamp(0.35, 1.0);
    let kind_bias = match kind {
        NprLineKind::Crease => 0.20,
        NprLineKind::Seam => 0.16,
        NprLineKind::Feature => 0.12,
        _ => 0.0,
    };
    let role_bias = match role {
        amigo_render_api::NprLineFamilyRole3d::ClothFold => 0.10 + length_score * 0.10,
        amigo_render_api::NprLineFamilyRole3d::DetailInk => {
            if material_detail {
                0.12
            } else {
                -0.04
            }
        }
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => {
            if material_seam {
                0.12
            } else {
                -0.08
            }
        }
        amigo_render_api::NprLineFamilyRole3d::OuterContour => 0.0,
        amigo_render_api::NprLineFamilyRole3d::ShadowHatch
        | amigo_render_api::NprLineFamilyRole3d::ContactShadow
        | amigo_render_api::NprLineFamilyRole3d::Generic => 0.0,
    };
    let readability_penalty = if npr_edge_uses_character_readability(settings) {
        match kind {
            NprLineKind::Crease | NprLineKind::Seam => 0.16,
            NprLineKind::Feature => 0.22,
            _ => 0.0,
        }
    } else {
        0.0
    };
    (
        detail_keep * 0.30
            + kind_bias
            + role_bias
            + length_score * 0.32
            + angle_score * 0.20
            + view_score * 0.10
            + depth_score * 0.04
            + if material_detail { 0.20 } else { 0.0 }
            - readability_penalty
    )
    .clamp(0.0, 1.0)
}

fn npr_edge_survives_author_policy(
    settings: &amigo_render_api::NprLineSettings3d,
    edge: &MeshEdge3d,
    kind: NprLineKind,
    keep_probability: f32,
    material_detail: bool,
) -> bool {
    if !npr_kind_is_technical_detail(kind) {
        return true;
    }

    let traits = NprLineCandidateTraits {
        technical_detail: true,
        material_detail,
        material_seam: edge.material_seam,
    };
    if npr_technical_detail_keep_with_traits(kind, traits, settings) >= 1.0
        && !npr_edge_uses_character_readability(settings)
    {
        return true;
    }
    if npr_edge_uses_character_readability(settings)
        && keep_probability < npr_edge_author_keep_floor(settings, kind, material_detail)
    {
        return false;
    }

    deterministic_noise(
        settings.seed,
        edge.edge_id,
        npr_line_kind_seed(kind),
        701,
    ) <= keep_probability
}

fn npr_edge_author_keep_floor(
    settings: &amigo_render_api::NprLineSettings3d,
    kind: NprLineKind,
    material_detail: bool,
) -> f32 {
    let role = npr_line_family_role_for_kind(kind, settings);
    let base = match role {
        amigo_render_api::NprLineFamilyRole3d::ClothFold => 0.34,
        amigo_render_api::NprLineFamilyRole3d::DetailInk => 0.42,
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => 0.38,
        amigo_render_api::NprLineFamilyRole3d::ShadowHatch
        | amigo_render_api::NprLineFamilyRole3d::ContactShadow => 0.40,
        amigo_render_api::NprLineFamilyRole3d::OuterContour
        | amigo_render_api::NprLineFamilyRole3d::Generic => match kind {
            NprLineKind::Feature => 0.44,
            NprLineKind::Crease | NprLineKind::Seam => 0.38,
            _ => 0.0,
        },
    };
    if material_detail {
        (base - 0.12_f32).max(0.0)
    } else {
        base
    }
}

fn npr_edge_uses_character_readability(
    settings: &amigo_render_api::NprLineSettings3d,
) -> bool {
    settings.pipeline.candidate_strategy == amigo_render_api::NprCandidateStrategy3d::CharacterSemantic
        || matches!(
            settings.pipeline.budget_strategy,
            amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority
                | amigo_render_api::NprBudgetStrategy3d::CharacterReadability
        )
}

fn npr_kind_is_technical_detail(kind: NprLineKind) -> bool {
    matches!(
        kind,
        NprLineKind::Crease | NprLineKind::Seam | NprLineKind::Feature
    )
}

fn npr_edge_angle_score(
    settings: &amigo_render_api::NprLineSettings3d,
    edge: &MeshEdge3d,
    triangles: &[MeshTriangle3d],
) -> f32 {
    if edge.faces.len() < 2 {
        return 1.0;
    }
    let threshold = settings.feature_angle_degrees.max(1.0);
    let Some(left) = triangles.get(edge.faces[0]) else {
        return 0.5;
    };
    let Some(right) = triangles.get(edge.faces[1]) else {
        return 0.5;
    };
    let raw_angle = edge_angle_degrees(left.normal, right.normal);
    let raw = (raw_angle - threshold) / threshold.max(1.0);
    raw.clamp(0.0, 1.0)
}

fn projected_avg_depth(a: ProjectedPoint, b: ProjectedPoint) -> f32 {
    (a.depth + b.depth) * 0.5
}

fn npr_line_kind_seed(kind: NprLineKind) -> u64 {
    match kind {
        NprLineKind::Boundary => 11,
        NprLineKind::Silhouette => 17,
        NprLineKind::Crease => 19,
        NprLineKind::Feature => 23,
        NprLineKind::Seam => 29,
        NprLineKind::Contact => 31,
    }
}

pub(crate) fn visible_npr_fragments_for_edge(
    visibility: &NprFaceVisibilityBuffer,
    edge: &MeshEdge3d,
    kind: NprLineKind,
    a: ProjectedPoint,
    b: ProjectedPoint,
    viewport: &Viewport,
    min_segment_px: f32,
    candidate_importance: f32,
    material_detail: bool,
    material_seam: bool,
    require_outer_contour: bool,
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
            && sample_npr_owned_face(visibility, point, depth, edge)
            && (!require_outer_contour
                || sample_npr_outer_contour_exposure(visibility, point, a.position, b.position));

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
                    candidate_importance,
                    material_detail,
                    material_seam,
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
                candidate_importance,
                material_detail,
                material_seam,
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
    candidate_importance: f32,
    material_detail: bool,
    material_seam: bool,
) {
    if screen_segment_length_px(start, end, viewport) >= min_segment_px.max(0.5) {
        let tangent = normalize_screen_vector(start, end, viewport);
        fragments.push(NprLineFragment {
            source_edge_id,
            kind,
            candidate_importance,
            technical_detail: npr_kind_is_technical_detail(kind),
            material_detail,
            material_seam,
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

fn sample_npr_outer_contour_exposure(
    visibility: &NprFaceVisibilityBuffer,
    point: Vec2,
    start: Vec2,
    end: Vec2,
) -> bool {
    let dx_px = (end.x - start.x) * visibility.width as f32 * 0.5;
    let dy_px = -(end.y - start.y) * visibility.height as f32 * 0.5;
    let length = (dx_px * dx_px + dy_px * dy_px).sqrt();
    if length <= f32::EPSILON {
        return false;
    }

    let normal_x = -dy_px / length;
    let normal_y = dx_px / length;
    [3.0_f32, 6.0]
        .into_iter()
        .any(|offset_px| {
            let offset = Vec2::new(
                normal_x * offset_px * 2.0 / visibility.width as f32,
                -normal_y * offset_px * 2.0 / visibility.height as f32,
            );
            let left_has_face = sample_npr_any_face(
                visibility,
                Vec2::new(point.x + offset.x, point.y + offset.y),
            );
            let right_has_face = sample_npr_any_face(
                visibility,
                Vec2::new(point.x - offset.x, point.y - offset.y),
            );
            left_has_face != right_has_face
        })
}

fn sample_npr_any_face(visibility: &NprFaceVisibilityBuffer, point: Vec2) -> bool {
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
            if visibility.face_id[index] != usize::MAX {
                return true;
            }
        }
    }

    false
}

fn npr_requires_screen_outer_contour(
    settings: &amigo_render_api::NprLineSettings3d,
    kind: NprLineKind,
) -> bool {
    kind == NprLineKind::Silhouette
        && npr_line_family_role_for_kind(kind, settings)
            == amigo_render_api::NprLineFamilyRole3d::OuterContour
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
