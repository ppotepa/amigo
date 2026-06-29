struct GpuNprVisibleSegment3d {
    start: vec4<f32>,
    end: vec4<f32>,
    kind_edge: vec4<u32>,
    metrics: vec4<f32>,
}

struct GpuNprPathLink3d {
    owner_edge: u32,
    start_next: u32,
    end_next: u32,
    flags: u32,
}

struct GpuNprPathState3d {
    owner_segment: u32,
    path_id: u32,
    kind: u32,
    flags: u32,
    segment_count: u32,
    _pad0: vec3<u32>,
}

struct GpuNprPathSegment3d {
    start: vec4<f32>,
    end: vec4<f32>,
    path: vec4<u32>,
    metrics: vec4<f32>,
    style_metrics: vec4<f32>,
}

struct GpuNprFrameUniforms3d {
    model_translation: vec4<f32>,
    model_rotation: vec4<f32>,
    model_scale: vec4<f32>,
    camera_translation: vec4<f32>,
    camera_rotation: vec4<f32>,
    viewport_half: vec4<f32>,
    params0: vec4<f32>,
    params1: vec4<f32>,
    params2: vec4<f32>,
    params3: vec4<f32>,
    params4: vec4<f32>,
    params5: vec4<f32>,
    params6: vec4<f32>,
    params7: vec4<f32>,
    params8: vec4<f32>,
    params9: vec4<f32>,
    params10: vec4<f32>,
    params11: vec4<f32>,
    params12: vec4<f32>,
    params13: vec4<f32>,
    params14: vec4<f32>,
    params15: vec4<f32>,
    params16: vec4<f32>,
    params17: vec4<f32>,
    params18: vec4<f32>,
    params19: vec4<f32>,
    params20: vec4<f32>,
    ink_color: vec4<f32>,
    seed: vec4<u32>,
    pipeline0: vec4<u32>,
    pipeline1: vec4<u32>,
    material_roles0: vec4<u32>,
}

struct PathWalkResult {
    point: vec2<f32>,
    depth: f32,
    extra_length: f32,
    near_point: vec2<f32>,
    near_depth: f32,
    near_length: f32,
    mid_point: vec2<f32>,
    mid_depth: f32,
    mid_length: f32,
    penultimate_point: vec2<f32>,
    penultimate_depth: f32,
    penultimate_length: f32,
    hops: u32,
}

const KIND_NONE: u32 = 0u;
const PATH_FLAG_EMIT: u32 = 1u;
const PATH_FLAG_CONNECTED_START: u32 = 2u;
const PATH_FLAG_CONNECTED_END: u32 = 4u;
const PATH_STRATEGY_DIRECT_VISIBLE_SEGMENTS: u32 = 1u;
const CANDIDATE_CHARACTER_SEMANTIC: u32 = 1u;
const BUDGET_FACE_SILHOUETTE_PRIORITY: u32 = 1u;
const BUDGET_CHARACTER_READABILITY: u32 = 2u;
const PATH_SEGMENTS_PER_VISIBLE_EDGE: u32 = 3u;

@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(9) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(10) var<storage, read> path_links: array<GpuNprPathLink3d>;
@group(0) @binding(13) var<storage, read_write> path_segments: array<GpuNprPathSegment3d>;
@group(0) @binding(14) var<storage, read> path_states: array<GpuNprPathState3d>;

fn uses_direct_visible_segments() -> bool {
    return uniforms.pipeline0.y == PATH_STRATEGY_DIRECT_VISIBLE_SEGMENTS;
}

fn uses_character_semantic_candidates() -> bool {
    return uniforms.pipeline0.x == CANDIDATE_CHARACTER_SEMANTIC;
}

fn uses_character_budget() -> bool {
    return uniforms.pipeline1.y == BUDGET_FACE_SILHOUETTE_PRIORITY
        || uniforms.pipeline1.y == BUDGET_CHARACTER_READABILITY;
}

fn active_edge_count() -> u32 {
    return min(uniforms.pipeline1.w, u32(arrayLength(&visible_segments)));
}

fn path_segment_base() -> u32 {
    return uniforms.material_roles0.z;
}

fn path_segment_slot_count() -> u32 {
    return uniforms.material_roles0.w;
}

fn clear_path_segment(path_segment_index: u32) {
    path_segments[path_segment_index] = GpuNprPathSegment3d(
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<u32>(0u),
        vec4<f32>(0.0),
        vec4<f32>(0.0),
    );
}

fn path_style_metrics(visible: GpuNprVisibleSegment3d, t0: f32, t1: f32) -> vec4<f32> {
    let start_depth = mix(visible.metrics.x, visible.metrics.y, clamp(t0, 0.0, 1.0));
    let end_depth = mix(visible.metrics.x, visible.metrics.y, clamp(t1, 0.0, 1.0));
    return vec4<f32>(start_depth, end_depth, (start_depth + end_depth) * 0.5, 0.0);
}

