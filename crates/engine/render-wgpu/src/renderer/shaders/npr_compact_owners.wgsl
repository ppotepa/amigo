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
    metrics: vec4<f32>,
}

struct GpuNprPathLink3d {
    owner_edge: u32,
    start_next: u32,
    end_next: u32,
    flags: u32,
}

struct GpuNprEndpointEntry3d {
    edge_index: u32,
    flags: u32,
    next_plus_one: u32,
    kind: u32,
    bin: vec2<i32>,
    endpoint_vertex: u32,
    _pad0: u32,
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

struct EndpointCandidatePick {
    edge_index: u32,
    score: f32,
    matched_start: bool,
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
const ENDPOINT_FLAG_MATCHED_START: u32 = 1u;
const STROKE_AKIRA_INK: u32 = 1u;
const STROKE_CONFIDENT_MANGA_INK: u32 = 4u;
const MAX_ENDPOINT_BUCKET_SCAN: u32 = 32u;
const SEGMENT_TRAIT_MATERIAL_DETAIL: u32 = 1u;
const SEGMENT_TRAIT_MATERIAL_SEAM: u32 = 2u;

@group(0) @binding(2) var<storage, read> edges: array<GpuNprEdge3d>;
@group(0) @binding(5) var<storage, read_write> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(10) var<storage, read_write> path_links: array<GpuNprPathLink3d>;
@group(0) @binding(11) var<storage, read_write> endpoint_heads: array<atomic<u32>>;
@group(0) @binding(12) var<storage, read_write> endpoint_entries: array<GpuNprEndpointEntry3d>;

fn active_edge_count() -> u32 {
    return min(uniforms.pipeline1.w, u32(arrayLength(&visible_segments)));
}

fn endpoint_quant_px(kind: u32) -> f32 {
    if (confident_primary_contour(kind) && kind == KIND_SILHOUETTE) {
        return clamp(max(uniforms.params40.x * 0.5, uniforms.params12.w), 0.5, 6.0);
    }
    if (confident_primary_contour(kind) && kind == KIND_BOUNDARY) {
        return clamp(max(uniforms.params40.y * 0.5, uniforms.params12.w), 0.5, 6.0);
    }
    return max(uniforms.params12.w, 0.5);
}

fn quantized_anchor_bin(point: vec2<f32>, kind: u32) -> vec2<i32> {
    let quant = endpoint_quant_px(kind);
    return vec2<i32>(round(point / quant));
}

fn endpoint_bucket_index(kind: u32, bin: vec2<i32>) -> u32 {
    let head_count = max(u32(arrayLength(&endpoint_heads)), 1u);
    let hx = bitcast<u32>(bin.x) * 0x9E3779B1u;
    let hy = bitcast<u32>(bin.y) * 0x85EBCA77u;
    let manga_ink =
        uniforms.pipeline0.z == STROKE_AKIRA_INK
        || uniforms.pipeline0.z == STROKE_CONFIDENT_MANGA_INK;
    let primary_kind = kind == KIND_BOUNDARY || kind == KIND_SILHOUETTE;
    let bucket_kind = select(kind, KIND_BOUNDARY, manga_ink && primary_kind);
    let hk = bucket_kind * 0xC2B2AE3Du;
    return (hx ^ hy ^ hk) & (head_count - 1u);
}

fn visible_segment_length(edge_index: u32) -> f32 {
    if (edge_index == 0xffffffffu || edge_index >= active_edge_count()) {
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

fn is_primary_contour(kind: u32) -> bool {
    return kind == KIND_BOUNDARY || kind == KIND_SILHOUETTE;
}

fn is_detail_line(kind: u32) -> bool {
    return kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE || kind == KIND_CONTACT;
}

fn uses_confident_manga_ink() -> bool {
    return uniforms.pipeline0.z == STROKE_CONFIDENT_MANGA_INK;
}

fn uses_akira_ink() -> bool {
    return uniforms.pipeline0.z == STROKE_AKIRA_INK;
}

fn uses_manga_ink() -> bool {
    return uses_akira_ink() || uses_confident_manga_ink();
}

fn confident_primary_contour(kind: u32) -> bool {
    return uses_manga_ink() && is_primary_contour(kind);
}

fn compatible_endpoint_kind(current_kind: u32, candidate_kind: u32) -> bool {
    return current_kind == candidate_kind
        || (uses_manga_ink() && is_primary_contour(current_kind) && is_primary_contour(candidate_kind));
}

fn max_chain_degree_for_kind(kind: u32) -> u32 {
    return select(select(2u, 3u, kind == KIND_SILHOUETTE), 4u, confident_primary_contour(kind));
}

fn connection_score_threshold(kind: u32) -> f32 {
    if (is_detail_line(kind) && uses_akira_ink()) {
        return 0.48;
    }
    return select(select(0.72, 0.64, is_detail_line(kind)), 0.54, confident_primary_contour(kind));
}

fn kind_join_gap_px(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params40.x;
    }
    if (kind == KIND_BOUNDARY) {
        return uniforms.params40.y;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params40.w;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params41.x;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params41.y;
    }
    return uniforms.params40.z;
}

fn kind_chain_angle_cos(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params41.z;
    }
    if (kind == KIND_BOUNDARY) {
        return uniforms.params41.w;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params42.y;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params42.z;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params42.w;
    }
    return uniforms.params42.x;
}

fn required_chain_alignment(kind: u32) -> f32 {
    let global_limit = npr_gpu_max_chain_angle_cos();
    let family_limit = kind_chain_angle_cos(kind);
    if (confident_primary_contour(kind)) {
        return min(global_limit, family_limit);
    }
    return max(global_limit, family_limit);
}

fn kind_technical_detail_keep(kind: u32) -> f32 {
    if (kind == KIND_SEAM) {
        return uniforms.params38.x;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params38.y;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params37.w;
    }
    if (kind == KIND_FEATURE) {
        return uniforms.params37.z;
    }
    return 1.0;
}

fn kind_preferred_stroke_length_px(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params38.z;
    }
    if (kind == KIND_BOUNDARY) {
        return uniforms.params38.w;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params39.y;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params39.z;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params39.w;
    }
    return uniforms.params39.x;
}

fn kind_continuation_bias(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params45.x;
    }
    if (kind == KIND_BOUNDARY) {
        return uniforms.params45.y;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params45.w;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params46.x;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params46.y;
    }
    return uniforms.params45.z;
}

