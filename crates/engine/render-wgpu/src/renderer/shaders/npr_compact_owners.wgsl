struct GpuNprEdge3d {
    a: u32,
    b: u32,
    face0: u32,
    face1: u32,
    face_count: u32,
    material_seam: u32,
    edge_id: u32,
    next_a: u32,
    next_b: u32,
    degree_a: u32,
    degree_b: u32,
    alt_next_a: u32,
    alt_next_b: u32,
    _pad0: vec2<u32>,
}

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

const KIND_NONE: u32 = 0u;
const KIND_BOUNDARY: u32 = 1u;
const KIND_SILHOUETTE: u32 = 2u;
const KIND_CREASE: u32 = 3u;
const KIND_SEAM: u32 = 4u;
const KIND_FEATURE: u32 = 5u;
const KIND_CONTACT: u32 = 6u;
const PATH_FLAG_EMIT: u32 = 1u;
const PATH_FLAG_CONNECTED_START: u32 = 2u;
const PATH_FLAG_CONNECTED_END: u32 = 4u;

@group(0) @binding(2) var<storage, read> edges: array<GpuNprEdge3d>;
@group(0) @binding(5) var<storage, read_write> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(10) var<storage, read_write> path_links: array<GpuNprPathLink3d>;

fn quantized_anchor_bin(point: vec2<f32>) -> vec2<i32> {
    let quant = max(uniforms.params12.w, 0.5);
    return vec2<i32>(round(point / quant));
}

fn same_anchor_bin(a: vec2<f32>, b: vec2<f32>) -> bool {
    let qa = quantized_anchor_bin(a);
    let qb = quantized_anchor_bin(b);
    return qa.x == qb.x && qa.y == qb.y;
}

fn visible_segment_length(edge_index: u32) -> f32 {
    if (edge_index == 0xffffffffu || edge_index >= u32(arrayLength(&visible_segments))) {
        return 0.0;
    }
    let visible = visible_segments[edge_index];
    if (visible.kind_edge.x == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        return 0.0;
    }
    return distance(visible.start.xy, visible.end.xy);
}

fn npr_gpu_max_chain_angle_cos() -> f32 {
    return clamp(uniforms.params15.x, -1.0, 1.0);
}

fn max_chain_degree_for_kind(kind: u32) -> u32 {
    return select(2u, 3u, kind == KIND_SILHOUETTE);
}

fn anchor_degree(edge: GpuNprEdge3d, anchor_point: vec2<f32>, edge_index: u32) -> u32 {
    let visible = visible_segments[edge_index];
    let distance_start = distance(anchor_point, visible.start.xy);
    let distance_end = distance(anchor_point, visible.end.xy);
    return select(edge.degree_a, edge.degree_b, distance_end < distance_start);
}

fn degree_penalty(edge: GpuNprEdge3d) -> f32 {
    return f32(max(edge.degree_a, 1u) - 1u + max(edge.degree_b, 1u) - 1u) * 1.35;
}

fn representative_edge_score(
    edge_index: u32,
    current_length: f32,
    connection_score: f32,
) -> f32 {
    if (edge_index == 0xffffffffu || edge_index >= u32(arrayLength(&visible_segments))) {
        return -1e9;
    }
    let edge_length = visible_segment_length(edge_index);
    if (edge_length <= 0.0) {
        return -1e9;
    }
    let edge = edges[edge_index];
    return max(edge_length, current_length * 0.55) + connection_score * 16.0 - degree_penalty(edge);
}

fn continuation_endpoint(anchor_point: vec2<f32>, next_edge_index: u32) -> vec2<f32> {
    let next = visible_segments[next_edge_index];
    let distance_start = distance(anchor_point, next.start.xy);
    let distance_end = distance(anchor_point, next.end.xy);
    return select(next.start.xy, next.end.xy, distance_start < distance_end);
}

