use amigo_math::{Vec2, Vec3};

use crate::renderer::{
    CachedMeshGeometry3d, MeshEdge3d, MeshTriangle3d, NprEdgeSampleResult3d,
    NprFaceVisibilityBuffer, NprLineFragment, NprLineKind, ProjectedPoint, Viewport, dot,
    normalize, screen_segment_length_px,
};

use super::types::NprRejectedTechnicalCandidate;
use super::{
    NprLineCandidateTraits, deterministic_noise, npr_cpu_line_selection_profile,
    npr_line_family_role_for_kind, npr_line_kind_enabled, npr_min_screen_length_px_with_traits,
    npr_preferred_stroke_length_px_with_traits, npr_technical_detail_keep_with_traits,
};

#[derive(Debug, Clone, Copy)]
struct NprMeshComplexityProfile3d {
    pressure: f32,
    technical_min_length_multiplier: f32,
    boundary_min_length_multiplier: f32,
    technical_keep_scale: f32,
    technical_keep_floor_boost: f32,
    require_boundary_outer_contour: bool,
}

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
    let complexity = npr_mesh_complexity_profile(settings, geometry.edges(), viewport);
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
            complexity,
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
                        complexity,
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
    complexity: NprMeshComplexityProfile3d,
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
        let min_screen_length_px = npr_edge_min_screen_length_px_with_complexity(
            settings, kind, edge, triangles, complexity,
        );
        if screen_length < min_screen_length_px {
            continue;
        }
        let mut candidate_importance = npr_edge_candidate_importance(
            settings,
            edge,
            triangles,
            kind,
            screen_length,
            a.position,
            b.position,
            face_view_alignment,
            projected_avg_depth(a, b),
        );
        let material_detail =
            npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids);
        candidate_importance = npr_complexity_adjusted_candidate_importance(
            settings,
            complexity,
            kind,
            candidate_importance,
            material_detail,
            edge.material_seam,
        );
        let material_seam = edge.material_seam;
        if !npr_edge_survives_author_policy(
            settings,
            edge,
            kind,
            candidate_importance,
            material_detail,
            complexity,
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

        let require_outer_contour = npr_requires_screen_outer_contour(settings, kind)
            || npr_complexity_requires_outer_contour(complexity, kind);
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

fn npr_mesh_complexity_profile(
    settings: &amigo_render_api::NprLineSettings3d,
    edges: &[MeshEdge3d],
    viewport: &Viewport,
) -> NprMeshComplexityProfile3d {
    let selection = npr_cpu_line_selection_profile(settings);
    if edges.is_empty() {
        return NprMeshComplexityProfile3d {
            pressure: 0.0,
            technical_min_length_multiplier: 1.0,
            boundary_min_length_multiplier: 1.0,
            technical_keep_scale: 1.0,
            technical_keep_floor_boost: 0.0,
            require_boundary_outer_contour: false,
        };
    }

    let edge_count = edges.len() as f32;
    let viewport_size = viewport.size();
    let viewport_area = (viewport_size.x * viewport_size.y).max(1.0);
    let edges_per_10k_px = edge_count / (viewport_area / 10_000.0).max(1.0);
    let high_density_pressure = ((edges_per_10k_px - selection.dense_edge_start_per_10k_px)
        / (selection.dense_edge_full_per_10k_px - selection.dense_edge_start_per_10k_px).max(1.0))
    .clamp(0.0, 1.0);
    let material_seam_ratio =
        edges.iter().filter(|edge| edge.material_seam).count() as f32 / edge_count;
    let boundary_ratio =
        edges.iter().filter(|edge| edge.faces.len() == 1).count() as f32 / edge_count;
    let seam_pressure = ((material_seam_ratio - selection.dense_material_seam_start_ratio)
        / (selection.dense_material_seam_full_ratio - selection.dense_material_seam_start_ratio)
            .max(0.001))
    .clamp(0.0, 1.0);
    let boundary_pressure = ((boundary_ratio - selection.dense_boundary_start_ratio)
        / (selection.dense_boundary_full_ratio - selection.dense_boundary_start_ratio).max(0.001))
    .clamp(0.0, 1.0);
    let pressure = high_density_pressure
        .max(seam_pressure * selection.dense_seam_pressure_weight)
        .max(boundary_pressure * selection.dense_boundary_pressure_weight)
        .clamp(0.0, 1.0);

    NprMeshComplexityProfile3d {
        pressure,
        technical_min_length_multiplier: 1.0
            + pressure * selection.dense_technical_min_length_boost,
        boundary_min_length_multiplier: 1.0
            + boundary_pressure * selection.dense_boundary_min_length_boost,
        technical_keep_scale: 1.0 - pressure * selection.dense_technical_keep_scale_drop,
        technical_keep_floor_boost: pressure * selection.dense_keep_floor_boost,
        require_boundary_outer_contour: boundary_pressure
            > selection.dense_boundary_outer_contour_threshold
            || pressure > selection.dense_pressure_outer_contour_threshold,
    }
}

fn npr_complexity_adjusted_candidate_importance(
    settings: &amigo_render_api::NprLineSettings3d,
    complexity: NprMeshComplexityProfile3d,
    kind: NprLineKind,
    candidate_importance: f32,
    material_detail: bool,
    material_seam: bool,
) -> f32 {
    if !npr_kind_is_technical_detail(kind) || complexity.pressure <= 0.0 {
        return candidate_importance;
    }

    let selection = npr_cpu_line_selection_profile(settings);
    let quality_relief = ((candidate_importance - selection.dense_quality_relief_start)
        / selection.dense_quality_relief_span.max(0.001))
    .clamp(0.0, 1.0);
    let protected_factor = if material_detail {
        selection.dense_material_detail_protection
    } else if quality_relief > 0.0 {
        1.0 - quality_relief * selection.dense_quality_relief_scale
    } else {
        1.0
    };
    let seam_penalty = if material_seam && !material_detail {
        complexity.pressure
            * (selection.dense_seam_penalty
                - quality_relief * selection.dense_seam_quality_relief_scale)
                .clamp(
                    selection.dense_seam_penalty_min,
                    selection
                        .dense_seam_penalty
                        .max(selection.dense_seam_penalty_min),
                )
    } else {
        0.0
    };
    let kind_penalty = match kind {
        NprLineKind::Seam => complexity.pressure * selection.dense_seam_penalty,
        NprLineKind::Feature => complexity.pressure * selection.dense_feature_penalty,
        NprLineKind::Crease => complexity.pressure * selection.dense_crease_penalty,
        _ => 0.0,
    } * protected_factor
        * (1.0 - quality_relief * selection.dense_quality_relief_penalty_scale);
    (candidate_importance - kind_penalty - seam_penalty).clamp(0.0, 1.0)
}

fn npr_edge_min_screen_length_px_with_complexity(
    settings: &amigo_render_api::NprLineSettings3d,
    kind: NprLineKind,
    edge: &MeshEdge3d,
    triangles: &[MeshTriangle3d],
    complexity: NprMeshComplexityProfile3d,
) -> f32 {
    let base = npr_edge_min_screen_length_px(settings, kind, edge, triangles);
    if npr_kind_is_technical_detail(kind) {
        let material_detail =
            npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids);
        let protected_multiplier = if material_detail {
            npr_cpu_line_selection_profile(settings).dense_material_detail_min_length_multiplier
        } else {
            1.0
        };
        base * (1.0
            + (complexity.technical_min_length_multiplier - 1.0).max(0.0) * protected_multiplier)
    } else if kind == NprLineKind::Boundary {
        base * complexity.boundary_min_length_multiplier
    } else {
        base
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
        base * npr_cpu_line_selection_profile(settings).material_detail_min_screen_length_multiplier
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
    p0: Vec2,
    p1: Vec2,
    face_view_alignment: &[f32],
    avg_depth: f32,
) -> f32 {
    if !npr_kind_is_technical_detail(kind) {
        return 1.0;
    }

    let material_detail =
        npr_edge_touches_material_ids(edge, triangles, &settings.ink_detail_material_ids);
    let material_seam = edge.material_seam;
    let profile = npr_cpu_line_selection_profile(settings);
    let role = npr_line_family_role_for_kind(kind, settings);
    let traits = NprLineCandidateTraits {
        technical_detail: true,
        material_detail,
        material_seam,
    };
    let detail_keep = npr_technical_detail_keep_with_traits(kind, traits, settings).clamp(0.0, 1.0);
    let angle_score = npr_edge_angle_score(settings, edge, triangles);
    let length_span = npr_preferred_stroke_length_px_with_traits(kind, traits, settings)
        .max(settings.min_screen_length_px * profile.candidate_length_span_min_screen_multiplier)
        .max(1.0);
    let length_score = (screen_length / length_span).clamp(0.0, 1.0);
    let local_readability_score = if npr_edge_uses_character_readability(settings) {
        npr_screen_region_readability_score(midpoint_vec2(p0, p1), kind, profile)
    } else {
        0.0
    };
    let view_score = edge
        .faces
        .iter()
        .copied()
        .filter_map(|face| face_view_alignment.get(face).copied())
        .map(|value| 1.0 - value.clamp(0.0, 1.0))
        .fold(0.5, f32::max);
    let depth_score = (1.0 - avg_depth.abs() * profile.candidate_depth_weight)
        .clamp(profile.candidate_depth_min_score, 1.0);
    let kind_bias = match kind {
        NprLineKind::Crease => profile.crease_importance,
        NprLineKind::Seam => profile.seam_importance,
        NprLineKind::Feature => profile.feature_importance,
        _ => 0.0,
    };
    let role_bias = match role {
        amigo_render_api::NprLineFamilyRole3d::ClothFold => {
            profile.cloth_fold_importance + length_score * profile.cloth_fold_length_weight
        }
        amigo_render_api::NprLineFamilyRole3d::DetailInk => {
            if material_detail {
                profile.detail_ink_material_base
            } else {
                profile.detail_ink_importance + length_score * profile.detail_ink_length_weight
            }
        }
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => {
            if material_seam {
                profile.material_cut_seam_base + length_score * profile.material_cut_length_weight
            } else {
                -profile.material_seam_penalty
            }
        }
        amigo_render_api::NprLineFamilyRole3d::OuterContour => 0.0,
        amigo_render_api::NprLineFamilyRole3d::ShadowHatch
        | amigo_render_api::NprLineFamilyRole3d::ContactShadow
        | amigo_render_api::NprLineFamilyRole3d::Generic => 0.0,
    };
    let readability_penalty = if npr_edge_uses_character_readability(settings) {
        let short_detail_penalty = (1.0 - length_score).clamp(0.0, 1.0);
        let base_penalty = match kind {
            NprLineKind::Crease => {
                profile.short_crease_base_penalty
                    + short_detail_penalty * profile.short_crease_penalty
            }
            NprLineKind::Seam => {
                profile.short_seam_base_penalty + short_detail_penalty * profile.short_seam_penalty
            }
            NprLineKind::Feature => {
                profile.short_feature_base_penalty
                    + short_detail_penalty * profile.short_feature_penalty
            }
            _ => 0.0,
        };
        let readable_region_relief = (local_readability_score
            * profile.readable_region_relief_scale)
            .clamp(0.0, profile.readable_region_penalty_relief);
        base_penalty
            * (1.0 - readable_region_relief)
            * if material_detail {
                profile.material_detail_penalty_scale
            } else {
                1.0
            }
    } else {
        0.0
    };
    (detail_keep * profile.detail_keep_importance_weight
        + kind_bias
        + role_bias
        + length_score * profile.length_weight
        + local_readability_score
        + angle_score * profile.angle_weight
        + view_score * profile.view_weight
        + depth_score * profile.depth_weight
        + if material_detail {
            profile.material_detail_bonus
        } else {
            0.0
        }
        - readability_penalty)
        .clamp(0.0, 1.0)
}

fn npr_edge_survives_author_policy(
    settings: &amigo_render_api::NprLineSettings3d,
    edge: &MeshEdge3d,
    kind: NprLineKind,
    keep_probability: f32,
    material_detail: bool,
    complexity: NprMeshComplexityProfile3d,
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
        && complexity.pressure <= 0.0
    {
        return true;
    }
    let profile = npr_cpu_line_selection_profile(settings);
    let keep_floor = npr_edge_author_keep_floor(settings, kind, material_detail)
        + npr_dense_keep_floor_boost(profile, complexity, material_detail);
    if keep_probability < keep_floor.clamp(0.0, profile.keep_floor_max.clamp(0.0, 1.0)) {
        return false;
    }

    let effective_keep_probability = npr_dense_effective_keep_probability(
        profile,
        complexity,
        keep_probability,
        material_detail,
    );

    deterministic_noise(settings.seed, edge.edge_id, npr_line_kind_seed(kind), 701)
        <= effective_keep_probability
}

fn npr_dense_keep_floor_boost(
    profile: amigo_render_api::NprLineSelectionProfile3d,
    complexity: NprMeshComplexityProfile3d,
    material_detail: bool,
) -> f32 {
    if material_detail {
        complexity.technical_keep_floor_boost
            * profile
                .dense_material_detail_keep_floor_boost_scale
                .clamp(0.0, 1.0)
    } else {
        complexity.technical_keep_floor_boost
    }
}

fn npr_dense_effective_keep_probability(
    profile: amigo_render_api::NprLineSelectionProfile3d,
    complexity: NprMeshComplexityProfile3d,
    keep_probability: f32,
    material_detail: bool,
) -> f32 {
    if material_detail {
        keep_probability
            * (1.0
                - (1.0 - complexity.technical_keep_scale)
                    * profile
                        .dense_material_detail_keep_scale_retention
                        .clamp(0.0, 1.0))
    } else {
        keep_probability * complexity.technical_keep_scale
    }
}

fn npr_edge_author_keep_floor(
    settings: &amigo_render_api::NprLineSettings3d,
    kind: NprLineKind,
    material_detail: bool,
) -> f32 {
    let role = npr_line_family_role_for_kind(kind, settings);
    let profile = npr_cpu_line_selection_profile(settings);
    let base = match role {
        amigo_render_api::NprLineFamilyRole3d::ClothFold => profile.cloth_fold_keep_floor,
        amigo_render_api::NprLineFamilyRole3d::DetailInk => profile.detail_ink_keep_floor,
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => profile.material_cut_keep_floor,
        amigo_render_api::NprLineFamilyRole3d::ShadowHatch => profile.shadow_hatch_keep_floor,
        amigo_render_api::NprLineFamilyRole3d::ContactShadow => profile.contact_shadow_keep_floor,
        amigo_render_api::NprLineFamilyRole3d::OuterContour
        | amigo_render_api::NprLineFamilyRole3d::Generic => match kind {
            NprLineKind::Feature => profile.generic_feature_keep_floor,
            NprLineKind::Crease | NprLineKind::Seam => profile.generic_crease_keep_floor,
            _ => 0.0,
        },
    };
    if material_detail {
        (base - profile.material_detail_keep_floor_relief.max(0.0)).max(0.0)
    } else {
        base
    }
}

fn npr_edge_uses_character_readability(settings: &amigo_render_api::NprLineSettings3d) -> bool {
    settings.pipeline.candidate_strategy
        == amigo_render_api::NprCandidateStrategy3d::CharacterSemantic
        || matches!(
            settings.pipeline.budget_strategy,
            amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority
                | amigo_render_api::NprBudgetStrategy3d::CharacterReadability
        )
}

fn npr_screen_region_readability_score(
    midpoint: Vec2,
    kind: NprLineKind,
    profile: amigo_render_api::NprLineSelectionProfile3d,
) -> f32 {
    let face_focus = ((midpoint.y - profile.readable_face_start_y)
        / profile.readable_face_height.max(0.001))
    .clamp(0.0, 1.0)
        * (1.0 - midpoint.x.abs() / profile.readable_face_half_width.max(0.001)).clamp(0.0, 1.0);
    let torso_focus = (1.0 - midpoint.x.abs() / profile.readable_torso_half_width.max(0.001))
        .clamp(0.0, 1.0)
        * (1.0
            - ((midpoint.y - profile.readable_torso_center_y).abs()
                / profile.readable_torso_half_height.max(0.001)))
        .clamp(0.0, 1.0);
    let hand_focus = ((midpoint.x.abs() - profile.readable_hand_start_x)
        / profile.readable_hand_width.max(0.001))
    .clamp(0.0, 1.0)
        * ((midpoint.y - profile.readable_hand_start_y) / profile.readable_hand_height.max(0.001))
            .clamp(0.0, 1.0);
    match kind {
        NprLineKind::Feature => {
            face_focus * profile.feature_face_bonus
                + torso_focus * profile.feature_torso_bonus
                + hand_focus * profile.feature_hand_bonus
        }
        NprLineKind::Crease => {
            torso_focus * profile.crease_torso_bonus
                + face_focus * profile.crease_face_bonus
                + hand_focus * profile.crease_hand_bonus
        }
        NprLineKind::Seam => {
            torso_focus * profile.seam_torso_bonus + hand_focus * profile.seam_hand_bonus
        }
        _ => 0.0,
    }
}

fn midpoint_vec2(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
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
    [3.0_f32, 6.0].into_iter().any(|offset_px| {
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
    matches!(kind, NprLineKind::Silhouette | NprLineKind::Boundary)
        && npr_line_family_role_for_kind(kind, settings)
            == amigo_render_api::NprLineFamilyRole3d::OuterContour
}

fn npr_complexity_requires_outer_contour(
    complexity: NprMeshComplexityProfile3d,
    kind: NprLineKind,
) -> bool {
    complexity.require_boundary_outer_contour && kind == NprLineKind::Boundary
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_edge(edge_id: u64, faces: Vec<usize>, material_seam: bool) -> MeshEdge3d {
        MeshEdge3d {
            edge_id,
            a: 0,
            b: 1,
            faces,
            material_seam,
        }
    }

    #[test]
    fn complexity_profile_stays_neutral_for_simple_mesh() {
        let viewport = Viewport::from_dimensions(1280.0, 720.0);
        let settings = amigo_render_api::NprLineSettings3d::default();
        let edges = (0..128)
            .map(|index| test_edge(index, vec![0, 1], false))
            .collect::<Vec<_>>();

        let profile = npr_mesh_complexity_profile(&settings, &edges, &viewport);

        assert_eq!(profile.pressure, 0.0);
        assert_eq!(profile.technical_min_length_multiplier, 1.0);
        assert!(!profile.require_boundary_outer_contour);
    }

    #[test]
    fn complexity_profile_restricts_dense_material_seam_meshes() {
        let viewport = Viewport::from_dimensions(100.0, 100.0);
        let settings = amigo_render_api::NprLineSettings3d::default();
        let edges = (0..2_000)
            .map(|index| test_edge(index, vec![0, 1], index % 2 == 0))
            .collect::<Vec<_>>();

        let profile = npr_mesh_complexity_profile(&settings, &edges, &viewport);

        assert!(profile.pressure > 0.8);
        assert!(profile.technical_min_length_multiplier > 2.0);
        assert!(profile.technical_keep_scale < 0.55);
    }

    #[test]
    fn complexity_profile_uses_dense_seam_pressure_weight() {
        let viewport = Viewport::from_dimensions(2000.0, 2000.0);
        let edges = (0..2_000)
            .map(|index| test_edge(index, vec![0, 1], index % 2 == 0))
            .collect::<Vec<_>>();
        let mut suppressed = amigo_render_api::NprLineSettings3d::default();
        suppressed
            .cpu_strategy_profile
            .line_selection
            .dense_seam_pressure_weight = 0.0;
        let mut emphasized = suppressed.clone();
        emphasized
            .cpu_strategy_profile
            .line_selection
            .dense_seam_pressure_weight = 1.0;

        let suppressed_profile = npr_mesh_complexity_profile(&suppressed, &edges, &viewport);
        let emphasized_profile = npr_mesh_complexity_profile(&emphasized, &edges, &viewport);

        assert!(emphasized_profile.pressure > suppressed_profile.pressure);
    }

    #[test]
    fn technical_min_length_uses_material_detail_protection_multiplier() {
        let edge = test_edge(1, vec![0, 1], false);
        let triangles = vec![
            MeshTriangle3d {
                indices: [0, 1, 2],
                normal: Vec3::new(0.0, 0.0, 1.0),
                material_id: Some(7),
            },
            MeshTriangle3d {
                indices: [1, 3, 2],
                normal: Vec3::new(0.45, 0.0, 0.89),
                material_id: Some(7),
            },
        ];
        let complexity = NprMeshComplexityProfile3d {
            pressure: 1.0,
            technical_min_length_multiplier: 4.0,
            boundary_min_length_multiplier: 1.0,
            technical_keep_scale: 0.4,
            technical_keep_floor_boost: 0.24,
            require_boundary_outer_contour: false,
        };
        let mut protected = amigo_render_api::NprLineSettings3d {
            min_screen_length_px: 2.0,
            ink_detail_material_ids: vec![7],
            ..amigo_render_api::NprLineSettings3d::default()
        };
        protected
            .cpu_strategy_profile
            .line_selection
            .dense_material_detail_min_length_multiplier = 0.25;
        let unprotected = amigo_render_api::NprLineSettings3d {
            min_screen_length_px: 2.0,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let protected_min = npr_edge_min_screen_length_px_with_complexity(
            &protected,
            NprLineKind::Feature,
            &edge,
            &triangles,
            complexity,
        );
        let unprotected_min = npr_edge_min_screen_length_px_with_complexity(
            &unprotected,
            NprLineKind::Feature,
            &edge,
            &triangles,
            complexity,
        );

        assert!(protected_min < unprotected_min);
    }

    #[test]
    fn edge_min_screen_length_uses_material_detail_multiplier() {
        let edge = test_edge(11, vec![0, 1], false);
        let triangles = vec![
            MeshTriangle3d {
                indices: [0, 1, 2],
                normal: Vec3::new(0.0, 0.0, 1.0),
                material_id: Some(9),
            },
            MeshTriangle3d {
                indices: [1, 3, 2],
                normal: Vec3::new(0.45, 0.0, 0.89),
                material_id: Some(9),
            },
        ];
        let mut settings = amigo_render_api::NprLineSettings3d {
            min_screen_length_px: 4.0,
            ink_detail_material_ids: vec![9],
            ..amigo_render_api::NprLineSettings3d::default()
        };
        settings
            .cpu_strategy_profile
            .line_selection
            .material_detail_min_screen_length_multiplier = 0.25;

        let protected =
            npr_edge_min_screen_length_px(&settings, NprLineKind::Feature, &edge, &triangles);
        settings.ink_detail_material_ids.clear();
        let unprotected =
            npr_edge_min_screen_length_px(&settings, NprLineKind::Feature, &edge, &triangles);

        assert!(protected < unprotected * 0.35);
    }

    #[test]
    fn complexity_adjustment_penalizes_unprotected_seams_more_than_material_details() {
        let profile = NprMeshComplexityProfile3d {
            pressure: 1.0,
            technical_min_length_multiplier: 3.0,
            boundary_min_length_multiplier: 1.0,
            technical_keep_scale: 0.4,
            technical_keep_floor_boost: 0.24,
            require_boundary_outer_contour: false,
        };
        let settings = amigo_render_api::NprLineSettings3d::default();

        let seam = npr_complexity_adjusted_candidate_importance(
            &settings,
            profile,
            NprLineKind::Seam,
            0.80,
            false,
            true,
        );
        let protected = npr_complexity_adjusted_candidate_importance(
            &settings,
            profile,
            NprLineKind::Seam,
            0.80,
            true,
            true,
        );

        assert!(seam < protected);
        assert!(protected < 0.80);
    }

    #[test]
    fn complexity_adjustment_preserves_high_quality_feature_candidates_better_than_weak_ones() {
        let profile = NprMeshComplexityProfile3d {
            pressure: 1.0,
            technical_min_length_multiplier: 3.0,
            boundary_min_length_multiplier: 1.0,
            technical_keep_scale: 0.4,
            technical_keep_floor_boost: 0.24,
            require_boundary_outer_contour: false,
        };
        let settings = amigo_render_api::NprLineSettings3d::default();

        let weak = npr_complexity_adjusted_candidate_importance(
            &settings,
            profile,
            NprLineKind::Feature,
            0.42,
            false,
            false,
        );
        let strong = npr_complexity_adjusted_candidate_importance(
            &settings,
            profile,
            NprLineKind::Feature,
            0.82,
            false,
            false,
        );

        assert!(strong > weak);
        assert!(strong > 0.70);
    }

    #[test]
    fn screen_region_readability_prefers_upper_center_features() {
        let profile = amigo_render_api::NprLineSelectionProfile3d::toriyama_readability();
        let upper_center = npr_screen_region_readability_score(
            Vec2::new(0.02, 0.58),
            NprLineKind::Feature,
            profile,
        );
        let lower_side = npr_screen_region_readability_score(
            Vec2::new(0.62, -0.68),
            NprLineKind::Feature,
            profile,
        );

        assert!(upper_center > lower_side);
        assert!(upper_center > 0.12);
    }

    #[test]
    fn edge_candidate_importance_rewards_readable_upper_body_features() {
        let settings = amigo_render_api::NprLineSettings3d {
            pipeline: amigo_render_api::NprPipelineStrategies3d {
                candidate_strategy: amigo_render_api::NprCandidateStrategy3d::CharacterSemantic,
                budget_strategy: amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority,
                ..amigo_render_api::NprPipelineStrategies3d::default()
            },
            line_families: vec![amigo_render_api::NprLineFamily3d {
                role: Some(amigo_render_api::NprLineFamilyRole3d::DetailInk),
                sources: vec![amigo_render_api::NprLineSource3d::Feature],
                technical_detail_preference: Some(1.0),
                preferred_stroke_length_px: Some(96.0),
                ..amigo_render_api::NprLineFamily3d::default()
            }],
            cpu_strategy_profile: amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let edge = MeshEdge3d {
            edge_id: 77,
            a: 0,
            b: 1,
            faces: vec![0, 1],
            material_seam: false,
        };
        let triangles = vec![
            MeshTriangle3d {
                indices: [0, 1, 2],
                normal: Vec3::new(0.0, 0.0, 1.0),
                material_id: None,
            },
            MeshTriangle3d {
                indices: [1, 3, 2],
                normal: Vec3::new(0.55, 0.0, 0.84),
                material_id: None,
            },
        ];
        let face_view_alignment = vec![0.45, 0.45];

        let upper = npr_edge_candidate_importance(
            &settings,
            &edge,
            &triangles,
            NprLineKind::Feature,
            32.0,
            Vec2::new(-0.04, 0.52),
            Vec2::new(0.04, 0.58),
            &face_view_alignment,
            0.0,
        );
        let lower = npr_edge_candidate_importance(
            &settings,
            &edge,
            &triangles,
            NprLineKind::Feature,
            32.0,
            Vec2::new(0.52, -0.68),
            Vec2::new(0.60, -0.62),
            &face_view_alignment,
            0.0,
        );

        assert!(upper > lower);
        assert!(upper > 0.55);
    }

    #[test]
    fn edge_candidate_importance_uses_detail_ink_profile_weights() {
        let mut low = amigo_render_api::NprLineSettings3d {
            pipeline: amigo_render_api::NprPipelineStrategies3d {
                candidate_strategy: amigo_render_api::NprCandidateStrategy3d::CharacterSemantic,
                budget_strategy: amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority,
                ..amigo_render_api::NprPipelineStrategies3d::default()
            },
            line_families: vec![amigo_render_api::NprLineFamily3d {
                role: Some(amigo_render_api::NprLineFamilyRole3d::DetailInk),
                sources: vec![amigo_render_api::NprLineSource3d::Feature],
                technical_detail_preference: Some(0.35),
                preferred_stroke_length_px: Some(96.0),
                ..amigo_render_api::NprLineFamily3d::default()
            }],
            cpu_strategy_profile: amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
            ..amigo_render_api::NprLineSettings3d::default()
        };
        low.cpu_strategy_profile
            .line_selection
            .detail_ink_length_weight = 0.0;
        low.cpu_strategy_profile
            .line_selection
            .detail_keep_importance_weight = 0.0;
        let mut high = low.clone();
        high.cpu_strategy_profile
            .line_selection
            .detail_ink_length_weight = 0.30;
        high.cpu_strategy_profile
            .line_selection
            .detail_keep_importance_weight = 0.45;
        let edge = MeshEdge3d {
            edge_id: 78,
            a: 0,
            b: 1,
            faces: vec![0, 1],
            material_seam: false,
        };
        let triangles = vec![
            MeshTriangle3d {
                indices: [0, 1, 2],
                normal: Vec3::new(0.0, 0.0, 1.0),
                material_id: None,
            },
            MeshTriangle3d {
                indices: [1, 3, 2],
                normal: Vec3::new(0.50, 0.0, 0.86),
                material_id: None,
            },
        ];
        let face_view_alignment = vec![0.50, 0.50];

        let low_score = npr_edge_candidate_importance(
            &low,
            &edge,
            &triangles,
            NprLineKind::Feature,
            48.0,
            Vec2::new(-0.04, 0.42),
            Vec2::new(0.04, 0.48),
            &face_view_alignment,
            0.0,
        );
        let high_score = npr_edge_candidate_importance(
            &high,
            &edge,
            &triangles,
            NprLineKind::Feature,
            48.0,
            Vec2::new(-0.04, 0.42),
            Vec2::new(0.04, 0.48),
            &face_view_alignment,
            0.0,
        );

        assert!(high_score > low_score + 0.20);
    }

    #[test]
    fn author_keep_floor_uses_profiled_shadow_and_material_detail_relief() {
        let mut settings = amigo_render_api::NprLineSettings3d {
            contact: true,
            line_families: vec![amigo_render_api::NprLineFamily3d {
                role: Some(amigo_render_api::NprLineFamilyRole3d::ContactShadow),
                sources: vec![amigo_render_api::NprLineSource3d::Contact],
                ..amigo_render_api::NprLineFamily3d::default()
            }],
            ..amigo_render_api::NprLineSettings3d::default()
        };
        settings
            .cpu_strategy_profile
            .line_selection
            .contact_shadow_keep_floor = 0.62;
        settings
            .cpu_strategy_profile
            .line_selection
            .material_detail_keep_floor_relief = 0.22;

        let plain = npr_edge_author_keep_floor(&settings, NprLineKind::Contact, false);
        let material_detail = npr_edge_author_keep_floor(&settings, NprLineKind::Contact, true);

        assert_eq!(plain, 0.62);
        assert!((material_detail - 0.40).abs() < 0.001);
    }

    #[test]
    fn dense_material_detail_keep_policy_uses_profiled_boost_and_retention() {
        let complexity = NprMeshComplexityProfile3d {
            pressure: 1.0,
            technical_min_length_multiplier: 2.0,
            boundary_min_length_multiplier: 1.0,
            technical_keep_scale: 0.25,
            technical_keep_floor_boost: 0.40,
            require_boundary_outer_contour: false,
        };
        let mut profile = amigo_render_api::NprLineSelectionProfile3d::default();
        profile.dense_material_detail_keep_floor_boost_scale = 0.25;
        profile.dense_material_detail_keep_scale_retention = 0.20;

        let material_floor_boost = npr_dense_keep_floor_boost(profile, complexity, true);
        let plain_floor_boost = npr_dense_keep_floor_boost(profile, complexity, false);
        let material_keep = npr_dense_effective_keep_probability(profile, complexity, 0.80, true);
        let plain_keep = npr_dense_effective_keep_probability(profile, complexity, 0.80, false);

        assert!((material_floor_boost - 0.10).abs() < 0.001);
        assert!((plain_floor_boost - 0.40).abs() < 0.001);
        assert!(material_keep > plain_keep);
    }

    #[test]
    fn edge_candidate_importance_keeps_lower_material_seams_below_readable_features() {
        let settings = amigo_render_api::NprLineSettings3d {
            pipeline: amigo_render_api::NprPipelineStrategies3d {
                candidate_strategy: amigo_render_api::NprCandidateStrategy3d::CharacterSemantic,
                budget_strategy: amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority,
                ..amigo_render_api::NprPipelineStrategies3d::default()
            },
            line_families: vec![
                amigo_render_api::NprLineFamily3d {
                    role: Some(amigo_render_api::NprLineFamilyRole3d::DetailInk),
                    sources: vec![amigo_render_api::NprLineSource3d::Feature],
                    technical_detail_preference: Some(1.0),
                    preferred_stroke_length_px: Some(96.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
                amigo_render_api::NprLineFamily3d {
                    role: Some(amigo_render_api::NprLineFamilyRole3d::MaterialCut),
                    sources: vec![amigo_render_api::NprLineSource3d::Seam],
                    technical_detail_preference: Some(0.15),
                    material_seam_preference: Some(0.12),
                    preferred_stroke_length_px: Some(120.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
            ],
            cpu_strategy_profile: amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let feature_edge = MeshEdge3d {
            edge_id: 88,
            a: 0,
            b: 1,
            faces: vec![0, 1],
            material_seam: false,
        };
        let seam_edge = MeshEdge3d {
            edge_id: 89,
            a: 0,
            b: 1,
            faces: vec![0, 1],
            material_seam: true,
        };
        let triangles = vec![
            MeshTriangle3d {
                indices: [0, 1, 2],
                normal: Vec3::new(0.0, 0.0, 1.0),
                material_id: None,
            },
            MeshTriangle3d {
                indices: [1, 3, 2],
                normal: Vec3::new(0.50, 0.0, 0.86),
                material_id: None,
            },
        ];
        let face_view_alignment = vec![0.50, 0.50];

        let readable_feature = npr_edge_candidate_importance(
            &settings,
            &feature_edge,
            &triangles,
            NprLineKind::Feature,
            38.0,
            Vec2::new(-0.05, 0.45),
            Vec2::new(0.06, 0.52),
            &face_view_alignment,
            0.0,
        );
        let lower_seam = npr_edge_candidate_importance(
            &settings,
            &seam_edge,
            &triangles,
            NprLineKind::Seam,
            38.0,
            Vec2::new(0.55, -0.78),
            Vec2::new(0.65, -0.72),
            &face_view_alignment,
            0.0,
        );

        assert!(readable_feature > lower_seam);
        assert!(lower_seam < 0.62);
    }

    #[test]
    fn boundary_outer_contour_requires_screen_exposure() {
        let settings = amigo_render_api::NprLineSettings3d::default();

        assert!(npr_requires_screen_outer_contour(
            &settings,
            NprLineKind::Boundary
        ));
    }
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