fn kind_breakup_bias(kind: u32) -> f32 {
    if (kind == KIND_CREASE) {
        return uniforms.params46.w;
    }
    if (kind == KIND_FEATURE) {
        return uniforms.params46.z;
    }
    return 0.5;
}

fn kind_technical_detail_preference(kind: u32) -> f32 {
    if (kind == KIND_FEATURE) {
        return uniforms.params47.x;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params47.y;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params47.z;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params47.w;
    }
    return 0.0;
}

fn kind_ink_detail_material_preference(kind: u32) -> f32 {
    if (kind == KIND_FEATURE) {
        return uniforms.params48.x;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params48.y;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params48.z;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params48.w;
    }
    return 0.0;
}

fn kind_material_seam_preference(kind: u32) -> f32 {
    if (kind == KIND_FEATURE) {
        return uniforms.params49.x;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params49.y;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params49.z;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params49.w;
    }
    return 0.0;
}

fn visible_trait_flags(visible: GpuNprVisibleSegment3d) -> u32 {
    return u32(round(visible.metrics.w));
}

fn visible_has_material_detail(visible: GpuNprVisibleSegment3d) -> bool {
    return (visible_trait_flags(visible) & SEGMENT_TRAIT_MATERIAL_DETAIL) != 0u;
}

fn visible_has_material_seam(visible: GpuNprVisibleSegment3d) -> bool {
    return (visible_trait_flags(visible) & SEGMENT_TRAIT_MATERIAL_SEAM) != 0u;
}

