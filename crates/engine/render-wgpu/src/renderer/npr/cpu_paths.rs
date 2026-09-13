use std::collections::BTreeMap;

use amigo_math::Vec2;

use crate::renderer::{
    NprLineFragment, NprLineKind, NprStrokePath, Viewport, npr_breakup_bias_with_traits,
    npr_continuation_bias_with_traits, npr_line_family_role_for_kind,
    npr_min_stroke_length_px_with_traits, npr_preferred_stroke_length_px_with_traits,
    npr_stroke_join_gap_px_with_traits, npr_stroke_join_max_angle_degrees_with_traits,
    npr_technical_detail_keep_with_traits, screen_segment_length_px,
};

use super::{
    NprLineCandidateTraits, deterministic_noise, npr_cpu_line_selection_profile,
    npr_cpu_path_joining_profile,
};

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

#[cfg(test)]
pub(crate) fn build_npr_stroke_paths(
    fragments: &[NprLineFragment],
    viewport: &Viewport,
    endpoint_quant_px: f32,
    simplify_px: f32,
) -> Vec<NprStrokePath> {
    build_npr_stroke_paths_with_policy(fragments, viewport, endpoint_quant_px, simplify_px, None)
}

pub(crate) fn build_npr_stroke_paths_for_settings(
    fragments: &[NprLineFragment],
    viewport: &Viewport,
    settings: &amigo_render_api::NprLineSettings3d,
) -> Vec<NprStrokePath> {
    build_npr_stroke_paths_with_policy(
        fragments,
        viewport,
        settings.endpoint_snap_px,
        settings.path_simplify_px,
        Some(settings),
    )
}

fn build_npr_stroke_paths_with_policy(
    fragments: &[NprLineFragment],
    viewport: &Viewport,
    endpoint_quant_px: f32,
    simplify_px: f32,
    settings: Option<&amigo_render_api::NprLineSettings3d>,
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
            settings,
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
    settings: Option<&amigo_render_api::NprLineSettings3d>,
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
            let fragments = walk_npr_path(
                kind,
                &nodes,
                &adjacency,
                &mut visited,
                *endpoint,
                viewport,
                endpoint_snap_px,
                settings,
            );
            push_npr_stroke_path(&mut paths, kind, fragments, viewport, simplify_px, settings);
        }
    }

    for fragment_index in 0..nodes.len() {
        if visited[fragment_index] {
            continue;
        }
        let fragments = walk_npr_path(
            kind,
            &nodes,
            &adjacency,
            &mut visited,
            NprFragmentEndpoint {
                fragment_index,
                endpoint: 0,
            },
            viewport,
            endpoint_snap_px,
            settings,
        );
        push_npr_stroke_path(&mut paths, kind, fragments, viewport, simplify_px, settings);
    }

    paths
}

fn walk_npr_path(
    kind: NprLineKind,
    nodes: &[NprPathFragment],
    adjacency: &BTreeMap<(i32, i32), Vec<NprFragmentEndpoint>>,
    visited: &mut [bool],
    start: NprFragmentEndpoint,
    viewport: &Viewport,
    endpoint_snap_px: f32,
    settings: Option<&amigo_render_api::NprLineSettings3d>,
) -> Vec<NprLineFragment> {
    let mut path = Vec::new();
    let mut current = start;
    let mut guard = 0usize;
    let mut current_length_px = 0.0f32;

    while !visited[current.fragment_index] && guard < 20_000 {
        guard += 1;
        visited[current.fragment_index] = true;
        let node = nodes[current.fragment_index];
        let (fragment, next_key, next_point, entry_tangent) = if current.endpoint == 0 {
            (
                node.fragment,
                node.k1,
                node.fragment.p1,
                node.fragment.tangent1,
            )
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
                node.fragment.p0,
                mul_vec2(node.fragment.tangent0, -1.0),
            )
        };
        current_length_px += screen_segment_length_px(fragment.p0, fragment.p1, viewport);
        path.push(fragment);

        let Some(next) = best_npr_path_continuation(
            kind,
            nodes,
            adjacency,
            visited,
            next_key,
            next_point,
            entry_tangent,
            current_length_px,
            viewport,
            endpoint_snap_px,
            settings,
        ) else {
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
    settings: Option<&amigo_render_api::NprLineSettings3d>,
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
        let arc_lengths_px = npr_path_arc_lengths(&points, viewport);
        let length_px = arc_lengths_px.last().copied().unwrap_or(0.0);
        let sorted_source_edges = sorted_npr_source_edges(&source_edges);
        let path_id = stable_path_id(kind, &source_edges);
        let traits = NprLineCandidateTraits {
            technical_detail: fragments.iter().any(|fragment| fragment.technical_detail),
            material_detail: fragments.iter().any(|fragment| fragment.material_detail),
            material_seam: fragments.iter().any(|fragment| fragment.material_seam),
        };
        if let Some(settings) = settings {
            if !npr_path_survives_author_policy(
                kind,
                length_px,
                path_id,
                source_edges.len(),
                traits,
                settings,
            ) {
                return;
            }
        }
        let avg_depth = fragments
            .iter()
            .map(|fragment| fragment.avg_depth)
            .sum::<f32>()
            / fragments.len() as f32;
        let candidate_importance = fragments
            .iter()
            .map(|fragment| fragment.candidate_importance)
            .sum::<f32>()
            / fragments.len() as f32;
        let technical_detail = fragments.iter().all(|fragment| fragment.technical_detail);
        let material_detail = fragments.iter().any(|fragment| fragment.material_detail);
        let material_seam = fragments.iter().any(|fragment| fragment.material_seam);
        let average_point = npr_path_average_point(&points);
        paths.push(NprStrokePath {
            path_id,
            kind,
            arc_lengths_px,
            importance: npr_path_importance(
                settings,
                kind,
                avg_depth,
                candidate_importance,
                technical_detail,
                source_edges.len(),
                average_point,
            ),
            candidate_importance,
            technical_detail,
            material_detail,
            material_seam,
            closed: npr_path_is_closed(&points, viewport),
            points,
            source_edges,
            sorted_source_edges,
        });
    }
}

