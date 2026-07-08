use std::collections::{BTreeMap, BTreeSet};

use amigo_math::Vec2;

use crate::renderer::{
    NprEntityPathHistory3d, NprStrokePath, NprTemporalPathState3d, npr_path_average_y,
};

pub(crate) fn stabilize_npr_paths_for_entity(
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
        let blended =
            if let (true, Some(previous_id)) = (temporal_path_smoothing, matched_previous_id) {
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
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(path_id, _)| path_id)
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
    let forward =
        distance_vec2(previous_start, current_start) + distance_vec2(previous_end, current_end);
    let reversed =
        distance_vec2(previous_start, current_end) + distance_vec2(previous_end, current_start);
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
        .map(|(prev, curr)| {
            Vec2::new(
                curr.x * (1.0 - hold) + prev.x * hold,
                curr.y * (1.0 - hold) + prev.y * hold,
            )
        })
        .collect::<Vec<_>>();
    let arc_lengths_px = current.arc_lengths_px.clone();
    NprStrokePath {
        points,
        arc_lengths_px,
        importance: current.importance * (1.0 - hold) + previous.importance * hold,
        ..current
    }
}

fn prune_stale_npr_history(history: &mut BTreeMap<u64, NprTemporalPathState3d>, frame_index: u64) {
    let stale = history
        .iter()
        .filter_map(|(key, state)| {
            (frame_index.saturating_sub(state.last_seen_frame) > 24).then_some(*key)
        })
        .collect::<Vec<_>>();
    for key in stale {
        history.remove(&key);
    }
}

fn distance_vec2(left: Vec2, right: Vec2) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    (dx * dx + dy * dy).sqrt()
}