fn kind_trait_strength(kind: u32, visible: GpuNprVisibleSegment3d) -> f32 {
    var strength = kind_technical_detail_preference(kind) * 0.35;
    if (visible_has_material_detail(visible)) {
        strength = strength + kind_ink_detail_material_preference(kind) * 0.45;
    }
    if (visible_has_material_seam(visible) || kind == KIND_SEAM) {
        strength = strength + kind_material_seam_preference(kind) * 0.55;
    }
    return clamp(strength, 0.0, 1.0);
}

fn degree_penalty(edge: GpuNprEdge3d) -> f32 {
    return f32(max(edge.degree_a, 1u) - 1u + max(edge.degree_b, 1u) - 1u) * 1.35;
}

fn representative_edge_score(edge_index: u32, current_kind: u32, current_length: f32, connection_score: f32) -> f32 {
    if (edge_index == 0xffffffffu || edge_index >= active_edge_count()) {
        return -1e9;
    }
    let visible = visible_segments[edge_index];
    let edge_length = visible_segment_length(edge_index);
    if (edge_length <= 0.0) {
        return -1e9;
    }
    let edge = edges[edge_index];
    let trait_strength = kind_trait_strength(current_kind, visible);
    let preferred_length = max(
        kind_preferred_stroke_length_px(current_kind) * clamp(0.90 + trait_strength * 0.22, 0.72, 1.24),
        16.0,
    );
    let length_ratio = clamp(edge_length / preferred_length, 0.0, 1.35);
    let target_closeness = 1.0 - clamp(abs(edge_length - preferred_length) / preferred_length, 0.0, 1.0);
    let candidate_importance = clamp(visible.metrics.z, 0.0, 1.25);
    let detail_keep = clamp(kind_technical_detail_keep(current_kind) + trait_strength * 0.22, 0.0, 1.0);
    let continuation_bias = clamp(kind_continuation_bias(current_kind) + trait_strength * 0.24, 0.0, 1.0);
    let breakup_bias = clamp(kind_breakup_bias(current_kind) - trait_strength * 0.20, 0.0, 1.0);
    let detail_short_penalty =
        select(
            0.0,
            clamp((preferred_length * 0.58 - edge_length) / max(preferred_length * 0.58, 1.0), 0.0, 1.0)
                * (1.0 - detail_keep)
                * (6.0 + breakup_bias * 4.5),
            is_detail_line(current_kind),
        );
    let contour_length_bonus =
        select(
            0.0,
            clamp(length_ratio, 0.0, 1.0) * (5.5 + continuation_bias * 4.5),
            is_primary_contour(current_kind),
        );
    let base_span = max(edge_length, current_length * 0.55);
    return base_span
        + connection_score * (10.0 + continuation_bias * 4.0 - breakup_bias * 2.0)
        + target_closeness * 9.0
        + candidate_importance * (6.0 + trait_strength * 1.8)
        + contour_length_bonus
        - detail_short_penalty
        - degree_penalty(edge);
}

fn entry_is_matched_start(entry: GpuNprEndpointEntry3d) -> bool {
    return (entry.flags & ENDPOINT_FLAG_MATCHED_START) != 0u;
}

fn matched_point_for_entry(entry: GpuNprEndpointEntry3d, visible: GpuNprVisibleSegment3d) -> vec2<f32> {
    return select(visible.end.xy, visible.start.xy, entry_is_matched_start(entry));
}

fn far_point_for_entry(entry: GpuNprEndpointEntry3d, visible: GpuNprVisibleSegment3d) -> vec2<f32> {
    return select(visible.start.xy, visible.end.xy, entry_is_matched_start(entry));
}

fn matched_depth_for_entry(entry: GpuNprEndpointEntry3d, visible: GpuNprVisibleSegment3d) -> f32 {
    return select(visible.end.z, visible.start.z, entry_is_matched_start(entry));
}