fn edge_connection_score(
    edge_index: u32,
    next_edge_index: u32,
    anchor_point: vec2<f32>,
    current_direction: vec2<f32>,
    current_length: f32,
) -> f32 {
    if (next_edge_index == 0xffffffffu || next_edge_index >= u32(arrayLength(&visible_segments))) {
        return 0.0;
    }
    let visible = visible_segments[edge_index];
    let next = visible_segments[next_edge_index];
    if (
        next.kind_edge.x == KIND_NONE
        || next.kind_edge.x != visible.kind_edge.x
        || next.start.w <= 0.5
        || next.end.w <= 0.5
    ) {
        return 0.0;
    }
    let continuation = continuation_endpoint(anchor_point, next_edge_index);
    let gap = distance(anchor_point, continuation);
    let at_degree = anchor_degree(edges[edge_index], anchor_point, edge_index);
    if (at_degree > max_chain_degree_for_kind(visible.kind_edge.x)) {
        return 0.0;
    }
    let endpoint_snap = max(uniforms.params12.w, 0.5);
    if (!same_anchor_bin(anchor_point, continuation) && gap > endpoint_snap * 1.6) {
        return 0.0;
    }
    let next_far = select(next.start.xy, next.end.xy, distance(anchor_point, next.start.xy) < distance(anchor_point, next.end.xy));
    let delta = next_far - continuation;
    let next_length = max(length(delta), 0.0001);
    let next_dir = delta / next_length;
    let alignment = dot(current_direction, next_dir);
    if (alignment <= npr_gpu_max_chain_angle_cos()) {
        return 0.0;
    }
    let length_ratio = visible_segment_length(next_edge_index) / max(current_length, 1.0);
    return alignment + clamp(length_ratio, 0.0, 2.0) * 0.18;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= u32(arrayLength(&visible_segments))) {
        return;
    }
    visible_segments[edge_index].kind_edge.w = 0u;
    path_links[edge_index] = GpuNprPathLink3d(
        edge_index,
        0xffffffffu,
        0xffffffffu,
        0u,
    );

    let visible = visible_segments[edge_index];
    if (visible.kind_edge.x == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        return;
    }

    let edge = edges[edge_index];
    let current_length = visible_segment_length(edge_index);
    if (current_length <= 0.0) {
        return;
    }

    let start_dir = normalize(visible.start.xy - visible.end.xy);
    let end_dir = normalize(visible.end.xy - visible.start.xy);
    let start_primary = edge_connection_score(edge_index, edge.next_a, visible.start.xy, start_dir, current_length);
    let start_alt = edge_connection_score(edge_index, edge.alt_next_a, visible.start.xy, start_dir, current_length);
    let end_primary = edge_connection_score(edge_index, edge.next_b, visible.end.xy, end_dir, current_length);
    let end_alt = edge_connection_score(edge_index, edge.alt_next_b, visible.end.xy, end_dir, current_length);
    let start_next_edge = select(edge.next_a, edge.alt_next_a, start_alt > start_primary);
    let end_next_edge = select(edge.next_b, edge.alt_next_b, end_alt > end_primary);
    let start_score = max(start_primary, start_alt);
    let end_score = max(end_primary, end_alt);
    let connected_both = start_score > 0.72 && end_score > 0.72;
    let kind = visible.kind_edge.x;

    if (kind == KIND_CONTACT && (!connected_both || current_length < max(uniforms.params0.w * 3.0, 8.0))) {
        return;
    }

    if (
        (kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE)
        && start_score <= 0.0
        && end_score <= 0.0
        && current_length < max(uniforms.params0.w * 2.5, 7.0)
    ) {
        return;
    }

    let primary_line =
        kind == KIND_BOUNDARY
        || kind == KIND_SILHOUETTE
        || kind == KIND_CREASE
        || kind == KIND_SEAM
        || kind == KIND_FEATURE;

    let start_neighbor = max(visible_segment_length(edge.next_a), visible_segment_length(edge.alt_next_a));
    let end_neighbor = max(visible_segment_length(edge.next_b), visible_segment_length(edge.alt_next_b));
    let chain_span = current_length + max(start_neighbor, 0.0) + max(end_neighbor, 0.0);
    let chain_compactable =
        connected_both
        && chain_span >= max(current_length * 1.18, current_length + 5.0);

    let self_score = representative_edge_score(edge_index, current_length, max(start_score, end_score));
    let score_start_primary = representative_edge_score(edge.next_a, current_length, start_primary);
    let score_start_alt = representative_edge_score(edge.alt_next_a, current_length, start_alt);
    let score_end_primary = representative_edge_score(edge.next_b, current_length, end_primary);
    let score_end_alt = representative_edge_score(edge.alt_next_b, current_length, end_alt);
    var best_owner = edge_index;
    var best_owner_score = self_score;
    if (score_start_primary > best_owner_score + 0.001 || (abs(score_start_primary - best_owner_score) <= 0.001 && edge.next_a < best_owner)) {
        best_owner = edge.next_a;
        best_owner_score = score_start_primary;
    }
    if (score_start_alt > best_owner_score + 0.001 || (abs(score_start_alt - best_owner_score) <= 0.001 && edge.alt_next_a < best_owner)) {
        best_owner = edge.alt_next_a;
        best_owner_score = score_start_alt;
    }
    if (score_end_primary > best_owner_score + 0.001 || (abs(score_end_primary - best_owner_score) <= 0.001 && edge.next_b < best_owner)) {
        best_owner = edge.next_b;
        best_owner_score = score_end_primary;
    }
    if (score_end_alt > best_owner_score + 0.001 || (abs(score_end_alt - best_owner_score) <= 0.001 && edge.alt_next_b < best_owner)) {
        best_owner = edge.alt_next_b;
        best_owner_score = score_end_alt;
    }

    if (primary_line) {
        var flags = PATH_FLAG_EMIT;
        if (start_score > 0.72) {
            flags = flags | PATH_FLAG_CONNECTED_START;
        }
        if (end_score > 0.72) {
            flags = flags | PATH_FLAG_CONNECTED_END;
        }
        path_links[edge_index] = GpuNprPathLink3d(
            best_owner,
            start_next_edge,
            end_next_edge,
            flags,
        );
        visible_segments[edge_index].kind_edge.w = 1u;
        return;
    }

    let stronger_neighbor_owner =
        best_owner != edge_index
        && best_owner_score >= max(self_score * 1.08, self_score + 1.0)
        && chain_span >= max(current_length * 1.08, current_length + 4.0);
    if (stronger_neighbor_owner) {
        return;
    }

    if (chain_compactable) {
        if (best_owner != edge_index) {
            return;
        }
    }

    var flags = PATH_FLAG_EMIT;
    if (start_score > 0.72) {
        flags = flags | PATH_FLAG_CONNECTED_START;
    }
    if (end_score > 0.72) {
        flags = flags | PATH_FLAG_CONNECTED_END;
    }
    path_links[edge_index] = GpuNprPathLink3d(
        best_owner,
        start_next_edge,
        end_next_edge,
        flags,
    );
    visible_segments[edge_index].kind_edge.w = 1u;
}
