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

struct GpuNprAggregatedPath3d {
    start: vec4<f32>,
    end: vec4<f32>,
    control: vec4<f32>,
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
    params21: vec4<f32>,
    params22: vec4<f32>,
    params23: vec4<f32>,
    params24: vec4<f32>,
    params25: vec4<f32>,
    params26: vec4<f32>,
    params27: vec4<f32>,
    params28: vec4<f32>,
    params29: vec4<f32>,
    params30: vec4<f32>,
    params31: vec4<f32>,
    params32: vec4<f32>,
    params33: vec4<f32>,
    params34: vec4<f32>,
    params35: vec4<f32>,
    params36: vec4<f32>,
    params37: vec4<f32>,
    params38: vec4<f32>,
    params39: vec4<f32>,
    params40: vec4<f32>,
    params41: vec4<f32>,
    params42: vec4<f32>,
    params43: vec4<f32>,
    params44: vec4<f32>,
    params45: vec4<f32>,
    params46: vec4<f32>,
    params47: vec4<f32>,
    params48: vec4<f32>,
    params49: vec4<f32>,
    params50: vec4<f32>,
    params51: vec4<f32>,
    params52: vec4<f32>,
    params53: vec4<f32>,
    params54: vec4<f32>,
    ink_color: vec4<f32>,
    seed: vec4<u32>,
    pipeline0: vec4<u32>,
    pipeline1: vec4<u32>,
    material_roles0: vec4<u32>,
}

const KIND_NONE: u32 = 0u;
const KIND_BOUNDARY: u32 = 1u;
const KIND_SILHOUETTE: u32 = 2u;
const PATH_FLAG_EMIT: u32 = 1u;
const PATH_FLAG_CONNECTED_START: u32 = 2u;
const PATH_FLAG_CONNECTED_END: u32 = 4u;
const STROKE_AKIRA_INK: u32 = 1u;
const STROKE_CONFIDENT_MANGA_INK: u32 = 4u;

@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(10) var<storage, read> path_links: array<GpuNprPathLink3d>;
@group(0) @binding(14) var<storage, read> path_states: array<GpuNprPathState3d>;
@group(0) @binding(15) var<storage, read_write> aggregated_paths: array<GpuNprAggregatedPath3d>;

fn active_edge_count() -> u32 {
    return min(uniforms.pipeline1.w, u32(arrayLength(&visible_segments)));
}

fn valid_visible(index: u32, kind: u32) -> bool {
    if (index == 0xffffffffu || index >= active_edge_count()) {
        return false;
    }
    let segment = visible_segments[index];
    return segment.kind_edge.x == kind && segment.start.w > 0.5 && segment.end.w > 0.5;
}

fn visible_length(segment: GpuNprVisibleSegment3d) -> f32 {
    return distance(segment.start.xy, segment.end.xy);
}

fn distance_to_line_segment(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let line = end - start;
    let line_len2 = dot(line, line);
    if (line_len2 <= 0.0001) {
        return distance(point, start);
    }
    let t = clamp(dot(point - start, line) / line_len2, 0.0, 1.0);
    return distance(point, start + line * t);
}

fn uses_manga_stroked_paths() -> bool {
    return uniforms.pipeline0.z == STROKE_AKIRA_INK
        || uniforms.pipeline0.z == STROKE_CONFIDENT_MANGA_INK;
}

fn is_primary_contour(kind: u32) -> bool {
    return kind == KIND_BOUNDARY || kind == KIND_SILHOUETTE;
}

fn contour_join_gap_px(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return max(uniforms.params40.x, 1.0);
    }
    if (kind == KIND_BOUNDARY) {
        return max(uniforms.params40.y, 1.0);
    }
    return 2.0;
}

fn contour_join_min_dot(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return max(uniforms.params41.z, uniforms.params15.x);
    }
    if (kind == KIND_BOUNDARY) {
        return max(uniforms.params41.w, uniforms.params15.x);
    }
    return uniforms.params15.x;
}

fn matched_start_for_anchor(anchor: vec2<f32>, segment: GpuNprVisibleSegment3d) -> bool {
    return distance(anchor, segment.start.xy) <= distance(anchor, segment.end.xy);
}

fn valid_endpoint_vertex(vertex: u32) -> bool {
    return vertex != 0xffffffffu;
}

fn visible_endpoint_vertex(segment: GpuNprVisibleSegment3d, matched_start: bool) -> u32 {
    return select(segment.kind_edge.w, segment.kind_edge.z, matched_start);
}

fn same_source_endpoint(
    current: GpuNprVisibleSegment3d,
    current_start: bool,
    next: GpuNprVisibleSegment3d,
    next_start: bool,
) -> bool {
    let current_vertex = visible_endpoint_vertex(current, current_start);
    let next_vertex = visible_endpoint_vertex(next, next_start);
    if (valid_endpoint_vertex(current_vertex) && valid_endpoint_vertex(next_vertex)) {
        return current_vertex == next_vertex;
    }
    // Some GPU-visible runs do not preserve the exact source vertex after clipping. In that
    // case the screen-space gap and owner checks below still guard the join.
    return true;
}

fn clear_aggregated_path(index: u32) {
    aggregated_paths[index] = GpuNprAggregatedPath3d(
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<u32>(0u),
        vec4<f32>(0.0),
        vec4<f32>(0.0),
    );
}