fn endpoint_degree_for_entry(edge: GpuNprEdge3d, entry: GpuNprEndpointEntry3d) -> u32 {
    return select(edge.degree_b, edge.degree_a, entry_is_matched_start(entry));
}

fn valid_endpoint_vertex(vertex: u32) -> bool {
    return vertex != 0xffffffffu;
}

fn visible_endpoint_vertex(visible: GpuNprVisibleSegment3d, matched_start: bool) -> u32 {
    return select(visible.kind_edge.w, visible.kind_edge.z, matched_start);
}

fn edge_connection_score_from_entry(
    current_edge_index: u32,
    current_kind: u32,
    current_endpoint_vertex: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
    candidate_entry_index: u32,
) -> f32 {
    let allow_screen_space_endpoint = confident_primary_contour(current_kind);
    if (!valid_endpoint_vertex(current_endpoint_vertex) && !allow_screen_space_endpoint) {
        return 0.0;
    }
    if (candidate_entry_index >= u32(arrayLength(&endpoint_entries))) {
        return 0.0;
    }
    let candidate_entry = endpoint_entries[candidate_entry_index];
    let next_edge_index = candidate_entry.edge_index;
    if (
        next_edge_index == 0xffffffffu
        || next_edge_index == current_edge_index
        || next_edge_index >= active_edge_count()
    ) {
        return 0.0;
    }
    if (!compatible_endpoint_kind(current_kind, candidate_entry.kind)) {
        return 0.0;
    }
    if (!valid_endpoint_vertex(candidate_entry.endpoint_vertex) && !allow_screen_space_endpoint) {
        return 0.0;
    }
    if (
        valid_endpoint_vertex(candidate_entry.endpoint_vertex)
        && valid_endpoint_vertex(current_endpoint_vertex)
        && candidate_entry.endpoint_vertex != current_endpoint_vertex
        && !allow_screen_space_endpoint
    ) {
        return 0.0;
    }

    let next = visible_segments[next_edge_index];
    if (next.kind_edge.x == KIND_NONE || next.start.w <= 0.5 || next.end.w <= 0.5) {
        return 0.0;
    }

    let continuation = matched_point_for_entry(candidate_entry, next);
    let gap = distance(anchor_point, continuation);
    let endpoint_snap = max(uniforms.params12.w, 0.5);
    let join_gap_px = max(kind_join_gap_px(current_kind), endpoint_snap);
    let confident_primary = confident_primary_contour(current_kind);
    if (gap > join_gap_px * select(1.0, 1.15, confident_primary)) {
        return 0.0;
    }

    let current_bin = quantized_anchor_bin(anchor_point, current_kind);
    let bin_gap = f32(abs(candidate_entry.bin.x - current_bin.x) + abs(candidate_entry.bin.y - current_bin.y));
    if (bin_gap > select(2.0, 3.0, confident_primary) && gap > join_gap_px * select(0.8, 1.2, confident_primary)) {
        return 0.0;
    }

    let next_edge = edges[next_edge_index];
    let degree = endpoint_degree_for_entry(next_edge, candidate_entry);
    if (!allow_screen_space_endpoint && degree > max_chain_degree_for_kind(current_kind)) {
        return 0.0;
    }

    let next_far = far_point_for_entry(candidate_entry, next);
    let delta = next_far - continuation;
    let next_length = max(length(delta), 0.0001);
    let next_dir = delta / next_length;
    let alignment = dot(current_direction, next_dir);
    let required_alignment = required_chain_alignment(current_kind);
    if (alignment <= required_alignment) {
        return 0.0;
    }

    let depth_gap = abs(anchor_depth - matched_depth_for_entry(candidate_entry, next));
    let visible_length = visible_segment_length(next_edge_index);
    let length_ratio = visible_length / max(current_length, 1.0);
    let balanced_length_ratio = min(length_ratio, 1.0 / max(length_ratio, 0.0001));
    let detail_alignment_floor = select(0.86, 0.76, uses_akira_ink());
    if (is_detail_line(current_kind) && alignment < max(required_alignment, detail_alignment_floor)) {
        return 0.0;
    }
    let detail_length_ratio_floor = select(0.42, 0.30, uses_akira_ink());
    if (is_detail_line(current_kind) && balanced_length_ratio < detail_length_ratio_floor) {
        return 0.0;
    }
    let detail_depth_gap_max = select(0.035, 0.060, uses_akira_ink());
    if (is_detail_line(current_kind) && depth_gap > detail_depth_gap_max) {
        return 0.0;
    }
    let cost =
        (gap / max(join_gap_px, 1.0))
        + bin_gap * 0.8
        + (1.0 - clamp(alignment, -1.0, 1.0)) * select(14.0, 9.5, confident_primary)
        + depth_gap * select(2.1, 5.8, allow_screen_space_endpoint)
        + abs(current_length - visible_length) / max(max(current_length, visible_length), 1.0) * 3.2
        + select(0.0, 0.85, degree != 1u && !allow_screen_space_endpoint);
    let raw_score = (1.0 / (1.0 + cost * 0.18)) + clamp(length_ratio, 0.0, 2.0) * 0.18;
    let detail_scale = select(1.0, clamp((alignment - 0.82) * 4.0, 0.0, 1.0), is_detail_line(current_kind));
    return raw_score * detail_scale;
}