fn make_path_segment(
    visible: GpuNprVisibleSegment3d,
    start: vec4<f32>,
    end: vec4<f32>,
    path: vec4<u32>,
    metrics: vec4<f32>,
) -> GpuNprPathSegment3d {
    return GpuNprPathSegment3d(
        start,
        end,
        path,
        metrics,
        path_style_metrics(visible, metrics.x, metrics.y),
    );
}

fn path_importance(kind: u32, depth01: f32) -> f32 {
    let depth_factor = clamp(1.18 - depth01 * 0.38, 0.72, 1.18);
    if (kind == 2u) {
        return depth_factor * select(1.08, 1.18, uses_character_budget());
    }
    if (kind == 1u) {
        return depth_factor * 0.96;
    }
    if (kind == 6u) {
        return depth_factor * select(0.92, 0.72, uses_character_budget());
    }
    return depth_factor * select(0.88, 0.70, uses_character_semantic_candidates() || uses_character_budget());
}

fn path_importance_with_chain(
    kind: u32,
    depth01: f32,
    hop_count: u32,
    segment_count: u32,
    candidate_importance: f32,
) -> f32 {
    let base = path_importance(kind, depth01);
    let hop_boost = clamp(1.0 + f32(min(hop_count, 4u)) * 0.035, 1.0, 1.14);
    let segment_boost = clamp(0.88 + f32(min(segment_count, 6u)) * 0.055, 0.88, 1.16);
    let candidate_scale = mix(0.68, 1.18, clamp(candidate_importance, 0.0, 1.0));
    return clamp(base * hop_boost * segment_boost * candidate_scale, 0.42, 1.32);
}

fn valid_visible_segment(edge_index: u32, kind: u32) -> bool {
    if (edge_index == 0xffffffffu || edge_index >= active_edge_count()) {
        return false;
    }
    let seg = visible_segments[edge_index];
    return seg.kind_edge.x == kind && seg.start.w > 0.5 && seg.end.w > 0.5;
}

fn endpoint_match_is_start(anchor: vec2<f32>, seg: GpuNprVisibleSegment3d) -> bool {
    return distance(anchor, seg.start.xy) <= distance(anchor, seg.end.xy);
}

fn endpoint_match_gap(anchor: vec2<f32>, seg: GpuNprVisibleSegment3d) -> f32 {
    return min(distance(anchor, seg.start.xy), distance(anchor, seg.end.xy));
}

fn continue_from_matched_side(link: GpuNprPathLink3d, matched_start: bool) -> u32 {
    return select(link.start_next, link.end_next, matched_start);
}

fn valid_endpoint_vertex(vertex: u32) -> bool {
    return vertex != 0xffffffffu;
}

fn visible_endpoint_vertex(seg: GpuNprVisibleSegment3d, matched_start: bool) -> u32 {
    return select(seg.kind_edge.w, seg.kind_edge.z, matched_start);
}

fn matched_endpoint_is_valid(seg: GpuNprVisibleSegment3d, matched_start: bool) -> bool {
    return valid_endpoint_vertex(visible_endpoint_vertex(seg, matched_start));
}

fn hash_u32(value: u32) -> u32 {
    var x = value;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    return (x >> 16u) ^ x;
}

fn quantize_path_coord(value: f32) -> u32 {
    let snap = max(uniforms.params12.w, 0.5);
    return bitcast<u32>(i32(round(value / snap)));
}

fn stable_path_id(
    kind: u32,
    start_point: vec2<f32>,
    end_point: vec2<f32>,
    hop_count: u32,
    total_length: f32,
) -> u32 {
    let sx0 = quantize_path_coord(start_point.x);
    let sy0 = quantize_path_coord(start_point.y);
    let ex0 = quantize_path_coord(end_point.x);
    let ey0 = quantize_path_coord(end_point.y);
    let start_key = vec2<u32>(sx0, sy0);
    let end_key = vec2<u32>(ex0, ey0);
    let canonical_start = select(start_key, end_key, start_key.x > end_key.x || (start_key.x == end_key.x && start_key.y > end_key.y));
    let canonical_end = select(end_key, start_key, start_key.x > end_key.x || (start_key.x == end_key.x && start_key.y > end_key.y));
    let length_bucket = quantize_path_coord(total_length);
    return hash_u32(
        (kind * 0x9E3779B1u)
        ^ canonical_start.x
        ^ (canonical_start.y * 3u)
        ^ (canonical_end.x * 5u)
        ^ (canonical_end.y * 7u)
        ^ (hop_count * 11u)
        ^ (length_bucket * 13u)
    );
}