fn npr_path_survives_author_policy(
    kind: NprLineKind,
    length_px: f32,
    path_id: u64,
    source_edge_count: usize,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> bool {
    let min_length = npr_kind_min_stroke_length_px(kind, traits, settings);
    if length_px < min_length {
        return false;
    }

    if !npr_path_uses_character_readability(settings) || !npr_kind_is_technical_detail(kind) {
        return true;
    }

    let length_span = npr_preferred_stroke_length_px_with_traits(kind, traits, settings)
        .max(min_length.max(settings.min_screen_length_px) * 2.0)
        .max(1.0);
    let length_score = ((length_px - min_length) / length_span).clamp(0.0, 1.0);
    let role = npr_line_family_role_for_kind(kind, settings);
    let profile = npr_cpu_path_joining_profile(settings);
    if npr_path_is_unimportant_isolated_detail(
        profile,
        role,
        length_px,
        length_span,
        source_edge_count,
        traits,
    ) {
        return false;
    }
    let chain_bonus = ((source_edge_count.saturating_sub(1) as f32)
        * profile.survival_chain_bonus_per_edge)
        .clamp(0.0, profile.survival_chain_bonus_max);
    let continuation_bias = npr_continuation_bias_with_traits(kind, traits, settings);
    let breakup_bias = npr_breakup_bias_with_traits(kind, traits, settings);
    let role_bonus = match role {
        amigo_render_api::NprLineFamilyRole3d::ClothFold => {
            profile.survival_cloth_fold_base_bonus
                + ((source_edge_count.saturating_sub(1) as f32)
                    * profile.survival_cloth_fold_chain_bonus_per_edge)
                    .clamp(0.0, profile.survival_cloth_fold_chain_bonus_max)
        }
        amigo_render_api::NprLineFamilyRole3d::DetailInk => {
            if traits.material_detail {
                profile.survival_detail_material_bonus
            } else {
                profile.survival_detail_plain_bonus
            }
        }
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => {
            if traits.material_seam {
                profile.survival_material_cut_seam_bonus
            } else {
                profile.survival_material_cut_plain_bonus
            }
        }
        _ => 0.0,
    };
    let long_form_bonus = length_score * profile.survival_long_form_length_weight
        + ((source_edge_count.saturating_sub(1) as f32)
            * profile.survival_long_form_chain_bonus_per_edge)
            .clamp(0.0, profile.survival_long_form_chain_bonus_max);
    let keep_probability = (npr_technical_detail_keep_with_traits(kind, traits, settings)
        .clamp(0.0, 1.0)
        * profile.survival_trait_keep_weight
        + profile.survival_base_keep
        + length_score * profile.survival_length_weight
        + settings.line_confidence.clamp(0.0, 1.0) * profile.survival_confidence_weight
        + chain_bonus
        + role_bonus
        + long_form_bonus
        + continuation_bias * profile.survival_continuation_weight
        - breakup_bias * profile.survival_breakup_penalty)
        .clamp(0.0, 1.0);
    deterministic_noise(settings.seed, path_id, npr_line_kind_seed(kind), 911) <= keep_probability
}

fn npr_path_is_unimportant_isolated_detail(
    profile: amigo_render_api::NprPathJoiningProfile3d,
    role: amigo_render_api::NprLineFamilyRole3d,
    length_px: f32,
    preferred_length_px: f32,
    source_edge_count: usize,
    traits: NprLineCandidateTraits,
) -> bool {
    if traits.material_detail || traits.material_seam || source_edge_count > 1 {
        return false;
    }

    let short_ratio = match role {
        amigo_render_api::NprLineFamilyRole3d::DetailInk => profile.isolated_detail_short_ratio,
        amigo_render_api::NprLineFamilyRole3d::ClothFold => profile.isolated_cloth_fold_short_ratio,
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => {
            profile.isolated_material_cut_short_ratio
        }
        _ => return false,
    };
    length_px < preferred_length_px * short_ratio.clamp(0.0, 1.0)
}

fn npr_kind_min_stroke_length_px(
    kind: NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let base = npr_min_stroke_length_px_with_traits(kind, traits, settings).max(0.0);
    if base <= 0.0 {
        return 0.0;
    }
    let profile = npr_cpu_path_joining_profile(settings);

    let readability_multiplier = if settings.pipeline.budget_strategy
        == amigo_render_api::NprBudgetStrategy3d::CharacterReadability
    {
        profile.min_length_character_readability_multiplier
    } else {
        1.0
    };

    let kind_multiplier = match kind {
        NprLineKind::Silhouette => profile.min_length_silhouette_multiplier,
        NprLineKind::Boundary => profile.min_length_boundary_multiplier,
        NprLineKind::Contact => profile.min_length_contact_multiplier,
        NprLineKind::Crease => profile.min_length_crease_multiplier,
        NprLineKind::Seam => profile.min_length_seam_multiplier,
        NprLineKind::Feature => profile.min_length_feature_multiplier,
    };
    base * kind_multiplier.max(0.0) * readability_multiplier.max(0.0)
}

fn npr_path_uses_character_readability(settings: &amigo_render_api::NprLineSettings3d) -> bool {
    settings.pipeline.candidate_strategy
        == amigo_render_api::NprCandidateStrategy3d::CharacterSemantic
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

fn npr_point_key(point: Vec2, viewport: &Viewport, endpoint_quant_px: f32) -> (i32, i32) {
    let quant = endpoint_quant_px.max(0.5);
    (
        ((point.x * viewport.half_width) / quant).round() as i32,
        ((point.y * viewport.half_height) / quant).round() as i32,
    )
}

pub(crate) fn simplify_npr_path(
    points: &[Vec2],
    viewport: &Viewport,
    epsilon_px: f32,
) -> Vec<Vec2> {
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

pub(crate) fn npr_path_average_y(path: &NprStrokePath) -> f32 {
    path.points.iter().map(|point| point.y).sum::<f32>() / path.points.len() as f32
}

fn best_npr_path_continuation(
    kind: NprLineKind,
    nodes: &[NprPathFragment],
    adjacency: &BTreeMap<(i32, i32), Vec<NprFragmentEndpoint>>,
    visited: &[bool],
    join_key: (i32, i32),
    join_point: Vec2,
    entry_tangent: Vec2,
    current_length_px: f32,
    viewport: &Viewport,
    endpoint_snap_px: f32,
    settings: Option<&amigo_render_api::NprLineSettings3d>,
) -> Option<NprFragmentEndpoint> {
    let join_radius = settings
        .map(|value| {
            (npr_stroke_join_gap_px_with_traits(
                kind,
                NprLineCandidateTraits {
                    technical_detail: npr_kind_is_technical_detail(kind),
                    material_detail: false,
                    material_seam: false,
                },
                value,
            )
            .max(0.0)
                / endpoint_snap_px.max(0.5))
            .ceil() as i32
        })
        .unwrap_or(0)
        .max(0);
    let mut candidates = Vec::new();
    for y in -join_radius..=join_radius {
        for x in -join_radius..=join_radius {
            if let Some(entries) = adjacency.get(&(join_key.0 + x, join_key.1 + y)) {
                candidates.extend(
                    entries
                        .iter()
                        .copied()
                        .filter(|entry| !visited[entry.fragment_index]),
                );
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    candidates.into_iter().min_by(|left, right| {
        let left_score = npr_path_join_score(
            kind,
            nodes[left.fragment_index].fragment,
            left.endpoint,
            join_point,
            entry_tangent,
            current_length_px,
            viewport,
            settings,
        );
        let right_score = npr_path_join_score(
            kind,
            nodes[right.fragment_index].fragment,
            right.endpoint,
            join_point,
            entry_tangent,
            current_length_px,
            viewport,
            settings,
        );
        left_score
            .partial_cmp(&right_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn npr_path_join_score(
    kind: NprLineKind,
    fragment: NprLineFragment,
    endpoint: u8,
    join_point: Vec2,
    entry_tangent: Vec2,
    current_length_px: f32,
    viewport: &Viewport,
    settings: Option<&amigo_render_api::NprLineSettings3d>,
) -> f32 {
    let (start, tangent) = if endpoint == 0 {
        (fragment.p0, fragment.tangent0)
    } else {
        (fragment.p1, mul_vec2(fragment.tangent1, -1.0))
    };
    let gap = screen_segment_length_px(join_point, start, viewport);
    let tangent_dot =
        dot_vec2(normalize_vec2(entry_tangent), normalize_vec2(tangent)).clamp(-1.0, 1.0);
    let tangent_mismatch = 1.0 - tangent_dot;
    let angle_degrees = tangent_dot.acos().to_degrees();
    let join_midpoint = midpoint_vec2(join_point, start);
    let profile = settings
        .map(npr_cpu_path_joining_profile)
        .unwrap_or_default();
    let selection = settings
        .map(npr_cpu_line_selection_profile)
        .unwrap_or_default();
    let max_gap = settings
        .map(|value| {
            npr_stroke_join_gap_px_with_traits(
                kind,
                NprLineCandidateTraits {
                    technical_detail: fragment.technical_detail,
                    material_detail: fragment.material_detail,
                    material_seam: fragment.material_seam,
                },
                value,
            )
            .max(0.0)
        })
        .unwrap_or(0.0);
    let max_angle = settings
        .map(|value| {
            npr_stroke_join_max_angle_degrees_with_traits(
                kind,
                NprLineCandidateTraits {
                    technical_detail: fragment.technical_detail,
                    material_detail: fragment.material_detail,
                    material_seam: fragment.material_seam,
                },
                value,
            )
            .max(0.0)
        })
        .unwrap_or(180.0);
    let preferred_length = settings
        .map(|value| {
            npr_preferred_stroke_length_px_with_traits(
                kind,
                NprLineCandidateTraits {
                    technical_detail: fragment.technical_detail,
                    material_detail: fragment.material_detail,
                    material_seam: fragment.material_seam,
                },
                value,
            )
            .max(0.0)
        })
        .unwrap_or(0.0);
    let continuation_bias = settings
        .map(|value| {
            npr_continuation_bias_with_traits(
                kind,
                NprLineCandidateTraits {
                    technical_detail: fragment.technical_detail,
                    material_detail: fragment.material_detail,
                    material_seam: fragment.material_seam,
                },
                value,
            )
        })
        .unwrap_or(0.5);
    let breakup_bias = settings
        .map(|value| {
            npr_breakup_bias_with_traits(
                kind,
                NprLineCandidateTraits {
                    technical_detail: fragment.technical_detail,
                    material_detail: fragment.material_detail,
                    material_seam: fragment.material_seam,
                },
                value,
            )
        })
        .unwrap_or(0.5);
    let readable_detail_relax = if fragment.technical_detail {
        (npr_path_screen_region_readability_score(profile, selection, join_midpoint, kind)
            * profile.readable_detail_relax_multiplier
            + fragment.candidate_importance.clamp(0.0, 1.0)
                * profile.readable_detail_importance_relax)
            .clamp(0.0, profile.readable_detail_relax_max)
    } else {
        0.0
    };
    let preferred_bias = if preferred_length > 0.0 && current_length_px < preferred_length {
        -((preferred_length - current_length_px) / preferred_length).clamp(0.0, 1.0)
            * (profile.preferred_length_bias_base
                + continuation_bias * profile.continuation_bias_scale
                + readable_detail_relax * profile.readable_continuation_bonus)
    } else {
        0.0
    };
    let gap_weight = (profile.gap_weight_base + breakup_bias * profile.gap_weight_breakup_scale
        - continuation_bias * profile.gap_weight_continuation_scale
        - readable_detail_relax * profile.gap_weight_readable_relax_scale)
        .max(profile.gap_weight_min);
    let tangent_weight = (profile.tangent_weight_base
        + breakup_bias * profile.tangent_weight_breakup_scale
        - continuation_bias * profile.tangent_weight_continuation_scale
        - readable_detail_relax * profile.tangent_weight_readable_relax_scale)
        .max(profile.tangent_weight_min);
    let overflow_penalty = if max_gap > 0.0 && gap > max_gap {
        1000.0 + (gap - max_gap) * tangent_weight
    } else {
        0.0
    };
    let angle_penalty = if angle_degrees > max_angle {
        1000.0 + (angle_degrees - max_angle) * (2.0 + breakup_bias * 3.0)
    } else {
        0.0
    };
    let readability_join_bonus = if fragment.technical_detail {
        (npr_path_screen_region_readability_score(profile, selection, join_midpoint, kind)
            * profile.readability_join_region_scale
            + fragment.candidate_importance.clamp(0.0, 1.0)
                * profile.readability_join_importance_scale)
            * profile.readable_region_join_bonus
            * (profile.readability_join_continuation_base
                + continuation_bias * profile.readability_join_continuation_scale)
    } else {
        0.0
    };
    let human_arc_bonus = npr_human_join_arc_bonus(
        profile,
        kind,
        fragment.technical_detail,
        angle_degrees,
        continuation_bias,
        fragment.candidate_importance,
    );
    let dead_straight_penalty =
        npr_dead_straight_join_penalty(profile, kind, fragment.technical_detail, angle_degrees);
    gap * gap_weight
        + tangent_mismatch * tangent_weight
        + fragment.avg_depth.abs() * 0.025
        + preferred_bias
        + overflow_penalty
        + angle_penalty
        + dead_straight_penalty
        - readability_join_bonus
        - human_arc_bonus
}

pub(crate) fn npr_path_arc_lengths(points: &[Vec2], viewport: &Viewport) -> Vec<f32> {
    let mut result = Vec::with_capacity(points.len());
    let mut total = 0.0;
    result.push(0.0);
    for index in 1..points.len() {
        total += screen_segment_length_px(points[index - 1], points[index], viewport);
        result.push(total);
    }
    result
}

fn npr_path_is_closed(points: &[Vec2], viewport: &Viewport) -> bool {
    if points.len() < 3 {
        return false;
    }
    screen_segment_length_px(points[0], points[points.len() - 1], viewport) <= 3.0
}

fn npr_path_importance(
    settings: Option<&amigo_render_api::NprLineSettings3d>,
    kind: NprLineKind,
    avg_depth: f32,
    candidate_importance: f32,
    technical_detail: bool,
    source_edge_count: usize,
    average_point: Vec2,
) -> f32 {
    let profile = settings
        .map(npr_cpu_path_joining_profile)
        .unwrap_or_default();
    let depth_factor = (profile.path_importance_depth_base
        - avg_depth.abs() * profile.path_importance_depth_weight)
        .clamp(
            profile.path_importance_depth_min,
            profile
                .path_importance_depth_max
                .max(profile.path_importance_depth_min),
        );
    let kind_factor = match kind {
        NprLineKind::Silhouette => profile.path_importance_silhouette_multiplier,
        NprLineKind::Boundary => profile.path_importance_boundary_multiplier,
        NprLineKind::Crease => profile.path_importance_crease_multiplier,
        NprLineKind::Seam => profile.path_importance_seam_multiplier,
        NprLineKind::Feature => profile.path_importance_feature_multiplier,
        NprLineKind::Contact => profile.path_importance_contact_multiplier,
    };
    let candidate_factor = if technical_detail {
        let selection = settings
            .map(npr_cpu_line_selection_profile)
            .unwrap_or_default();
        let chain_bonus = ((source_edge_count.saturating_sub(1) as f32)
            * profile.path_importance_chain_bonus_per_edge)
            .clamp(0.0, profile.path_importance_chain_bonus_max);
        let local_bonus =
            npr_path_screen_region_readability_score(profile, selection, average_point, kind);
        (profile.path_importance_candidate_base
            + candidate_importance.clamp(0.0, 1.0) * profile.path_importance_candidate_scale
            + chain_bonus
            + local_bonus)
            .clamp(profile.path_importance_min, profile.path_importance_max)
    } else {
        1.0
    };
    depth_factor * kind_factor * candidate_factor
}

fn npr_path_average_point(points: &[Vec2]) -> Vec2 {
    if points.is_empty() {
        return Vec2::ZERO;
    }
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for point in points {
        sum_x += point.x;
        sum_y += point.y;
    }
    let count = points.len() as f32;
    Vec2::new(sum_x / count, sum_y / count)
}

fn npr_path_screen_region_readability_score(
    profile: amigo_render_api::NprPathJoiningProfile3d,
    selection: amigo_render_api::NprLineSelectionProfile3d,
    average_point: Vec2,
    kind: NprLineKind,
) -> f32 {
    let face_focus = ((average_point.y - selection.readable_face_start_y)
        / selection.readable_face_height.max(0.001))
    .clamp(0.0, 1.0)
        * (1.0 - average_point.x.abs() / selection.readable_face_half_width.max(0.001))
            .clamp(0.0, 1.0);
    let torso_focus = (1.0
        - average_point.x.abs() / selection.readable_torso_half_width.max(0.001))
    .clamp(0.0, 1.0)
        * (1.0
            - ((average_point.y - selection.readable_torso_center_y).abs()
                / selection.readable_torso_half_height.max(0.001)))
        .clamp(0.0, 1.0);
    let hand_focus = ((average_point.x.abs() - selection.readable_hand_start_x)
        / selection.readable_hand_width.max(0.001))
    .clamp(0.0, 1.0)
        * ((average_point.y - selection.readable_hand_start_y)
            / selection.readable_hand_height.max(0.001))
        .clamp(0.0, 1.0);
    match kind {
        NprLineKind::Feature => {
            face_focus * profile.region_feature_face_bonus
                + torso_focus * profile.region_feature_torso_bonus
                + hand_focus * profile.region_feature_hand_bonus
        }
        NprLineKind::Crease => {
            face_focus * profile.region_crease_face_bonus
                + torso_focus * profile.region_crease_torso_bonus
                + hand_focus * profile.region_crease_hand_bonus
        }
        NprLineKind::Seam => {
            torso_focus * profile.region_seam_torso_bonus
                + hand_focus * profile.region_seam_hand_bonus
        }
        _ => 0.0,
    }
}

fn npr_human_join_arc_bonus(
    profile: amigo_render_api::NprPathJoiningProfile3d,
    kind: NprLineKind,
    technical_detail: bool,
    angle_degrees: f32,
    continuation_bias: f32,
    candidate_importance: f32,
) -> f32 {
    if !technical_detail {
        return 0.0;
    }

    let (target_angle, window, kind_strength) = match kind {
        NprLineKind::Feature => (
            profile.feature_arc_target_degrees,
            profile.feature_arc_window_degrees,
            profile.feature_arc_bonus,
        ),
        NprLineKind::Crease => (
            profile.crease_arc_target_degrees,
            profile.crease_arc_window_degrees,
            profile.crease_arc_bonus,
        ),
        NprLineKind::Seam => (
            profile.seam_arc_target_degrees,
            profile.seam_arc_window_degrees,
            profile.seam_arc_bonus,
        ),
        _ => return 0.0,
    };
    let curve_alignment = (1.0 - ((angle_degrees - target_angle).abs() / window)).clamp(0.0, 1.0);
    curve_alignment
        * kind_strength
        * (0.6 + continuation_bias.clamp(0.0, 1.0) * 0.5)
        * (0.65 + candidate_importance.clamp(0.0, 1.0) * 0.35)
}

fn npr_dead_straight_join_penalty(
    profile: amigo_render_api::NprPathJoiningProfile3d,
    kind: NprLineKind,
    technical_detail: bool,
    angle_degrees: f32,
) -> f32 {
    if !technical_detail || !matches!(kind, NprLineKind::Feature | NprLineKind::Crease) {
        return 0.0;
    }
    if angle_degrees >= 2.5 {
        return 0.0;
    }
    let penalty_strength = match kind {
        NprLineKind::Feature => profile.feature_dead_straight_penalty,
        NprLineKind::Crease => profile.crease_dead_straight_penalty,
        _ => 0.0,
    };
    (1.0 - angle_degrees / 2.5).clamp(0.0, 1.0) * penalty_strength
}

pub(crate) fn sorted_npr_source_edges(source_edges: &[u64]) -> Vec<u64> {
    let mut sorted = source_edges.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}

pub(crate) fn stable_path_id(kind: NprLineKind, source_edges: &[u64]) -> u64 {
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

fn dot_vec2(left: Vec2, right: Vec2) -> f32 {
    left.x * right.x + left.y * right.y
}

fn mul_vec2(value: Vec2, scalar: f32) -> Vec2 {
    Vec2::new(value.x * scalar, value.y * scalar)
}

fn midpoint_vec2(left: Vec2, right: Vec2) -> Vec2 {
    Vec2::new((left.x + right.x) * 0.5, (left.y + right.y) * 0.5)
}

fn normalize_vec2(value: Vec2) -> Vec2 {
    let len = (value.x * value.x + value.y * value.y).sqrt();
    if len <= f32::EPSILON {
        Vec2::ZERO
    } else {
        Vec2::new(value.x / len, value.y / len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_detail_threshold_is_more_lenient_for_readable_feature_paths() {
        let mut profile = amigo_render_api::NprPathJoiningProfile3d::default();
        profile.isolated_detail_short_ratio = 0.12;

        assert!(!npr_path_is_unimportant_isolated_detail(
            profile,
            amigo_render_api::NprLineFamilyRole3d::DetailInk,
            20.0,
            80.0,
            1,
            NprLineCandidateTraits {
                technical_detail: true,
                material_detail: false,
                material_seam: false,
            },
        ));

        assert!(npr_path_is_unimportant_isolated_detail(
            amigo_render_api::NprPathJoiningProfile3d::default(),
            amigo_render_api::NprLineFamilyRole3d::DetailInk,
            20.0,
            120.0,
            1,
            NprLineCandidateTraits {
                technical_detail: true,
                material_detail: false,
                material_seam: false,
            },
        ));
    }

    #[test]
    fn path_importance_rewards_chained_feature_paths() {
        let isolated =
            npr_path_importance(None, NprLineKind::Feature, 0.0, 0.75, true, 1, Vec2::ZERO);
        let chained =
            npr_path_importance(None, NprLineKind::Feature, 0.0, 0.75, true, 4, Vec2::ZERO);

        assert!(chained > isolated);
    }

    #[test]
    fn path_importance_uses_profile_depth_and_kind_multipliers() {
        let mut profile = amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink();
        profile.path_joining.path_importance_feature_multiplier = 1.30;
        profile.path_joining.path_importance_depth_base = 1.25;
        profile.path_joining.path_importance_depth_weight = 0.02;
        profile.path_joining.path_importance_depth_min = 0.95;
        profile.path_joining.path_importance_depth_max = 1.25;
        let profiled_settings = amigo_render_api::NprLineSettings3d {
            cpu_strategy_profile: profile,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let default_importance =
            npr_path_importance(None, NprLineKind::Feature, 0.8, 0.80, true, 2, Vec2::ZERO);
        let profiled_importance = npr_path_importance(
            Some(&profiled_settings),
            NprLineKind::Feature,
            0.8,
            0.80,
            true,
            2,
            Vec2::ZERO,
        );

        assert!(profiled_importance > default_importance * 1.25);
    }

    #[test]
    fn path_importance_prefers_upper_center_feature_region() {
        let upper_center = npr_path_importance(
            None,
            NprLineKind::Feature,
            0.0,
            0.75,
            true,
            2,
            Vec2::new(0.04, 0.56),
        );
        let lower_side = npr_path_importance(
            None,
            NprLineKind::Feature,
            0.0,
            0.75,
            true,
            2,
            Vec2::new(0.58, -0.70),
        );

        assert!(upper_center > lower_side);
    }

    #[test]
    fn kind_min_stroke_length_uses_path_joining_profile_multipliers() {
        let traits = NprLineCandidateTraits {
            technical_detail: true,
            material_detail: false,
            material_seam: false,
        };
        let default_settings = amigo_render_api::NprLineSettings3d {
            min_stroke_length_px: 24.0,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let mut relaxed_settings = default_settings.clone();
        relaxed_settings
            .cpu_strategy_profile
            .path_joining
            .min_length_feature_multiplier = 0.42;

        let default_min =
            npr_kind_min_stroke_length_px(NprLineKind::Feature, traits, &default_settings);
        let relaxed_min =
            npr_kind_min_stroke_length_px(NprLineKind::Feature, traits, &relaxed_settings);

        assert!(default_min > 0.0);
        assert!(relaxed_min < default_min);
    }

    #[test]
    fn join_score_prefers_gentle_feature_arc_over_dead_straight() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            cpu_strategy_profile: amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let straight = NprLineFragment {
            source_edge_id: 1,
            kind: NprLineKind::Feature,
            candidate_importance: 0.92,
            technical_detail: true,
            material_detail: false,
            material_seam: false,
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(0.08, 0.0),
            t0: 0.0,
            t1: 1.0,
            tangent0: Vec2::new(1.0, 0.0),
            tangent1: Vec2::new(1.0, 0.0),
            avg_depth: 0.0,
        };
        let arc = NprLineFragment {
            tangent0: normalize_vec2(Vec2::new(1.0, 0.22)),
            tangent1: normalize_vec2(Vec2::new(1.0, 0.22)),
            ..straight
        };

        let straight_score = npr_path_join_score(
            NprLineKind::Feature,
            straight,
            0,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            18.0,
            &viewport,
            Some(&settings),
        );
        let arc_score = npr_path_join_score(
            NprLineKind::Feature,
            arc,
            0,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            18.0,
            &viewport,
            Some(&settings),
        );

        assert!(arc_score < straight_score);
    }

    #[test]
    fn human_join_arc_bonus_uses_profiled_seam_arc_values() {
        let mut profile = amigo_render_api::NprPathJoiningProfile3d::default();
        profile.seam_arc_target_degrees = 10.0;
        profile.seam_arc_window_degrees = 12.0;
        profile.seam_arc_bonus = 0.0;

        let suppressed = npr_human_join_arc_bonus(profile, NprLineKind::Seam, true, 10.0, 0.6, 0.8);

        profile.seam_arc_bonus = 0.5;
        let emphasized = npr_human_join_arc_bonus(profile, NprLineKind::Seam, true, 10.0, 0.6, 0.8);

        assert_eq!(suppressed, 0.0);
        assert!(emphasized > 0.3);
    }

    #[test]
    fn join_score_prefers_upper_center_feature_detail_region() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let settings = amigo_render_api::NprLineSettings3d {
            cpu_strategy_profile: amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let upper_center = NprLineFragment {
            source_edge_id: 1,
            kind: NprLineKind::Feature,
            candidate_importance: 0.88,
            technical_detail: true,
            material_detail: false,
            material_seam: false,
            p0: Vec2::new(0.02, 0.54),
            p1: Vec2::new(0.08, 0.56),
            t0: 0.0,
            t1: 1.0,
            tangent0: normalize_vec2(Vec2::new(1.0, 0.18)),
            tangent1: normalize_vec2(Vec2::new(1.0, 0.18)),
            avg_depth: 0.0,
        };
        let lower_side = NprLineFragment {
            p0: Vec2::new(0.56, -0.64),
            p1: Vec2::new(0.62, -0.62),
            ..upper_center
        };

        let upper_score = npr_path_join_score(
            NprLineKind::Feature,
            upper_center,
            0,
            upper_center.p0,
            Vec2::new(1.0, 0.0),
            20.0,
            &viewport,
            Some(&settings),
        );
        let lower_score = npr_path_join_score(
            NprLineKind::Feature,
            lower_side,
            0,
            lower_side.p0,
            Vec2::new(1.0, 0.0),
            20.0,
            &viewport,
            Some(&settings),
        );

        assert!(upper_score < lower_score);
    }

    #[test]
    fn path_region_score_uses_line_selection_readable_region_shape() {
        let mut focused_profile = amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink();
        focused_profile.line_selection.readable_face_start_y = 0.48;
        focused_profile.line_selection.readable_face_height = 0.20;
        focused_profile.line_selection.readable_face_half_width = 0.18;
        focused_profile.line_selection.readable_torso_half_width = 0.01;
        focused_profile.line_selection.readable_hand_width = 0.01;
        focused_profile.path_joining.region_feature_face_bonus = 1.0;
        focused_profile.path_joining.region_feature_torso_bonus = 0.0;
        focused_profile.path_joining.region_feature_hand_bonus = 0.0;
        let mut excluded_profile = focused_profile;
        excluded_profile.line_selection.readable_face_start_y = 0.90;
        excluded_profile.line_selection.readable_face_height = 0.10;

        let focused_score = npr_path_screen_region_readability_score(
            focused_profile.path_joining,
            focused_profile.line_selection,
            Vec2::new(0.02, 0.54),
            NprLineKind::Feature,
        );
        let excluded_score = npr_path_screen_region_readability_score(
            excluded_profile.path_joining,
            excluded_profile.line_selection,
            Vec2::new(0.02, 0.54),
            NprLineKind::Feature,
        );

        assert!(focused_score > excluded_score);
    }

    #[test]
    fn join_score_profile_can_relax_readable_feature_detail_join_penalties() {
        let viewport = Viewport::from_dimensions(800.0, 600.0);
        let mut relaxed_profile = amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink();
        relaxed_profile.path_joining.gap_weight_base = 0.58;
        relaxed_profile.path_joining.tangent_weight_base = 6.2;
        relaxed_profile
            .path_joining
            .tangent_weight_readable_relax_scale = 5.0;
        relaxed_profile.path_joining.readability_join_region_scale = 1.90;
        relaxed_profile
            .path_joining
            .readability_join_importance_scale = 0.30;
        let relaxed = amigo_render_api::NprLineSettings3d {
            cpu_strategy_profile: relaxed_profile,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let baseline = amigo_render_api::NprLineSettings3d {
            cpu_strategy_profile: amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let fragment = NprLineFragment {
            source_edge_id: 1,
            kind: NprLineKind::Feature,
            candidate_importance: 0.90,
            technical_detail: true,
            material_detail: false,
            material_seam: false,
            p0: Vec2::new(0.02, 0.54),
            p1: Vec2::new(0.10, 0.58),
            t0: 0.0,
            t1: 1.0,
            tangent0: normalize_vec2(Vec2::new(1.0, 0.26)),
            tangent1: normalize_vec2(Vec2::new(1.0, 0.26)),
            avg_depth: 0.0,
        };

        let baseline_score = npr_path_join_score(
            NprLineKind::Feature,
            fragment,
            0,
            fragment.p0,
            Vec2::new(1.0, 0.0),
            20.0,
            &viewport,
            Some(&baseline),
        );
        let relaxed_score = npr_path_join_score(
            NprLineKind::Feature,
            fragment,
            0,
            fragment.p0,
            Vec2::new(1.0, 0.0),
            20.0,
            &viewport,
            Some(&relaxed),
        );

        assert!(relaxed_score < baseline_score);
    }
}