fn edge_connection_score_from_topology(
    current_edge_index: u32,
    current_kind: u32,
    current_endpoint_vertex: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
    next_edge_index: u32,
) -> EndpointCandidatePick {
    if (
        !valid_endpoint_vertex(current_endpoint_vertex)
        || next_edge_index == 0xffffffffu
        || next_edge_index == current_edge_index
        || next_edge_index >= active_edge_count()
    ) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let next = visible_segments[next_edge_index];
    if (
        !compatible_endpoint_kind(current_kind, next.kind_edge.x)
        || next.start.w <= 0.5
        || next.end.w <= 0.5
    ) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let next_edge = edges[next_edge_index];
    let topology_start = next_edge.a == current_endpoint_vertex;
    let topology_end = next_edge.b == current_endpoint_vertex;
    let matched_start = select(
        select(
            distance(anchor_point, next.start.xy) <= distance(anchor_point, next.end.xy),
            false,
            topology_end,
        ),
        true,
        topology_start,
    );
    let topology_matches = topology_start || topology_end;
    if (!topology_matches) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let continuation = select(next.end.xy, next.start.xy, matched_start);
    let gap = distance(anchor_point, continuation);
    let endpoint_snap = max(uniforms.params12.w, 0.5);
    let join_gap_px = max(kind_join_gap_px(current_kind), endpoint_snap);
    if (gap > join_gap_px) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let next_far = select(next.start.xy, next.end.xy, matched_start);
    let delta = next_far - continuation;
    let next_length = max(length(delta), 0.0001);
    let next_dir = delta / next_length;
    let alignment = dot(current_direction, next_dir);
    let chain_alignment = required_chain_alignment(current_kind);
    let alignment_floor = select(chain_alignment, max(chain_alignment, 0.72), is_detail_line(current_kind));
    if (alignment <= alignment_floor) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let depth_gap = abs(anchor_depth - select(next.end.z, next.start.z, matched_start));
    let max_depth_gap = select(0.055, 0.075, uses_akira_ink() && is_detail_line(current_kind));
    if (depth_gap > max_depth_gap) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let visible_length = visible_segment_length(next_edge_index);
    let length_ratio = visible_length / max(current_length, 1.0);
    let balanced_length_ratio = min(length_ratio, 1.0 / max(length_ratio, 0.0001));
    if (is_detail_line(current_kind) && balanced_length_ratio < select(0.38, 0.28, uses_akira_ink())) {
        return EndpointCandidatePick(0xffffffffu, 0.0, false);
    }

    let score =
        0.72
        + clamp(alignment, 0.0, 1.0) * 0.28
        - (gap / max(join_gap_px, 1.0)) * 0.18
        - depth_gap * 2.0
        + clamp(length_ratio, 0.0, 1.5) * 0.06;
    return EndpointCandidatePick(next_edge_index, max(score, 0.0), matched_start);
}