fn edge_local_path_id(kind: u32, edge_id: u32) -> u32 {
    return hash_u32(edge_id ^ (kind * 0x9E3779B1u));
}

fn emit_direct_visible_edge_path_segment(
    visible: GpuNprVisibleSegment3d,
    length_px: f32,
    segment_slot: u32,
    path_segment_index: u32,
) {
    if (segment_slot != 0u) {
        return;
    }

    let kind = visible.kind_edge.x;
    let edge_id = visible.kind_edge.y;
    let avg_depth = (visible.start.z + visible.end.z) * 0.5;
    let importance = path_importance_with_chain(kind, avg_depth, 0u, 1u, visible.metrics.z);
    path_segments[path_segment_index] = make_path_segment(
        visible,
        visible.start,
        visible.end,
        vec4<u32>(
            edge_local_path_id(kind, edge_id),
            kind,
            0u,
            PATH_FLAG_EMIT,
        ),
        vec4<f32>(0.0, 1.0, length_px, importance),
    );
}

fn max_chain_cosine() -> f32 {
    return clamp(uniforms.params15.x, -0.999, 0.999);
}

fn walk_path_endpoint(
    owner_edge_index: u32,
    kind: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    initial_direction: vec2<f32>,
    initial_length: f32,
    first_next: u32,
) -> PathWalkResult {
    var result = PathWalkResult(
        anchor_point,
        anchor_depth,
        0.0,
        anchor_point,
        anchor_depth,
        0.0,
        anchor_point,
        anchor_depth,
        0.0,
        anchor_point,
        anchor_depth,
        0.0,
        0u,
    );
    var current_anchor = anchor_point;
    var current_depth = anchor_depth;
    var current_direction = initial_direction;
    var current_length = max(initial_length, 0.0001);
    var next_edge = first_next;
    var previous_edge = owner_edge_index;
    let max_hops = u32(uniforms.params14.z);
    let endpoint_snap = max(uniforms.params12.w, 0.5) * 5.5;
    let chain_cos = max_chain_cosine();

    loop {
        if (next_edge == 0xffffffffu || next_edge == previous_edge || result.hops >= max_hops) {
            break;
        }
        if (!valid_visible_segment(next_edge, kind)) {
            break;
        }

        let seg = visible_segments[next_edge];
        let link = path_links[next_edge];
        if ((link.flags & PATH_FLAG_EMIT) == 0u && link.owner_edge != owner_edge_index) {
            break;
        }
        if (link.owner_edge != owner_edge_index && link.owner_edge != next_edge) {
            break;
        }

        let gap = endpoint_match_gap(current_anchor, seg);
        if (gap > endpoint_snap) {
            break;
        }

        let matched_start = endpoint_match_is_start(current_anchor, seg);
        if (!matched_endpoint_is_valid(seg, matched_start)) {
            break;
        }
        let far_point = select(seg.start.xy, seg.end.xy, matched_start);
        let far_depth = select(seg.start.z, seg.end.z, matched_start);
        let delta = far_point - current_anchor;
        let segment_len = length(delta);
        if (segment_len <= 0.0001) {
            break;
        }
        let segment_direction = delta / segment_len;
        let alignment = dot(current_direction, segment_direction);
        if (alignment < chain_cos) {
            break;
        }
        let depth_gap = abs(current_depth - far_depth);
        let max_depth_gap = 0.04 + (1.0 - max(alignment, 0.0)) * 0.14;
        if (depth_gap > max_depth_gap) {
            break;
        }
        let length_ratio =
            abs(segment_len - current_length) / max(max(segment_len, current_length), 1.0);
        if (result.hops > 0u && length_ratio > 0.8) {
            break;
        }

        result.point = far_point;
        result.depth = far_depth;
        if (result.hops >= 1u) {
            result.penultimate_point = current_anchor;
            result.penultimate_depth = current_depth;
            result.penultimate_length = result.extra_length;
        }
        result.extra_length = result.extra_length + segment_len;
        if (result.hops == 0u) {
            result.near_point = far_point;
            result.near_depth = far_depth;
            result.near_length = segment_len;
        } else if (result.hops == 1u) {
            result.mid_point = far_point;
            result.mid_depth = far_depth;
            result.mid_length = result.extra_length;
        }
        result.hops = result.hops + 1u;

        previous_edge = next_edge;
        current_anchor = far_point;
        current_depth = far_depth;
        current_direction = segment_direction;
        current_length = segment_len;
        next_edge = continue_from_matched_side(link, matched_start);
    }

    return result;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let source_index = id.x;
    let local_path_segment_count = path_segment_slot_count();
    if (source_index >= local_path_segment_count) {
        return;
    }
    let path_segment_index = path_segment_base() + source_index;
    if (path_segment_index >= u32(arrayLength(&path_segments))) {
        return;
    }
    clear_path_segment(path_segment_index);

    let edge_index = source_index / PATH_SEGMENTS_PER_VISIBLE_EDGE;
    let segment_slot = source_index % PATH_SEGMENTS_PER_VISIBLE_EDGE;

    if (edge_index >= active_edge_count() || edge_index >= u32(arrayLength(&path_links))) {
        return;
    }

    let visible = visible_segments[edge_index];
    if (visible.kind_edge.x == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        return;
    }

    let length_px = distance(visible.start.xy, visible.end.xy);
    if (length_px <= 0.0001) {
        return;
    }

    if (uses_direct_visible_segments()) {
        emit_direct_visible_edge_path_segment(visible, length_px, segment_slot, path_segment_index);
        return;
    }

    let link = path_links[edge_index];
    let state = path_states[edge_index];
    if ((state.flags & PATH_FLAG_EMIT) == 0u || state.owner_segment != edge_index) {
        return;
    }

    let connected_start = (link.flags & PATH_FLAG_CONNECTED_START) != 0u;
    let connected_end = (link.flags & PATH_FLAG_CONNECTED_END) != 0u;
    let owner_start_direction = normalize(visible.start.xy - visible.end.xy);
    let owner_end_direction = normalize(visible.end.xy - visible.start.xy);
    let start_walk = walk_path_endpoint(
        edge_index,
        visible.kind_edge.x,
        visible.start.xy,
        visible.start.z,
        owner_start_direction,
        length_px,
        select(0xffffffffu, link.start_next, connected_start),
    );
    let end_walk = walk_path_endpoint(
        edge_index,
        visible.kind_edge.x,
        visible.end.xy,
        visible.end.z,
        owner_end_direction,
        length_px,
        select(0xffffffffu, link.end_next, connected_end),
    );

    let final_start = vec4<f32>(start_walk.point, start_walk.depth, visible.start.w);
    let final_end = vec4<f32>(end_walk.point, end_walk.depth, visible.end.w);
    let total_length = length_px + start_walk.extra_length + end_walk.extra_length;
    if (total_length <= 0.0001) {
        return;
    }
    let hop_count = start_walk.hops + end_walk.hops;
    let avg_depth =
        (visible.start.z + visible.end.z + start_walk.depth + end_walk.depth) * 0.25;
    let importance = path_importance_with_chain(
        visible.kind_edge.x,
        avg_depth,
        hop_count,
        state.segment_count,
        visible.metrics.z,
    );
    let has_start_extension = start_walk.extra_length > 0.0001;
    let has_end_extension = end_walk.extra_length > 0.0001;
    let computed_path_id = stable_path_id(
        visible.kind_edge.x,
        final_start.xy,
        final_end.xy,
        hop_count,
        total_length,
    );
    let stable_path_id = computed_path_id;
    let owner_t0 = clamp(start_walk.extra_length / total_length, 0.0, 1.0);
    let owner_t1 = clamp((start_walk.extra_length + length_px) / total_length, owner_t0, 1.0);

    // GPU realtime should read as drawn strokes, not as every topology micro-segment.
    // Emit at most three gesture spans per path: start extension, owner stroke, end extension.
    if (segment_slot == 0u) {
        if (has_start_extension && distance(final_start.xy, visible.start.xy) > 0.0001) {
            path_segments[path_segment_index] = make_path_segment(
                visible,
                final_start,
                visible.start,
                vec4<u32>(
                    stable_path_id,
                    visible.kind_edge.x,
                    hop_count,
                    PATH_FLAG_EMIT | PATH_FLAG_CONNECTED_END,
                ),
                vec4<f32>(0.0, owner_t0, total_length, importance),
            );
        }
        return;
    }

    if (segment_slot == 1u) {
        path_segments[path_segment_index] = make_path_segment(
            visible,
            visible.start,
            visible.end,
            vec4<u32>(
                stable_path_id,
                visible.kind_edge.x,
                hop_count,
                PATH_FLAG_EMIT
                    | select(0u, PATH_FLAG_CONNECTED_START, has_start_extension)
                    | select(0u, PATH_FLAG_CONNECTED_END, has_end_extension),
            ),
            vec4<f32>(owner_t0, owner_t1, total_length, importance),
        );
        return;
    }

    if (segment_slot == 2u) {
        if (has_end_extension && distance(visible.end.xy, final_end.xy) > 0.0001) {
            path_segments[path_segment_index] = make_path_segment(
                visible,
                visible.end,
                final_end,
                vec4<u32>(
                    stable_path_id,
                    visible.kind_edge.x,
                    hop_count,
                    PATH_FLAG_EMIT | PATH_FLAG_CONNECTED_START,
                ),
                vec4<f32>(owner_t1, 1.0, total_length, importance),
            );
        }
        return;
    }

    return;
}
