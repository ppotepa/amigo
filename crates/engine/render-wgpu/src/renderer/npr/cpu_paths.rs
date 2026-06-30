use std::collections::BTreeMap;

use amigo_math::Vec2;

use crate::renderer::{
    NprLineFragment, NprLineKind, NprStrokePath, Viewport, npr_breakup_bias_with_traits,
    npr_continuation_bias_with_traits, npr_line_family_role_for_kind,
    npr_min_stroke_length_px_with_traits, npr_preferred_stroke_length_px_with_traits,
    npr_stroke_join_gap_px_with_traits, npr_stroke_join_max_angle_degrees_with_traits,
    npr_technical_detail_keep_with_traits, screen_segment_length_px,
};

use super::{NprLineCandidateTraits, deterministic_noise};

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
            (node.fragment, node.k1, node.fragment.p1, node.fragment.tangent1)
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
        )
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
        paths.push(NprStrokePath {
            path_id,
            kind,
            arc_lengths_px,
            importance: npr_path_importance(
                kind,
                avg_depth,
                candidate_importance,
                technical_detail,
                source_edges.len(),
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
    if npr_path_is_unimportant_isolated_detail(
        role,
        length_px,
        length_span,
        source_edge_count,
        traits,
    ) {
        return false;
    }
    let chain_bonus = ((source_edge_count.saturating_sub(1) as f32) * 0.04).clamp(0.0, 0.18);
    let continuation_bias = npr_continuation_bias_with_traits(kind, traits, settings);
    let breakup_bias = npr_breakup_bias_with_traits(kind, traits, settings);
    let role_bonus = match role {
        amigo_render_api::NprLineFamilyRole3d::ClothFold => {
            0.05 + (source_edge_count.saturating_sub(1) as f32 * 0.025).clamp(0.0, 0.10)
        }
        amigo_render_api::NprLineFamilyRole3d::DetailInk => {
            if traits.material_detail { 0.10 } else { 0.02 }
        }
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => {
            if traits.material_seam { 0.08 } else { -0.06 }
        }
        _ => 0.0,
    };
    let keep_probability = (npr_technical_detail_keep_with_traits(kind, traits, settings).clamp(0.0, 1.0) * 0.14
        + 0.08
        + length_score * 0.56
        + settings.line_confidence.clamp(0.0, 1.0) * 0.10
        + chain_bonus
        + role_bonus
        + continuation_bias * 0.10
        - breakup_bias * 0.10)
        .clamp(0.0, 1.0);
    deterministic_noise(settings.seed, path_id, npr_line_kind_seed(kind), 911) <= keep_probability
}

fn npr_path_is_unimportant_isolated_detail(
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
        amigo_render_api::NprLineFamilyRole3d::DetailInk => 0.30,
        amigo_render_api::NprLineFamilyRole3d::ClothFold => 0.32,
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => 0.36,
        _ => return false,
    };
    length_px < preferred_length_px * short_ratio
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

    let readability_multiplier =
        if settings.pipeline.budget_strategy == amigo_render_api::NprBudgetStrategy3d::CharacterReadability
        {
            1.18
        } else {
            1.0
        };

    let kind_multiplier = match kind {
        NprLineKind::Silhouette => 0.38,
        NprLineKind::Boundary => 0.55,
        NprLineKind::Contact => 0.75,
        NprLineKind::Crease | NprLineKind::Seam | NprLineKind::Feature => 1.0,
    };
    base * kind_multiplier * readability_multiplier
}

fn npr_path_uses_character_readability(settings: &amigo_render_api::NprLineSettings3d) -> bool {
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
                candidates.extend(entries.iter().copied().filter(|entry| !visited[entry.fragment_index]));
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
    let tangent_dot = dot_vec2(normalize_vec2(entry_tangent), normalize_vec2(tangent)).clamp(-1.0, 1.0);
    let tangent_mismatch = 1.0 - tangent_dot;
    let angle_degrees = tangent_dot.acos().to_degrees();
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
    let preferred_bias = if preferred_length > 0.0 && current_length_px < preferred_length {
        -((preferred_length - current_length_px) / preferred_length).clamp(0.0, 1.0)
            * (1.6 + continuation_bias * 2.2)
    } else {
        0.0
    };
    let gap_weight = 0.8 + breakup_bias * 0.35 - continuation_bias * 0.15;
    let tangent_weight = 8.0 + breakup_bias * 8.0 - continuation_bias * 2.0;
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
    gap * gap_weight + tangent_mismatch * tangent_weight + fragment.avg_depth.abs() * 0.025 + preferred_bias
        + overflow_penalty
        + angle_penalty
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
    kind: NprLineKind,
    avg_depth: f32,
    candidate_importance: f32,
    technical_detail: bool,
    source_edge_count: usize,
) -> f32 {
    let depth_factor = (1.18 - avg_depth.abs() * 0.08).clamp(0.72, 1.18);
    let kind_factor = match kind {
        NprLineKind::Silhouette => 1.08,
        NprLineKind::Boundary => 0.96,
        NprLineKind::Crease => 0.88,
        NprLineKind::Seam => 0.82,
        NprLineKind::Feature => 0.88,
        NprLineKind::Contact => 0.92,
    };
    let candidate_factor = if technical_detail {
        let chain_bonus = ((source_edge_count.saturating_sub(1) as f32) * 0.03).clamp(0.0, 0.18);
        (0.58 + candidate_importance.clamp(0.0, 1.0) * 0.42 + chain_bonus).clamp(0.52, 1.08)
    } else {
        1.0
    };
    depth_factor * kind_factor * candidate_factor
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

fn normalize_vec2(value: Vec2) -> Vec2 {
    let len = (value.x * value.x + value.y * value.y).sqrt();
    if len <= f32::EPSILON {
        Vec2::ZERO
    } else {
        Vec2::new(value.x / len, value.y / len)
    }
}