fn better_candidate(current: EndpointCandidatePick, candidate: EndpointCandidatePick) -> EndpointCandidatePick {
    if (candidate.score > current.score + 0.001) {
        return candidate;
    }
    if (
        abs(candidate.score - current.score) <= 0.001
        && candidate.edge_index != 0xffffffffu
        && candidate.edge_index < current.edge_index
    ) {
        return candidate;
    }
    return current;
}

fn adopt_owner_if_stable(
    current_owner: u32,
    current_owner_score: f32,
    candidate_owner: u32,
    candidate_score: f32,
    tolerance: f32,
) -> u32 {
    if (candidate_owner == 0xffffffffu) {
        return current_owner;
    }
    if (candidate_score + tolerance < current_owner_score) {
        return current_owner;
    }
    if (candidate_owner < current_owner) {
        return candidate_owner;
    }
    return current_owner;
}

fn scan_bucket_candidates(
    current_edge_index: u32,
    current_kind: u32,
    current_endpoint_vertex: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
    bin: vec2<i32>,
    current_best: EndpointCandidatePick,
) -> EndpointCandidatePick {
    let bucket_index = endpoint_bucket_index(current_kind, bin);
    var best = current_best;
    var head_plus_one = atomicLoad(&endpoint_heads[bucket_index]);
    var scanned = 0u;
    loop {
        if (head_plus_one == 0u || scanned >= MAX_ENDPOINT_BUCKET_SCAN) {
            break;
        }
        scanned = scanned + 1u;
        let entry_index = head_plus_one - 1u;
        if (entry_index >= u32(arrayLength(&endpoint_entries))) {
            break;
        }
        let entry = endpoint_entries[entry_index];
        let score = edge_connection_score_from_entry(
            current_edge_index,
            current_kind,
            current_endpoint_vertex,
            anchor_point,
            anchor_depth,
            current_direction,
            current_length,
            entry_index,
        );
        let score_threshold = select(0.05, select(0.58, 0.36, uses_akira_ink()), is_detail_line(current_kind));
        if (score >= score_threshold) {
            best = better_candidate(
                best,
                EndpointCandidatePick(entry.edge_index, score, entry_is_matched_start(entry)),
            );
        }
        head_plus_one = entry.next_plus_one;
    }
    return best;
}

fn best_endpoint_candidate_from_bins(
    current_edge_index: u32,
    current_kind: u32,
    current_endpoint_vertex: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
) -> EndpointCandidatePick {
    let current_edge = edges[current_edge_index];
    let from_a = valid_endpoint_vertex(current_endpoint_vertex) && current_endpoint_vertex == current_edge.a;
    let from_b = valid_endpoint_vertex(current_endpoint_vertex) && current_endpoint_vertex == current_edge.b;
    var best = EndpointCandidatePick(0xffffffffu, 0.0, false);
    if (from_a) {
        best = better_candidate(
            best,
            edge_connection_score_from_topology(
                current_edge_index,
                current_kind,
                current_endpoint_vertex,
                anchor_point,
                anchor_depth,
                current_direction,
                current_length,
                current_edge.next_a,
            ),
        );
        best = better_candidate(
            best,
            edge_connection_score_from_topology(
                current_edge_index,
                current_kind,
                current_endpoint_vertex,
                anchor_point,
                anchor_depth,
                current_direction,
                current_length,
                current_edge.alt_next_a,
            ),
        );
    }
    if (from_b) {
        best = better_candidate(
            best,
            edge_connection_score_from_topology(
                current_edge_index,
                current_kind,
                current_endpoint_vertex,
                anchor_point,
                anchor_depth,
                current_direction,
                current_length,
                current_edge.next_b,
            ),
        );
        best = better_candidate(
            best,
            edge_connection_score_from_topology(
                current_edge_index,
                current_kind,
                current_endpoint_vertex,
                anchor_point,
                anchor_depth,
                current_direction,
                current_length,
                current_edge.alt_next_b,
            ),
        );
    }
    let base_bin = quantized_anchor_bin(anchor_point, current_kind);
    for (var offset_y: i32 = -1; offset_y <= 1; offset_y = offset_y + 1) {
        for (var offset_x: i32 = -1; offset_x <= 1; offset_x = offset_x + 1) {
            best = scan_bucket_candidates(
                current_edge_index,
                current_kind,
                current_endpoint_vertex,
                anchor_point,
                anchor_depth,
                current_direction,
                current_length,
                base_bin + vec2<i32>(offset_x, offset_y),
                best,
            );
        }
    }
    return best;
}

