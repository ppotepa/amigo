use std::collections::BTreeMap;

use amigo_math::Vec2;

use crate::renderer::{
    NprLineFragment, NprLineKind, NprStrokePath, Viewport, screen_segment_length_px,
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

pub(crate) fn build_npr_stroke_paths(
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
        let avg_depth = fragments
            .iter()
            .map(|fragment| fragment.avg_depth)
            .sum::<f32>()
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
            let left_score = npr_path_join_score(
                nodes[left.fragment_index].fragment,
                left.endpoint,
                join_key,
                entry_tangent,
                viewport,
            );
            let right_score = npr_path_join_score(
                nodes[right.fragment_index].fragment,
                right.endpoint,
                join_key,
                entry_tangent,
                viewport,
            );
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