fn extend_endpoint(edge_index: u32, next_index: u32, from_start: bool, kind: u32) -> vec4<f32> {
    let current = visible_segments[edge_index];
    if (!valid_visible(next_index, kind)) {
        return select(current.end, current.start, from_start);
    }
    let next = visible_segments[next_index];
    let anchor = select(current.end.xy, current.start.xy, from_start);
    let matched_start = matched_start_for_anchor(anchor, next);
    return select(next.start, next.end, matched_start);
}

fn walk_endpoint(edge_index: u32, first_next_index: u32, from_start: bool, kind: u32) -> vec4<f32> {
    let current_segment = visible_segments[edge_index];
    var result = select(current_segment.end, current_segment.start, from_start);
    var anchor = result.xy;
    var anchor_depth = result.z;
    var current_direction = normalize(select(
        current_segment.end.xy - current_segment.start.xy,
        current_segment.start.xy - current_segment.end.xy,
        from_start,
    ));
    var current_length = max(distance(current_segment.start.xy, current_segment.end.xy), 0.0001);
    var next_index = first_next_index;
    let max_gap = clamp(contour_join_gap_px(kind) * 1.05, 6.0, 10.0);
    let min_continue_dot = max(contour_join_min_dot(kind), 0.42);
    let max_hops = min(max(u32(uniforms.params14.w), 3u), 12u);
    let owner_state = path_states[edge_index];
    for (var hop = 0u; hop < 12u; hop = hop + 1u) {
        if (hop >= max_hops) {
            break;
        }
        if (!valid_visible(next_index, kind) || next_index >= u32(arrayLength(&path_links))) {
            break;
        }
        let next = visible_segments[next_index];
        let next_state = path_states[next_index];
        let next_link = path_links[next_index];
        let owned_by_path = next_state.owner_segment == edge_index || next_state.owner_segment == owner_state.owner_segment;
        let locally_owned = next_link.owner_edge == edge_index || next_link.owner_edge == next_index;
        if (!owned_by_path && !locally_owned) {
            break;
        }
        let matched_start = matched_start_for_anchor(anchor, next);
        if (!same_source_endpoint(current_segment, from_start, next, matched_start) && distance(anchor, select(next.start.xy, next.end.xy, matched_start)) > max_gap * 0.42) {
            break;
        }
        let matched = select(next.end, next.start, matched_start);
        let far = select(next.start, next.end, matched_start);
        if (distance(anchor, matched.xy) > max_gap) {
            break;
        }
        let next_direction = normalize(far.xy - matched.xy);
        if (dot(current_direction, next_direction) < min_continue_dot) {
            break;
        }
        let depth_gap = abs(anchor_depth - far.z);
        let max_depth_gap = 0.055 + (1.0 - max(dot(current_direction, next_direction), 0.0)) * 0.16;
        if (depth_gap > max_depth_gap) {
            break;
        }
        let next_length = max(distance(matched.xy, far.xy), 0.0001);
        let length_ratio = abs(next_length - current_length) / max(max(next_length, current_length), 1.0);
        if (hop > 0u && length_ratio > 1.25) {
            break;
        }
        result = far;
        anchor = result.xy;
        anchor_depth = result.z;
        current_direction = next_direction;
        current_length = next_length;
        next_index = select(next_link.start_next, next_link.end_next, matched_start);
    }
    return result;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= active_edge_count() || edge_index >= u32(arrayLength(&aggregated_paths))) {
        return;
    }

    clear_aggregated_path(edge_index);

    let state = path_states[edge_index];
    let segment = visible_segments[edge_index];
    let kind = segment.kind_edge.x;
    if (
        kind == KIND_NONE
        || segment.start.w <= 0.5
        || segment.end.w <= 0.5
        || state.owner_segment != edge_index
        || (state.flags & PATH_FLAG_EMIT) == 0u
    ) {
        return;
    }

    let link = path_links[edge_index];
    let use_aggregation = uses_manga_stroked_paths() && is_primary_contour(kind);
    let start = select(segment.start, walk_endpoint(edge_index, link.start_next, true, kind), use_aggregation);
    let end = select(segment.end, walk_endpoint(edge_index, link.end_next, false, kind), use_aggregation);
    let length_px = distance(start.xy, end.xy);
    if (length_px <= 0.0001) {
        return;
    }

    let seed_mid = (segment.start.xy + segment.end.xy) * 0.5;
    let chord_deviation = distance_to_line_segment(seed_mid, start.xy, end.xy);
    let extension_ratio = (length_px - distance(segment.start.xy, segment.end.xy)) / max(length_px, 1.0);
    let safe_aggregation =
        use_aggregation
        && extension_ratio >= 0.08
        && extension_ratio <= 0.72
        && chord_deviation <= max(contour_join_gap_px(kind) * 0.72, 4.0);
    let control = vec4<f32>(segment.start.xy, (segment.start.z + segment.end.z) * 0.5, 1.0);
    let connected_flags =
        select(0u, PATH_FLAG_CONNECTED_START, link.start_next != 0xffffffffu)
        | select(0u, PATH_FLAG_CONNECTED_END, link.end_next != 0xffffffffu);
    aggregated_paths[edge_index] = GpuNprAggregatedPath3d(
        start,
        end,
        control,
        vec4<u32>(state.path_id, edge_index, kind, state.flags | connected_flags),
        vec4<f32>(length_px, segment.metrics.z, (start.z + end.z) * 0.5, f32(state.segment_count)),
        vec4<f32>(segment.end.xy, segment.metrics.w, select(0.0, 1.35, safe_aggregation)),
    );
}
