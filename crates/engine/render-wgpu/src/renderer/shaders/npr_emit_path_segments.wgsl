struct GpuNprVisibleSegment3d {
    start: vec4<f32>,
    end: vec4<f32>,
    kind_edge: vec4<u32>,
}

struct GpuNprPathLink3d {
    owner_edge: u32,
    start_next: u32,
    end_next: u32,
    flags: u32,
}

struct GpuNprPathSegment3d {
    start: vec4<f32>,
    end: vec4<f32>,
    path: vec4<u32>,
    metrics: vec4<f32>,
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
    ink_color: vec4<f32>,
    seed: vec4<u32>,
}

struct PathWalkResult {
    point: vec2<f32>,
    depth: f32,
    extra_length: f32,
    hops: u32,
}

const KIND_NONE: u32 = 0u;
const PATH_FLAG_EMIT: u32 = 1u;
const PATH_FLAG_CONNECTED_START: u32 = 2u;
const PATH_FLAG_CONNECTED_END: u32 = 4u;

@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(9) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(10) var<storage, read> path_links: array<GpuNprPathLink3d>;
@group(0) @binding(13) var<storage, read_write> path_segments: array<GpuNprPathSegment3d>;

fn path_importance(kind: u32, depth01: f32) -> f32 {
    let depth_factor = clamp(1.18 - depth01 * 0.38, 0.72, 1.18);
    if (kind == 2u) {
        return depth_factor * 1.08;
    }
    if (kind == 1u) {
        return depth_factor * 0.96;
    }
    if (kind == 6u) {
        return depth_factor * 0.92;
    }
    return depth_factor * 0.88;
}

fn valid_visible_segment(edge_index: u32, kind: u32) -> bool {
    if (edge_index == 0xffffffffu || edge_index >= u32(arrayLength(&visible_segments))) {
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
    return select(link.start_next, link.end_next, !matched_start);
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

fn path_segment_flags(
    segment_slot: u32,
    has_start_extension: bool,
    has_end_extension: bool,
) -> u32 {
    var flags = PATH_FLAG_EMIT;
    if (segment_slot == 0u) {
        flags = flags | PATH_FLAG_CONNECTED_END;
        return flags;
    }
    if (segment_slot == 1u) {
        if (has_start_extension) {
            flags = flags | PATH_FLAG_CONNECTED_START;
        }
        flags = flags | PATH_FLAG_CONNECTED_END;
        return flags;
    }
    if (segment_slot == 2u) {
        flags = flags | PATH_FLAG_CONNECTED_START;
        if (has_end_extension) {
            flags = flags | PATH_FLAG_CONNECTED_END;
        }
        return flags;
    }
    flags = flags | PATH_FLAG_CONNECTED_START;
    return flags;
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
    var result = PathWalkResult(anchor_point, anchor_depth, 0.0, 0u);
    var current_anchor = anchor_point;
    var current_depth = anchor_depth;
    var current_direction = initial_direction;
    var current_length = max(initial_length, 0.0001);
    var next_edge = first_next;
    var previous_edge = owner_edge_index;
    let max_hops = max(u32(uniforms.params14.z), 1u);
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
        result.extra_length = result.extra_length + segment_len;
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
    if (source_index >= u32(arrayLength(&path_segments))) {
        return;
    }
    let edge_index = source_index / 4u;
    let segment_slot = source_index % 4u;

    if (edge_index >= u32(arrayLength(&visible_segments)) || edge_index >= u32(arrayLength(&path_links))) {
        return;
    }

    let visible = visible_segments[edge_index];
    let link = path_links[edge_index];
    if (visible.kind_edge.x == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        return;
    }
    if ((link.flags & PATH_FLAG_EMIT) == 0u || link.owner_edge != edge_index) {
        return;
    }

    let length_px = distance(visible.start.xy, visible.end.xy);
    if (length_px <= 0.0001) {
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
    let avg_depth =
        (visible.start.z + visible.end.z + start_walk.depth + end_walk.depth) * 0.25;
    let importance = path_importance(visible.kind_edge.x, avg_depth);
    let has_start_extension = start_walk.extra_length > 0.0001;
    let has_end_extension = end_walk.extra_length > 0.0001;
    let hop_count = start_walk.hops + end_walk.hops;
    let stable_path_id = stable_path_id(
        visible.kind_edge.x,
        final_start.xy,
        final_end.xy,
        hop_count,
        total_length,
    );
    let owner_t0 = clamp(start_walk.extra_length / total_length, 0.0, 1.0);
    let owner_t1 = clamp((start_walk.extra_length + length_px) / total_length, owner_t0, 1.0);
    let owner_mid = vec4<f32>(
        (visible.start.xy + visible.end.xy) * 0.5,
        (visible.start.z + visible.end.z) * 0.5,
        1.0,
    );
    let owner_mid_t = clamp((owner_t0 + owner_t1) * 0.5, owner_t0, owner_t1);

    if (segment_slot == 0u) {
        if (!has_start_extension) {
            return;
        }
        let emit_index = atomicAdd(&indirect_args[2], 1u);
        if (emit_index >= u32(arrayLength(&path_segments))) {
            _ = atomicSub(&indirect_args[2], 1u);
            return;
        }
        path_segments[emit_index] = GpuNprPathSegment3d(
            final_start,
            visible.start,
            vec4<u32>(
                stable_path_id,
                visible.kind_edge.x,
                hop_count,
                path_segment_flags(segment_slot, has_start_extension, has_end_extension),
            ),
            vec4<f32>(0.0, owner_t0, total_length, importance),
        );
    } else if (segment_slot == 1u) {
        let emit_index = atomicAdd(&indirect_args[2], 1u);
        if (emit_index >= u32(arrayLength(&path_segments))) {
            _ = atomicSub(&indirect_args[2], 1u);
            return;
        }
        path_segments[emit_index] = GpuNprPathSegment3d(
            visible.start,
            owner_mid,
            vec4<u32>(
                stable_path_id,
                visible.kind_edge.x,
                hop_count,
                path_segment_flags(segment_slot, has_start_extension, has_end_extension),
            ),
            vec4<f32>(owner_t0, owner_mid_t, total_length, importance),
        );
    } else if (segment_slot == 2u) {
        let emit_index = atomicAdd(&indirect_args[2], 1u);
        if (emit_index >= u32(arrayLength(&path_segments))) {
            _ = atomicSub(&indirect_args[2], 1u);
            return;
        }
        path_segments[emit_index] = GpuNprPathSegment3d(
            owner_mid,
            visible.end,
            vec4<u32>(
                stable_path_id,
                visible.kind_edge.x,
                hop_count,
                path_segment_flags(segment_slot, has_start_extension, has_end_extension),
            ),
            vec4<f32>(owner_mid_t, owner_t1, total_length, importance),
        );
    } else {
        if (!has_end_extension) {
            return;
        }
        let emit_index = atomicAdd(&indirect_args[2], 1u);
        if (emit_index >= u32(arrayLength(&path_segments))) {
            _ = atomicSub(&indirect_args[2], 1u);
            return;
        }
        path_segments[emit_index] = GpuNprPathSegment3d(
            visible.end,
            final_end,
            vec4<u32>(
                stable_path_id,
                visible.kind_edge.x,
                hop_count,
                path_segment_flags(segment_slot, has_start_extension, has_end_extension),
            ),
            vec4<f32>(owner_t1, 1.0, total_length, importance),
        );
    }
}