fn reciprocal_connection_ok(
    current_edge_index: u32,
    current_kind: u32,
    candidate_pick: EndpointCandidatePick,
) -> bool {
    if (
        candidate_pick.edge_index == 0xffffffffu
        || candidate_pick.edge_index >= active_edge_count()
    ) {
        return false;
    }
    let candidate_visible = visible_segments[candidate_pick.edge_index];
    if (
        !compatible_endpoint_kind(current_kind, candidate_visible.kind_edge.x)
        || candidate_visible.start.w <= 0.5
        || candidate_visible.end.w <= 0.5
    ) {
        return false;
    }

    let anchor_point = select(
        candidate_visible.end.xy,
        candidate_visible.start.xy,
        candidate_pick.matched_start,
    );
    let anchor_vertex = select(
        candidate_visible.kind_edge.w,
        candidate_visible.kind_edge.z,
        candidate_pick.matched_start,
    );
    let allow_screen_space_endpoint = confident_primary_contour(current_kind);
    if (!valid_endpoint_vertex(anchor_vertex) && !allow_screen_space_endpoint) {
        return false;
    }
    let anchor_depth = select(
        candidate_visible.end.z,
        candidate_visible.start.z,
        candidate_pick.matched_start,
    );
    let far_point = select(
        candidate_visible.start.xy,
        candidate_visible.end.xy,
        candidate_pick.matched_start,
    );
    let candidate_length = visible_segment_length(candidate_pick.edge_index);
    if (candidate_length <= 0.0) {
        return false;
    }
    let direction = normalize(anchor_point - far_point);
    let back_pick = best_endpoint_candidate_from_bins(
        candidate_pick.edge_index,
        current_kind,
        anchor_vertex,
        anchor_point,
        anchor_depth,
        direction,
        candidate_length,
    );
    let reciprocal_scale = select(0.92, 0.78, allow_screen_space_endpoint);
    return back_pick.edge_index == current_edge_index && back_pick.score >= connection_score_threshold(current_kind) * reciprocal_scale;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= active_edge_count()) {
        return;
    }

    path_links[edge_index] = GpuNprPathLink3d(edge_index, 0xffffffffu, 0xffffffffu, 0u);

    let visible = visible_segments[edge_index];
    if (visible.kind_edge.x == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        return;
    }

    let kind = visible.kind_edge.x;
    let start_endpoint_vertex = visible_endpoint_vertex(visible, true);
    let end_endpoint_vertex = visible_endpoint_vertex(visible, false);
    let current_length = visible_segment_length(edge_index);
    if (current_length <= 0.0) {
        return;
    }

    let start_pick = best_endpoint_candidate_from_bins(
        edge_index,
        kind,
        start_endpoint_vertex,
        visible.start.xy,
        visible.start.z,
        normalize(visible.start.xy - visible.end.xy),
        current_length,
    );
    let end_pick = best_endpoint_candidate_from_bins(
        edge_index,
        kind,
        end_endpoint_vertex,
        visible.end.xy,
        visible.end.z,
        normalize(visible.end.xy - visible.start.xy),
        current_length,
    );

    let connect_threshold = connection_score_threshold(kind);
    let start_screen_space =
        confident_primary_contour(kind) && !valid_endpoint_vertex(start_endpoint_vertex);
    let end_screen_space =
        confident_primary_contour(kind) && !valid_endpoint_vertex(end_endpoint_vertex);
    let connected_start =
        start_pick.score >= connect_threshold
        && (
            reciprocal_connection_ok(edge_index, kind, start_pick)
            || (start_screen_space && start_pick.score >= connect_threshold * 1.16)
        );
    let connected_end =
        end_pick.score >= connect_threshold
        && (
            reciprocal_connection_ok(edge_index, kind, end_pick)
            || (end_screen_space && end_pick.score >= connect_threshold * 1.16)
        );
    let connected_both = connected_start && connected_end;

    if (kind == KIND_CONTACT && (!connected_both || current_length < max(uniforms.params0.w * 3.0, 8.0))) {
        return;
    }

    if (
        (kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE)
        && !connected_start
        && !connected_end
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

    let start_neighbor = select(0.0, visible_segment_length(start_pick.edge_index), start_pick.edge_index != 0xffffffffu);
    let end_neighbor = select(0.0, visible_segment_length(end_pick.edge_index), end_pick.edge_index != 0xffffffffu);
    let chain_span = current_length + start_neighbor + end_neighbor;
    let chain_compactable =
        connected_both
        && chain_span >= max(current_length * 1.18, current_length + 5.0);

    let self_score = representative_edge_score(edge_index, kind, current_length, max(start_pick.score, end_pick.score));
    let score_start = representative_edge_score(start_pick.edge_index, kind, current_length, start_pick.score);
    let score_end = representative_edge_score(end_pick.edge_index, kind, current_length, end_pick.score);
    var best_owner = edge_index;
    var best_owner_score = self_score;
    if (score_start > best_owner_score + 0.001 || (abs(score_start - best_owner_score) <= 0.001 && start_pick.edge_index < best_owner)) {
        best_owner = start_pick.edge_index;
        best_owner_score = score_start;
    }
    if (score_end > best_owner_score + 0.001 || (abs(score_end - best_owner_score) <= 0.001 && end_pick.edge_index < best_owner)) {
        best_owner = end_pick.edge_index;
        best_owner_score = score_end;
    }

    if (connected_both && chain_compactable) {
        let owner_tolerance = max(current_length * 0.06, 0.9);
        var canonical_owner = best_owner;
        canonical_owner = adopt_owner_if_stable(
            canonical_owner,
            best_owner_score,
            edge_index,
            self_score,
            owner_tolerance,
        );
        canonical_owner = adopt_owner_if_stable(
            canonical_owner,
            best_owner_score,
            start_pick.edge_index,
            score_start,
            owner_tolerance,
        );
        canonical_owner = adopt_owner_if_stable(
            canonical_owner,
            best_owner_score,
            end_pick.edge_index,
            score_end,
            owner_tolerance,
        );
        best_owner = canonical_owner;
    }

    if (!primary_line) {
        let stronger_neighbor_owner =
            best_owner != edge_index
            && best_owner_score >= max(self_score * 1.08, self_score + 1.0)
            && chain_span >= max(current_length * 1.08, current_length + 4.0);
        if (stronger_neighbor_owner) {
            return;
        }
        if (chain_compactable && best_owner != edge_index) {
            return;
        }
    }

    var flags = PATH_FLAG_EMIT;
    if (connected_start) {
        flags = flags | PATH_FLAG_CONNECTED_START;
    }
    if (connected_end) {
        flags = flags | PATH_FLAG_CONNECTED_END;
    }

    path_links[edge_index] = GpuNprPathLink3d(
        best_owner,
        start_pick.edge_index,
        end_pick.edge_index,
        flags,
    );
}
