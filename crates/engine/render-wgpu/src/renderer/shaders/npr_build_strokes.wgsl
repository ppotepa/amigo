struct GpuNprVertex3d {
    position: vec4<f32>,
}

struct GpuNprTriangle3d {
    indices: vec4<u32>,
    normal: vec4<f32>,
    material_id: u32,
    _pad0: vec3<u32>,
}

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

struct GpuNprProjectedVertex3d {
    ndc_depth: vec4<f32>,
    screen: vec4<f32>,
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

struct NprStrokeSegmentVertex {
    start: vec2<f32>,
    end: vec2<f32>,
    color: vec4<f32>,
    width_px: f32,
    offset_px: f32,
    overshoot_start_px: f32,
    overshoot_end_px: f32,
    viewport_half: vec2<f32>,
    end_width_px: f32,
    end_alpha: f32,
}

struct ChainOwnerPick {
    owner: u32,
    score: f32,
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

@group(0) @binding(0) var<storage, read> vertices: array<GpuNprVertex3d>;
@group(0) @binding(1) var<storage, read> triangles: array<GpuNprTriangle3d>;
@group(0) @binding(2) var<storage, read> edges: array<GpuNprEdge3d>;
@group(0) @binding(3) var<storage, read_write> projected_vertices: array<GpuNprProjectedVertex3d>;
@group(0) @binding(5) var<storage, read_write> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(6) var<storage, read_write> stroke_segments: array<NprStrokeSegmentVertex>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(9) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(10) var<storage, read_write> path_links: array<GpuNprPathLink3d>;

fn rotate_euler(v: vec3<f32>, rotation: vec3<f32>) -> vec3<f32> {
    let cx = cos(rotation.x);
    let sx = sin(rotation.x);
    let cy = cos(rotation.y);
    let sy = sin(rotation.y);
    let cz = cos(rotation.z);
    let sz = sin(rotation.z);

    let rx = vec3<f32>(v.x, v.y * cx - v.z * sx, v.y * sx + v.z * cx);
    let ry = vec3<f32>(rx.x * cy + rx.z * sy, rx.y, -rx.x * sy + rx.z * cy);
    return vec3<f32>(ry.x * cz - ry.y * sz, ry.x * sz + ry.y * cz, ry.z);
}

fn transform_vertex(vertex_index: u32) -> vec3<f32> {
    let local = vertices[vertex_index].position.xyz * uniforms.model_scale.xyz;
    return rotate_euler(local, uniforms.model_rotation.xyz) + uniforms.model_translation.xyz;
}

fn transformed_normal(face_index: u32) -> vec3<f32> {
    if (face_index >= u32(arrayLength(&triangles))) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let triangle = triangles[face_index];
    let a = transform_vertex(triangle.indices.x);
    let b = transform_vertex(triangle.indices.y);
    let c = transform_vertex(triangle.indices.z);
    return normalize(cross(b - a, c - a));
}

fn triangle_center(face_index: u32) -> vec3<f32> {
    if (face_index >= u32(arrayLength(&triangles))) {
        return uniforms.model_translation.xyz;
    }
    let triangle = triangles[face_index];
    let a = transform_vertex(triangle.indices.x);
    let b = transform_vertex(triangle.indices.y);
    let c = transform_vertex(triangle.indices.z);
    return (a + b + c) / 3.0;
}

fn triangle_front(face_index: u32) -> bool {
    let world_normal = transformed_normal(face_index);
    let to_camera = normalize(uniforms.camera_translation.xyz - triangle_center(face_index));
    return dot(world_normal, to_camera) > 0.0;
}

fn feature_edge(edge: GpuNprEdge3d) -> bool {
    if (edge.face_count < 2u || edge.face0 >= u32(arrayLength(&triangles)) || edge.face1 >= u32(arrayLength(&triangles))) {
        return false;
    }
    return dot(transformed_normal(edge.face0), transformed_normal(edge.face1)) <= uniforms.params2.w;
}

fn kind_width_multiplier(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params3.x;
    }
    if (kind == KIND_CONTACT) {
        return max(uniforms.params3.z, 1.0);
    }
    if (
        kind == KIND_CREASE
        || kind == KIND_SEAM
        || kind == KIND_FEATURE
    ) {
        return uniforms.params3.z;
    }
    return uniforms.params3.y;
}

fn kind_alpha_multiplier(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params4.x;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params4.z * 0.94;
    }
    if (
        kind == KIND_CREASE
        || kind == KIND_SEAM
        || kind == KIND_FEATURE
    ) {
        return uniforms.params4.z * uniforms.params16.y * 0.82;
    }
    return uniforms.params4.y;
}

fn kind_wobble_px(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params12.x;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params12.z * 0.625;
    }
    if (
        kind == KIND_CREASE
        || kind == KIND_SEAM
        || kind == KIND_FEATURE
    ) {
        return uniforms.params12.z;
    }
    return uniforms.params12.y;
}

fn hash_u32(value: u32) -> u32 {
    var x = value;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    return (x >> 16u) ^ x;
}

fn signed_noise_01(seed: u32) -> f32 {
    return f32(hash_u32(seed) & 65535u) / 65535.0;
}

fn signed_noise(seed: u32) -> f32 {
    return signed_noise_01(seed) * 2.0 - 1.0;
}

fn effective_dropout_amount(kind: u32) -> f32 {
    let base = clamp(uniforms.params13.w * uniforms.params6.w, 0.0, 0.85);
    if (kind == KIND_SILHOUETTE) {
        return 0.0;
    }
    if (kind == KIND_BOUNDARY) {
        return base * 0.35;
    }
    if (kind == KIND_CONTACT) {
        return base * 0.6;
    }
    return base;
}

fn should_drop_segment_instance(
    kind: u32,
    pass_index: u32,
    edge_id: u32,
    render_length: f32,
) -> bool {
    if (pass_index >= u32(uniforms.params5.x)) {
        return false;
    }
    let effective = effective_dropout_amount(kind);
    if (effective <= 0.0) {
        return false;
    }
    let min_segment_px = max(uniforms.params0.w * 4.0, 2.0);
    if (render_length < min_segment_px) {
        return false;
    }
    let coverage = clamp(render_length / max(min_segment_px * 4.0, 1.0), 0.25, 1.0);
    let chance = effective * coverage;
    let roll = signed_noise_01(
        uniforms.seed.x ^ uniforms.seed.y ^ edge_id ^ (pass_index * 92821u) ^ 0xA5D3u
    );
    return roll < chance;
}

fn connection_offset_multiplier(
    connected_start: bool,
    connected_end: bool,
    pass_index: u32,
    render_length: f32,
) -> f32 {
    let is_search = pass_index >= u32(uniforms.params5.x);
    let start_lock_px = max(bitcast<f32>(uniforms.seed.z), 0.0);
    let end_lock_px = max(bitcast<f32>(uniforms.seed.w), 0.0);
    let lock_span = clamp((start_lock_px + end_lock_px) / max(render_length, 1.0), 0.0, 1.0);
    let lock_factor = 1.0 - lock_span * 0.45;
    if (connected_start && connected_end) {
        return select(0.12, 0.25, is_search) * lock_factor;
    }
    if (connected_start || connected_end) {
        return select(0.35, 0.55, is_search) * lock_factor;
    }
    return lock_factor;
}

fn endpoint_tangent_drift_px(
    kind: u32,
    edge_id: u32,
    pass_index: u32,
    connected: bool,
    render_length: f32,
    salt: u32,
    endpoint_lock_px: f32,
) -> f32 {
    if (connected) {
        return 0.0;
    }
    let tangent_scale = uniforms.params11.x;
    if (abs(tangent_scale) <= 0.0001) {
        return 0.0;
    }
    let lock_factor =
        1.0 - clamp(endpoint_lock_px / max(render_length, 1.0), 0.0, 0.85) * 0.55;
    let drift = coherent_signed_noise_1d(
        uniforms.seed.x ^ uniforms.seed.y,
        edge_id,
        pass_index,
        f32(edge_id % 89u) * 0.17 + f32(pass_index) * 0.31 + f32(salt) * 0.013,
        977u + salt,
    );
    return drift * max(kind_wobble_px(kind), 0.05) * tangent_scale * lock_factor;
}

fn coherent_signed_noise_1d(seed: u32, edge_id: u32, pass_index: u32, position: f32, salt: u32) -> f32 {
    let base = floor(position);
    let frac = clamp(position - base, 0.0, 1.0);
    let blend = frac * frac * (3.0 - 2.0 * frac);
    let left = signed_noise(seed ^ edge_id ^ (pass_index * 1597u) ^ (salt + u32(max(base, 0.0))));
    let right =
        signed_noise(seed ^ edge_id ^ (pass_index * 1597u) ^ (salt + u32(max(base, 0.0)) + 1u));
    return left + (right - left) * blend;
}

fn sample_curve4(points: vec4<f32>, t: f32) -> f32 {
    let scaled = clamp(t, 0.0, 1.0) * 3.0;
    let index = min(u32(floor(scaled)), 2u);
    let local_t = clamp(scaled - f32(index), 0.0, 1.0);
    let a = points[index];
    let b = points[index + 1u];
    return a + (b - a) * local_t;
}


fn importance_from_depth(kind: u32, depth01: f32) -> f32 {
    let depth_factor = clamp(1.18 - depth01 * 0.38, 0.72, 1.18);
    if (kind == KIND_SILHOUETTE) {
        return depth_factor * 1.08;
    }
    if (kind == KIND_BOUNDARY) {
        return depth_factor * 0.96;
    }
    if (kind == KIND_CONTACT) {
        return depth_factor * 0.92;
    }
    return depth_factor * 0.88;
}

fn distance_width_multiplier(importance: f32) -> f32 {
    let pressure_boost = 1.0 + uniforms.params7.w * (importance - 1.0);
    return clamp((1.0 - uniforms.params7.z * (1.0 - importance)) * pressure_boost, 0.62, 1.28);
}

fn depth_alpha_multiplier(importance: f32) -> f32 {
    let near = pow(clamp(importance, 0.0, 1.35), 0.8);
    return clamp(1.0 + uniforms.params11.z * (near - 0.5), 0.35, 1.25);
}

fn pressure_multiplier(t: f32) -> f32 {
    let shaped = sample_curve4(uniforms.params8, t);
    return shaped * (0.92 + uniforms.params11.y * 0.12);
}

fn alpha_pressure_multiplier(t: f32) -> f32 {
    return clamp(sample_curve4(uniforms.params9, t), 0.0, 1.5);
}

fn taper_multiplier(t: f32) -> f32 {
    let endpoint_weight = clamp(min(t, 1.0 - t) * 2.0, 0.0, 1.0);
    return 1.0 - clamp(uniforms.params5.w, 0.0, 1.0) * (1.0 - max(endpoint_weight, 0.35));
}

fn pass_offset(edge_id: u32, pass_index: u32) -> f32 {
    if (uniforms.params3.w <= 0.0 && uniforms.params7.x <= 0.0) {
        return 0.0;
    }
    let seed = uniforms.seed.x ^ uniforms.seed.y;
    let base = coherent_signed_noise_1d(
        seed,
        edge_id,
        pass_index,
        f32(edge_id % 97u) * 0.21 + f32(pass_index),
        631u,
    );
    let is_search = pass_index >= u32(uniforms.params5.x);
    let multiplier = select(0.7, 1.25, is_search);
    return base * (uniforms.params3.w + uniforms.params7.x * 0.18) * multiplier;
}

fn primary_pass_width_multiplier(primary_passes: u32, pass_index: u32) -> f32 {
    if (primary_passes >= 3u) {
        return 0.9;
    }
    if (primary_passes == 2u) {
        return select(1.6, 0.85, pass_index > 0u);
    }
    return 0.694;
}

fn primary_pass_alpha_multiplier(primary_passes: u32, pass_index: u32) -> f32 {
    if (primary_passes >= 3u) {
        return 0.18;
    }
    if (primary_passes == 2u) {
        return select(0.28, 0.75, pass_index > 0u);
    }
    return 0.92;
}

fn pass_wobble_multiplier(pass_index: u32) -> f32 {
    let is_search = pass_index >= u32(uniforms.params5.x);
    return select(1.0, 1.18, is_search);
}

fn should_enable_search_passes(
    kind: u32,
    canonical_owner: u32,
    edge_index: u32,
    current_repr_score: f32,
    owner_score: f32,
    neighbor_repr_score: f32,
    chain_quality: f32,
    connected_start: bool,
    connected_end: bool,
    line_length: f32,
    render_length: f32,
    chained_span: f32,
    viability: f32,
) -> bool {
    if (!npr_gpu_search_enabled()) {
        return false;
    }
    if (kind == KIND_SILHOUETTE || kind == KIND_CONTACT) {
        return false;
    }

    let owner_ok = canonical_owner == edge_index;
    let long_enough = render_length >= max(line_length * 1.18, line_length + 6.0);
    let span_enough = chained_span >= max(line_length * 1.2, line_length + 6.0);
    let stable_chain = connected_start && connected_end && chain_quality >= 0.72;
    let boundary_short = kind == KIND_BOUNDARY && render_length < 24.0;
    if (!(owner_ok && stable_chain && long_enough && span_enough) || boundary_short) {
        return false;
    }

    let owner_ratio = owner_score / max(current_repr_score, 0.001);
    let exploratory_tool = uniforms.params7.y >= 0.58 || uniforms.params11.y <= 0.74;
    let strict_tool = uniforms.params11.y >= 0.9 && uniforms.params7.y <= 0.3;

    if (strict_tool && (owner_ratio < 1.02 || viability < 0.2)) {
        return false;
    }

    if (!exploratory_tool && (owner_ratio < 1.0 || neighbor_repr_score > current_repr_score + 0.35)) {
        return false;
    }

    if (viability < -0.05 && owner_ratio < 1.08) {
        return false;
    }

    return true;
}

fn pass_width(kind: u32, pass_index: u32, importance: f32, t: f32) -> f32 {
    let base = uniforms.params1.x * kind_width_multiplier(kind) * uniforms.params6.x;
    let is_search = pass_index >= u32(uniforms.params5.x);
    if (is_search) {
        return max(
            base
                * 0.78
                * pressure_multiplier(t)
                * taper_multiplier(t)
                * distance_width_multiplier(importance),
            0.25,
        );
    }
    return max(
        base
            * primary_pass_width_multiplier(u32(uniforms.params5.x), pass_index)
            * pressure_multiplier(t)
            * taper_multiplier(t)
            * distance_width_multiplier(importance),
        0.25,
    );
}

fn pass_alpha(kind: u32, pass_index: u32, importance: f32, t: f32) -> f32 {
    let base = uniforms.ink_color.w * kind_alpha_multiplier(kind) * uniforms.params6.y;
    let is_search = pass_index >= u32(uniforms.params5.x);
    if (is_search) {
        return clamp(
            base
                * uniforms.params5.z
                * npr_gpu_search_alpha_multiplier()
                * uniforms.params6.y
                * alpha_pressure_multiplier(t)
                * depth_alpha_multiplier(importance),
            0.0,
            1.0,
        );
    }
    return clamp(
        base
            * primary_pass_alpha_multiplier(u32(uniforms.params5.x), pass_index)
            * alpha_pressure_multiplier(t)
            * depth_alpha_multiplier(importance),
        0.0,
        1.0,
    );
}

fn pass_overshoot(kind: u32, pass_index: u32) -> f32 {
    let base = select(
        uniforms.params1.y,
        uniforms.params4.w,
        kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE || kind == KIND_CONTACT,
    );
    let is_search = pass_index >= u32(uniforms.params5.x);
    let clamped = min(base, 0.5);
    return select(clamped, min(max(clamped, uniforms.params11.w), 0.15), is_search);
}

fn endpoint_connection_score(
    next_edge_index: u32,
    kind: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
) -> f32 {
    if (next_edge_index == 0xffffffffu || next_edge_index >= u32(arrayLength(&visible_segments))) {
        return 0.0;
    }
    let next = visible_segments[next_edge_index];
    if (next.start.w <= 0.5 || next.end.w <= 0.5 || next.kind_edge.x != kind) {
        return 0.0;
    }
    let match_is_start = continuation_match_is_start(anchor_point, next_edge_index);
    let gap = min(distance(anchor_point, next.start.xy), distance(anchor_point, next.end.xy));
    let endpoint_snap = max(uniforms.params12.w, 0.5);
    if (gap > endpoint_snap * 5.5) {
        return 0.0;
    }
    let continuation = continuation_endpoint(anchor_point, next_edge_index);
    let same_bin = same_anchor_bin(anchor_point, continuation);
    let bin_gap = anchor_bin_gap(anchor_point, continuation);
    if (!same_bin && gap > endpoint_snap * 1.6) {
        return 0.0;
    }
    let delta = continuation - anchor_point;
    let continuation_length = max(length(delta), 0.0001);
    let continuation_direction = delta / continuation_length;
    let alignment = dot(current_direction, continuation_direction);
    if (alignment <= 0.35) {
        return 0.0;
    }
    let matched_depth = select(next.end.z, next.start.z, match_is_start);
    let depth_gap = abs(anchor_depth - matched_depth);
    let next_length = distance(next.start.xy, next.end.xy);
    let length_mismatch = abs(current_length - next_length) / max(max(current_length, next_length), 1.0);
    let tangent_mismatch = 1.0 - clamp(alignment, -1.0, 1.0);
    let junction_penalty = select(0.0, 0.85, endpoint_degree(edges[next_edge_index], match_is_start) != 1u);
    let cost =
        (gap / endpoint_snap) * select(1.35, 0.6, same_bin)
        + bin_gap * 0.8
        + tangent_mismatch * 14.0
        + depth_gap * 2.1
        + length_mismatch * 3.2
        + junction_penalty;
    return 1.0 / (1.0 + cost * 0.18);
}

fn best_endpoint_candidate(
    primary_edge_index: u32,
    alternative_edge_index: u32,
    kind: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
) -> vec2<f32> {
    let primary_score =
        endpoint_connection_score(
            primary_edge_index,
            kind,
            anchor_point,
            anchor_depth,
            current_direction,
            current_length,
        );
    let alternative_score =
        endpoint_connection_score(
            alternative_edge_index,
            kind,
            anchor_point,
            anchor_depth,
            current_direction,
            current_length,
        );
    let best_score = max(primary_score, alternative_score);
    let score_gap = abs(primary_score - alternative_score);
    if (best_score < 0.58) {
        return vec2<f32>(0.0, 0.0);
    }
    if (primary_score > 0.0 && alternative_score > 0.0 && score_gap < 0.05 && best_score < 0.82) {
        return vec2<f32>(0.0, 0.0);
    }
    return vec2<f32>(primary_score, alternative_score);
}

fn best_edge_candidate_index(
    primary_edge_index: u32,
    alternative_edge_index: u32,
    scores: vec2<f32>,
) -> u32 {
    if (scores.x <= 0.0 && scores.y <= 0.0) {
        return 0xffffffffu;
    }
    return select(primary_edge_index, alternative_edge_index, scores.y > scores.x);
}

fn continuation_endpoint(anchor_point: vec2<f32>, next_edge_index: u32) -> vec2<f32> {
    let next = visible_segments[next_edge_index];
    let distance_start = distance(anchor_point, next.start.xy);
    let distance_end = distance(anchor_point, next.end.xy);
    return select(next.start.xy, next.end.xy, distance_start < distance_end);
}

fn continuation_match_is_start(anchor_point: vec2<f32>, next_edge_index: u32) -> bool {
    let next = visible_segments[next_edge_index];
    let distance_start = distance(anchor_point, next.start.xy);
    let distance_end = distance(anchor_point, next.end.xy);
    return distance_start < distance_end;
}

fn continuation_follow_edge(next_edge_index: u32, matched_start: bool) -> u32 {
    let next_edge = edges[next_edge_index];
    return select(next_edge.next_a, next_edge.next_b, matched_start);
}

fn endpoint_degree(edge: GpuNprEdge3d, matched_start: bool) -> u32 {
    return select(edge.degree_a, edge.degree_b, matched_start);
}

fn quantized_anchor_bin(point: vec2<f32>) -> vec2<i32> {
    let quant = max(uniforms.params12.w, 0.5);
    return vec2<i32>(round(point / quant));
}

fn same_anchor_bin(a: vec2<f32>, b: vec2<f32>) -> bool {
    let qa = quantized_anchor_bin(a);
    let qb = quantized_anchor_bin(b);
    return qa.x == qb.x && qa.y == qb.y;
}

fn anchor_bin_gap(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let qa = quantized_anchor_bin(a);
    let qb = quantized_anchor_bin(b);
    return f32(abs(qa.x - qb.x) + abs(qa.y - qb.y));
}

fn continuation_out_direction(anchor_point: vec2<f32>, next_edge_index: u32) -> vec2<f32> {
    let next = visible_segments[next_edge_index];
    let match_is_start = continuation_match_is_start(anchor_point, next_edge_index);
    let matched = select(next.end.xy, next.start.xy, match_is_start);
    let far = select(next.start.xy, next.end.xy, match_is_start);
    let delta = far - matched;
    let length_px = length(delta);
    if (length_px <= 0.0001) {
        return vec2<f32>(0.0, 0.0);
    }
    return delta / length_px;
}

fn chained_endpoint(current: vec2<f32>, continuation: vec2<f32>, score: f32) -> vec2<f32> {
    let delta = continuation - current;
    let length_px = max(length(delta), 0.0001);
    let chain_strength = min(0.28, 24.0 / length_px) * clamp(score, 0.0, 1.0);
    return current + delta * chain_strength;
}

fn terminal_endpoint_walk(
    current: vec2<f32>,
    current_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
    next_edge_index: u32,
    kind: u32,
) -> vec2<f32> {
    if (npr_gpu_max_terminal_walk_edges() <= 1u) {
        if (endpoint_connection_score(next_edge_index, kind, current, current_depth, current_direction, current_length) > 0.0) {
            return continuation_endpoint(current, next_edge_index);
        }
        return current;
    }
    let score0 =
        endpoint_connection_score(next_edge_index, kind, current, current_depth, current_direction, current_length);
    if (score0 <= 0.0) {
        return current;
    }

    let matched_start0 = continuation_match_is_start(current, next_edge_index);
    let degree0 = endpoint_degree(edges[next_edge_index], matched_start0);
    let continuation0 = continuation_endpoint(current, next_edge_index);
    var terminal = continuation0;

    if (score0 < 0.72 || degree0 != 1u) {
        return terminal;
    }

    let next_direction = normalize(continuation0 - current);
    let follow_primary = continuation_follow_edge(next_edge_index, matched_start0);
    let followed_edge = edges[next_edge_index];
    let follow_alternative = select(followed_edge.alt_next_a, followed_edge.alt_next_b, matched_start0);
    let continuation0_depth = select(
        visible_segments[next_edge_index].end.z,
        visible_segments[next_edge_index].start.z,
        matched_start0,
    );
    let continuation0_length =
        distance(visible_segments[next_edge_index].start.xy, visible_segments[next_edge_index].end.xy);
    let hop1_scores = best_endpoint_candidate(
        follow_primary,
        follow_alternative,
        kind,
        continuation0,
        continuation0_depth,
        next_direction,
        continuation0_length,
    );
    let hop1_edge_index = best_edge_candidate_index(follow_primary, follow_alternative, hop1_scores);
    let score1 = endpoint_connection_score(
        hop1_edge_index,
        kind,
        continuation0,
        continuation0_depth,
        next_direction,
        continuation0_length,
    );
    if (score1 <= 0.0) {
        return terminal;
    }

    let matched_start1 = continuation_match_is_start(continuation0, hop1_edge_index);
    let degree1 = endpoint_degree(edges[hop1_edge_index], matched_start1);
    let continuation1 = continuation_endpoint(continuation0, hop1_edge_index);
    terminal = continuation1;

    if (score1 < 0.72 || degree1 != 1u) {
        return terminal;
    }
    if (npr_gpu_max_terminal_walk_edges() <= 2u) {
        return terminal;
    }

    let next_direction1 = normalize(continuation1 - continuation0);
    let follow_primary1 = continuation_follow_edge(hop1_edge_index, matched_start1);
    let followed_edge1 = edges[hop1_edge_index];
    let follow_alternative1 = select(followed_edge1.alt_next_a, followed_edge1.alt_next_b, matched_start1);
    let continuation1_depth = select(
        visible_segments[hop1_edge_index].end.z,
        visible_segments[hop1_edge_index].start.z,
        matched_start1,
    );
    let continuation1_length =
        distance(visible_segments[hop1_edge_index].start.xy, visible_segments[hop1_edge_index].end.xy);
    let hop2_scores = best_endpoint_candidate(
        follow_primary1,
        follow_alternative1,
        kind,
        continuation1,
        continuation1_depth,
        next_direction1,
        continuation1_length,
    );
    let hop2_edge_index =
        best_edge_candidate_index(follow_primary1, follow_alternative1, hop2_scores);
    let score2 = endpoint_connection_score(
        hop2_edge_index,
        kind,
        continuation1,
        continuation1_depth,
        next_direction1,
        continuation1_length,
    );
    if (score2 <= 0.0) {
        return terminal;
    }

    let matched_start2 = continuation_match_is_start(continuation1, hop2_edge_index);
    let degree2 = endpoint_degree(edges[hop2_edge_index], matched_start2);
    let continuation2 = continuation_endpoint(continuation1, hop2_edge_index);
    terminal = continuation2;

    if (degree2 != 1u) {
        return terminal;
    }

    return terminal;
}

fn chained_endpoint_walk(
    current: vec2<f32>,
    current_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
    next_edge_index: u32,
    kind: u32,
) -> vec2<f32> {
    if (npr_gpu_max_chained_walk_edges() <= 1u) {
        let score = endpoint_connection_score(next_edge_index, kind, current, current_depth, current_direction, current_length);
        if (score > 0.0) {
            return chained_endpoint(current, continuation_endpoint(current, next_edge_index), score);
        }
        return current;
    }
    let score0 =
        endpoint_connection_score(next_edge_index, kind, current, current_depth, current_direction, current_length);
    if (score0 <= 0.0) {
        return current;
    }
    let matched_start0 = continuation_match_is_start(current, next_edge_index);
    let degree0 = endpoint_degree(edges[next_edge_index], matched_start0);
    let continuation0 = continuation_endpoint(current, next_edge_index);
    var chained = chained_endpoint(current, continuation0, score0);

    if (score0 < 0.72 || degree0 != 1u) {
        return chained;
    }

    let next_direction = normalize(continuation0 - current);
    let follow_primary = continuation_follow_edge(next_edge_index, matched_start0);
    let followed_edge = edges[next_edge_index];
    let follow_alternative = select(followed_edge.alt_next_a, followed_edge.alt_next_b, matched_start0);
    let continuation0_depth = select(
        visible_segments[next_edge_index].end.z,
        visible_segments[next_edge_index].start.z,
        matched_start0,
    );
    let continuation0_length =
        distance(visible_segments[next_edge_index].start.xy, visible_segments[next_edge_index].end.xy);
    let hop1_scores = best_endpoint_candidate(
        follow_primary,
        follow_alternative,
        kind,
        continuation0,
        continuation0_depth,
        next_direction,
        continuation0_length,
    );
    let hop1_edge_index = best_edge_candidate_index(follow_primary, follow_alternative, hop1_scores);
    let score1 = endpoint_connection_score(
        hop1_edge_index,
        kind,
        continuation0,
        continuation0_depth,
        next_direction,
        continuation0_length,
    );
    if (score1 <= 0.0) {
        return chained;
    }

    let matched_start1 = continuation_match_is_start(continuation0, hop1_edge_index);
    let degree1 = endpoint_degree(edges[hop1_edge_index], matched_start1);
    if (degree1 != 1u) {
        return chained;
    }

    let continuation1 = continuation_endpoint(continuation0, hop1_edge_index);
    let combined_score = score0 * score1;
    chained = chained_endpoint(chained, continuation1, combined_score * 0.55);

    if (score1 < 0.72) {
        return chained;
    }
    if (npr_gpu_max_chained_walk_edges() <= 2u) {
        return chained;
    }

    let next_direction1 = normalize(continuation1 - continuation0);
    let follow_primary1 = continuation_follow_edge(hop1_edge_index, matched_start1);
    let followed_edge1 = edges[hop1_edge_index];
    let follow_alternative1 = select(followed_edge1.alt_next_a, followed_edge1.alt_next_b, matched_start1);
    let continuation1_depth = select(
        visible_segments[hop1_edge_index].end.z,
        visible_segments[hop1_edge_index].start.z,
        matched_start1,
    );
    let continuation1_length =
        distance(visible_segments[hop1_edge_index].start.xy, visible_segments[hop1_edge_index].end.xy);
    let hop2_scores = best_endpoint_candidate(
        follow_primary1,
        follow_alternative1,
        kind,
        continuation1,
        continuation1_depth,
        next_direction1,
        continuation1_length,
    );
    let hop2_edge_index =
        best_edge_candidate_index(follow_primary1, follow_alternative1, hop2_scores);
    let score2 = endpoint_connection_score(
        hop2_edge_index,
        kind,
        continuation1,
        continuation1_depth,
        next_direction1,
        continuation1_length,
    );
    if (score2 <= 0.0) {
        return chained;
    }

    let matched_start2 = continuation_match_is_start(continuation1, hop2_edge_index);
    let degree2 = endpoint_degree(edges[hop2_edge_index], matched_start2);
    if (degree2 != 1u) {
        return chained;
    }

    let continuation2 = continuation_endpoint(continuation1, hop2_edge_index);
    let combined_score2 = combined_score * score2;
    chained = chained_endpoint(chained, continuation2, combined_score2 * 0.38);
    return chained;
}

fn best_follow_edge_index(
    primary_edge_index: u32,
    alternative_edge_index: u32,
    kind: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
) -> u32 {
    let scores = best_endpoint_candidate(
        primary_edge_index,
        alternative_edge_index,
        kind,
        anchor_point,
        anchor_depth,
        current_direction,
        current_length,
    );
    return best_edge_candidate_index(primary_edge_index, alternative_edge_index, scores);
}

fn visible_segment_length(edge_index: u32) -> f32 {
    if (edge_index == 0xffffffffu || edge_index >= u32(arrayLength(&visible_segments))) {
        return 0.0;
    }
    let visible = visible_segments[edge_index];
    return distance(visible.start.xy, visible.end.xy);
}

fn representative_edge_score(
    edge_index: u32,
    connection_score: f32,
    current_length: f32,
) -> f32 {
    let edge_length = visible_segment_length(edge_index);
    let length_score = max(edge_length, current_length * 0.6);
    let edge = edges[edge_index];
    let degree_penalty =
        f32(max(edge.degree_a, 1u) - 1u + max(edge.degree_b, 1u) - 1u) * 1.75;
    return length_score + connection_score * 18.0 - degree_penalty;
}

fn pick_better_owner(left: ChainOwnerPick, right: ChainOwnerPick) -> ChainOwnerPick {
    if (right.score > left.score + 0.001) {
        return right;
    }
    if (abs(right.score - left.score) <= 0.001 && right.owner < left.owner) {
        return right;
    }
    return left;
}

fn local_chain_centrality(connected_start: bool, connected_end: bool) -> f32 {
    if (connected_start && connected_end) {
        return 1.0;
    }
    if (connected_start || connected_end) {
        return 0.55;
    }
    return 0.0;
}

fn neighbor_representative_score(
    current_length: f32,
    start_next_edge: u32,
    end_next_edge: u32,
    connected_start_score: f32,
    connected_end_score: f32,
) -> f32 {
    let start_score = select(
        0.0,
        representative_edge_score(start_next_edge, connected_start_score, current_length),
        start_next_edge != 0xffffffffu,
    );
    let end_score = select(
        0.0,
        representative_edge_score(end_next_edge, connected_end_score, current_length),
        end_next_edge != 0xffffffffu,
    );
    return max(start_score, end_score);
}

fn span_takeover_score(
    base_length: f32,
    candidate_length: f32,
    connection_score: f32,
) -> f32 {
    let gained = max(candidate_length - base_length, 0.0);
    return gained + connection_score * 12.0;
}

fn emit_viability_score(
    current_repr_score: f32,
    neighbor_repr_score: f32,
    centrality: f32,
    line_length: f32,
    shorter_neighbor: f32,
) -> f32 {
    let length_ratio = line_length / max(shorter_neighbor, 1.0);
    return current_repr_score
        - neighbor_repr_score
        + centrality * 6.0
        + clamp(length_ratio, 0.0, 1.0) * 2.0;
}

fn debug_color_for_kind(kind: u32) -> vec4<f32> {
    if (kind == KIND_BOUNDARY) {
        return vec4<f32>(0.25, 0.85, 1.0, 1.0);
    }
    if (kind == KIND_SILHOUETTE) {
        return vec4<f32>(1.0, 0.35, 0.2, 1.0);
    }
    if (kind == KIND_CREASE) {
        return vec4<f32>(0.95, 0.85, 0.2, 1.0);
    }
    if (kind == KIND_SEAM) {
        return vec4<f32>(0.75, 0.35, 1.0, 1.0);
    }
    if (kind == KIND_FEATURE) {
        return vec4<f32>(0.3, 1.0, 0.45, 1.0);
    }
    if (kind == KIND_CONTACT) {
        return vec4<f32>(1.0, 0.65, 0.15, 1.0);
    }
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

fn debug_overlay_mode() -> u32 {
    return u32(uniforms.params13.x);
}

fn npr_gpu_max_render_length_px() -> f32 {
    return max(uniforms.params14.x, 8.0);
}

fn npr_gpu_max_segment_length_px() -> f32 {
    return max(uniforms.params14.y, 4.0);
}

fn npr_gpu_max_terminal_walk_edges() -> u32 {
    return u32(max(uniforms.params14.z, 0.0));
}

fn npr_gpu_max_chained_walk_edges() -> u32 {
    return u32(max(uniforms.params14.w, 0.0));
}

fn npr_gpu_search_enabled() -> bool {
    return uniforms.params15.y > 0.5;
}

fn npr_gpu_search_max_render_length_px() -> f32 {
    return max(uniforms.params15.z, 4.0);
}

fn npr_gpu_search_alpha_multiplier() -> f32 {
    return clamp(uniforms.params15.w, 0.0, 1.0);
}

fn debug_color_for_overlay(
    kind: u32,
    pass_index: u32,
    canonical_owner: u32,
    edge_index: u32,
    connected_start: bool,
    connected_end: bool,
    current_repr_score: f32,
    neighbor_repr_score: f32,
    viability: f32,
    width: f32,
    alpha: f32,
) -> vec4<f32> {
    let mode = debug_overlay_mode();
    if (mode == 1u) {
        return debug_color_for_kind(kind);
    }
    if (mode == 2u) {
        let owner = canonical_owner == edge_index;
        let rgb = select(
            vec3<f32>(0.2, 0.55, 1.0),
            vec3<f32>(1.0, 0.45, 0.15),
            owner,
        );
        let connected = f32(u32(connected_start) + u32(connected_end)) * 0.25 + 0.5;
        return vec4<f32>(rgb * connected, 1.0);
    }
    if (mode == 3u) {
        let delta = clamp((current_repr_score - neighbor_repr_score) * 0.08 + 0.5, 0.0, 1.0);
        return vec4<f32>(1.0 - delta, delta, 0.2 + max(viability, 0.0) * 0.04, 1.0);
    }
    if (mode == 4u) {
        return vec4<f32>(clamp(width / 6.0, 0.0, 1.0), clamp(alpha, 0.0, 1.0), 0.25, 1.0);
    }
    let search = pass_index >= u32(uniforms.params5.x);
    let base = uniforms.ink_color.rgb;
    let alpha_out = select(alpha, min(alpha * 1.15, 1.0), search);
    return vec4<f32>(base, alpha_out);
}

fn debug_overlay_segment_width(mode: u32, width: f32, end_width: f32) -> vec2<f32> {
    if (mode == 1u || mode == 2u) {
        return vec2<f32>(2.25, 2.25);
    }
    if (mode == 3u) {
        return vec2<f32>(4.0, 4.0);
    }
    return vec2<f32>(width, end_width);
}

fn debug_overlay_offset(mode: u32, offset_px: f32) -> f32 {
    if (mode == 0u) {
        return offset_px;
    }
    if (mode == 4u) {
        return offset_px * 0.2;
    }
    return 0.0;
}

fn debug_overlay_overshoot(mode: u32, overshoot_px: f32) -> f32 {
    if (mode == 0u || mode == 4u) {
        return overshoot_px;
    }
    return 0.0;
}

fn chain_owner_from_endpoint(
    seed_owner: u32,
    seed_score: f32,
    initial_edge_index: u32,
    kind: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
) -> ChainOwnerPick {
    if (initial_edge_index == 0xffffffffu) {
        return ChainOwnerPick(seed_owner, seed_score);
    }

    var best = ChainOwnerPick(seed_owner, seed_score);
    var current_anchor = anchor_point;
    var current_depth_local = anchor_depth;
    var current_direction_local = current_direction;
    var current_length_local = current_length;
    var edge_index = initial_edge_index;
    var accumulated_span = 0.0;

    for (var hop: u32 = 0u; hop < npr_gpu_max_chained_walk_edges(); hop = hop + 1u) {
        if (edge_index == 0xffffffffu || edge_index >= u32(arrayLength(&visible_segments))) {
            break;
        }

        let score = endpoint_connection_score(
            edge_index,
            kind,
            current_anchor,
            current_depth_local,
            current_direction_local,
            current_length_local,
        );
        if (score < 0.72) {
            break;
        }

        accumulated_span = accumulated_span + current_length_local * score;

        best = pick_better_owner(
            best,
            ChainOwnerPick(
                edge_index,
                representative_edge_score(edge_index, score, current_length_local)
                    + accumulated_span * 0.18,
            ),
        );

        let matched_start = continuation_match_is_start(current_anchor, edge_index);
        let degree = endpoint_degree(edges[edge_index], matched_start);
        if (degree != 1u) {
            break;
        }

        let continuation = continuation_endpoint(current_anchor, edge_index);
        let direction_delta = continuation - current_anchor;
        let direction_length = length(direction_delta);
        if (direction_length <= 0.0001) {
            break;
        }

        current_direction_local = direction_delta / direction_length;
        current_anchor = continuation;
        current_depth_local = select(
            visible_segments[edge_index].end.z,
            visible_segments[edge_index].start.z,
            matched_start,
        );
        current_length_local =
            distance(visible_segments[edge_index].start.xy, visible_segments[edge_index].end.xy);

        let followed_edge = edges[edge_index];
        let next_primary = continuation_follow_edge(edge_index, matched_start);
        let next_alternative = select(followed_edge.alt_next_a, followed_edge.alt_next_b, matched_start);
        edge_index = best_follow_edge_index(
            next_primary,
            next_alternative,
            kind,
            current_anchor,
            current_depth_local,
            current_direction_local,
            current_length_local,
        );
    }

    return best;
}

fn should_emit_segment_instance(
    edge_index: u32,
    edge: GpuNprEdge3d,
    screen_a: vec2<f32>,
    screen_b: vec2<f32>,
    line_length: f32,
    connected_start_score: f32,
    connected_end_score: f32,
    start_next_edge: u32,
    end_next_edge: u32,
) -> bool {
    if (edge.degree_a > 1u || edge.degree_b > 1u) {
        let strong_chain = connected_start_score >= 0.72 && connected_end_score >= 0.72;
        if (!strong_chain || line_length > 18.0) {
            return true;
        }
    }
    if (line_length > 22.0) {
        return true;
    }
    if (connected_start_score < 0.62 || connected_end_score < 0.62) {
        return true;
    }
    let left_is_better = start_next_edge < edge_index;
    let right_is_better = end_next_edge < edge_index;
    if (!(left_is_better || right_is_better)) {
        return true;
    }
    let start_gap = abs(line_length - distance(visible_segments[start_next_edge].start.xy, visible_segments[start_next_edge].end.xy));
    let end_gap = abs(line_length - distance(visible_segments[end_next_edge].start.xy, visible_segments[end_next_edge].end.xy));
    if (!(start_gap < 10.0 && end_gap < 10.0)) {
        return true;
    }

    let endpoint_snap = max(uniforms.params12.w, 0.5);
    if (line_length > endpoint_snap * 6.0) {
        return true;
    }

    let start_anchor_ok = same_anchor_bin(screen_a, continuation_endpoint(screen_a, start_next_edge));
    let end_anchor_ok = same_anchor_bin(screen_b, continuation_endpoint(screen_b, end_next_edge));
    if (!(start_anchor_ok && end_anchor_ok)) {
        return true;
    }

    let start_neighbor_len =
        distance(visible_segments[start_next_edge].start.xy, visible_segments[start_next_edge].end.xy);
    let end_neighbor_len =
        distance(visible_segments[end_next_edge].start.xy, visible_segments[end_next_edge].end.xy);
    let shorter_neighbor = min(start_neighbor_len, end_neighbor_len);
    if (line_length > shorter_neighbor * 0.65) {
        return true;
    }

    let current_delta = screen_b - screen_a;
    let current_dir = normalize(current_delta);
    let start_dir = continuation_out_direction(screen_a, start_next_edge);
    let end_dir = continuation_out_direction(screen_b, end_next_edge);
    let start_alignment = dot(-current_dir, start_dir);
    let end_alignment = dot(current_dir, end_dir);
    if (start_alignment < 0.7 || end_alignment < 0.7) {
        return true;
    }

    let centrality = local_chain_centrality(true, true);
    let current_repr_score =
        representative_edge_score(
            edge_index,
            max(connected_start_score, connected_end_score),
            line_length,
        );
    let neighbor_repr_score =
        neighbor_representative_score(
            line_length,
            start_next_edge,
            end_next_edge,
            connected_start_score,
            connected_end_score,
        );
    let viability =
        emit_viability_score(
            current_repr_score,
            neighbor_repr_score,
            centrality,
            line_length,
            shorter_neighbor,
        );
    if (
        centrality >= 1.0
        && line_length <= shorter_neighbor * 0.92
        && viability <= -1.5
    ) {
        return false;
    }

    return false;
}

fn canonical_chain_owner_index(
    edge_index: u32,
    edge: GpuNprEdge3d,
    kind: u32,
    screen_a: vec2<f32>,
    screen_b: vec2<f32>,
    line_length: f32,
    start_direction: vec2<f32>,
    end_direction: vec2<f32>,
    start_depth: f32,
    end_depth: f32,
    connected_start_score: f32,
    connected_end_score: f32,
    start_next_edge: u32,
    end_next_edge: u32,
) -> ChainOwnerPick {
    let base_score =
        representative_edge_score(
            edge_index,
            max(connected_start_score, connected_end_score),
            line_length,
        );
    var best = ChainOwnerPick(edge_index, base_score);
    let junction_heavy = edge.degree_a > 1u || edge.degree_b > 1u;
    let owner_follow_threshold = select(0.72, 0.78, junction_heavy);

    if (connected_start_score >= owner_follow_threshold && start_next_edge != 0xffffffffu) {
        best = pick_better_owner(
            best,
            chain_owner_from_endpoint(
            best.owner,
            base_score,
            start_next_edge,
            kind,
            screen_a,
            start_depth,
            start_direction,
            line_length,
        ));
    }

    if (connected_end_score >= owner_follow_threshold && end_next_edge != 0xffffffffu) {
        best = pick_better_owner(
            best,
            chain_owner_from_endpoint(
            best.owner,
            base_score,
            end_next_edge,
            kind,
            screen_b,
            end_depth,
            end_direction,
            line_length,
        ));
    }

    return best;
}

fn endpoint_taper(
    width: f32,
    alpha: f32,
    depth01: f32,
    pass_index: u32,
    connected_start: bool,
    connected_end: bool,
) -> vec4<f32> {
    let taper = clamp(uniforms.params5.w, 0.0, 1.0);
    let near_strength = 1.0 + (1.0 - depth01) * 0.18;
    let far_strength = 1.0 - taper * (0.28 + depth01 * 0.22);
    let search = pass_index >= u32(uniforms.params5.x);
    let tapered_start_width = width * select(near_strength, near_strength * 0.92, search);
    let tapered_end_width = max(width * select(far_strength, far_strength * 0.9, search), 0.5);
    let tapered_start_alpha = alpha * select(0.94 + (1.0 - depth01) * 0.06, 0.88, search);
    let tapered_end_alpha = alpha * max(1.0 - taper * 0.45, 0.35);
    let start_width = select(tapered_start_width, width, connected_start);
    let end_width = select(tapered_end_width, width, connected_end);
    let start_alpha = select(tapered_start_alpha, alpha, connected_start);
    let end_alpha = select(tapered_end_alpha, alpha, connected_end);
    return vec4<f32>(start_width, end_width, start_alpha, end_alpha);
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= u32(arrayLength(&edges))) {
        return;
    }

    let visible = visible_segments[edge_index];
    if (visible.start.w < 0.5 || visible.end.w < 0.5) {
        return;
    }
    let path_link = path_links[edge_index];
    if ((path_link.flags & PATH_FLAG_EMIT) == 0u) {
        return;
    }
    if (path_link.owner_edge != edge_index) {
        return;
    }

    let screen_a = visible.start.xy;
    let screen_b = visible.end.xy;
    let line_length = distance(screen_a, screen_b);
    if (line_length < uniforms.params0.w) {
        return;
    }

    let kind = visible.kind_edge.x;
    if (kind == KIND_NONE) {
        return;
    }
    let primary_pass_count = max(u32(uniforms.params5.x), 1u);
    let edge_id = visible.kind_edge.y;
    let edge = edges[edge_index];
    let line_depth = (visible.start.z + visible.end.z) * 0.5;
    let seed32 = uniforms.seed.x ^ uniforms.seed.y;
    let contour_line = kind == KIND_BOUNDARY || kind == KIND_SILHOUETTE;
    let structural_line = kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE;
    let primary_line = contour_line || structural_line;
    if (!primary_line) {
        if (!should_emit_segment_instance(
            edge_index,
            edge,
            screen_a,
            screen_b,
            line_length,
            0.0,
            0.0,
            0xffffffffu,
            0xffffffffu,
        )) {
            return;
        }
    }
    let start_next_edge = path_link.start_next;
    let end_next_edge = path_link.end_next;
    let connected_start =
        (path_link.flags & PATH_FLAG_CONNECTED_START) != 0u && start_next_edge != 0xffffffffu;
    let connected_end =
        (path_link.flags & PATH_FLAG_CONNECTED_END) != 0u && end_next_edge != 0xffffffffu;
    let start_depth = visible.start.z;
    let end_depth = visible.end.z;
    let start_direction = normalize(screen_a - screen_b);
    let end_direction = normalize(screen_b - screen_a);
    let connected_start_score = select(
        0.0,
        endpoint_connection_score(
            start_next_edge,
            kind,
            screen_a,
            start_depth,
            start_direction,
            line_length,
        ),
        connected_start,
    );
    let connected_end_score = select(
        0.0,
        endpoint_connection_score(
            end_next_edge,
            kind,
            screen_b,
            end_depth,
            end_direction,
            line_length,
        ),
        connected_end,
    );
    let current_repr_score =
        representative_edge_score(edge_index, max(connected_start_score, connected_end_score), line_length);
    let neighbor_repr_score =
        neighbor_representative_score(
            line_length,
            start_next_edge,
            end_next_edge,
            connected_start_score,
            connected_end_score,
        );
    let canonical_owner_pick = canonical_chain_owner_index(
        edge_index,
        edge,
        kind,
        screen_a,
        screen_b,
        line_length,
        start_direction,
        end_direction,
        start_depth,
        end_depth,
        connected_start_score,
        connected_end_score,
        start_next_edge,
        end_next_edge,
    );
    let canonical_owner = canonical_owner_pick.owner;
    if (canonical_owner != edge_index) {
        return;
    }
    let chain_quality = max(connected_start_score, connected_end_score);
    var render_start_mut = select(
        screen_a,
        terminal_endpoint_walk(
            screen_a,
            start_depth,
            start_direction,
            line_length,
            start_next_edge,
            kind,
        ),
        connected_start,
    );
    var render_end_mut = select(
        screen_b,
        terminal_endpoint_walk(
            screen_b,
            end_depth,
            end_direction,
            line_length,
            end_next_edge,
            kind,
        ),
        connected_end,
    );
    let chained_start_point = select(
        screen_a,
        chained_endpoint_walk(
            screen_a,
            start_depth,
            start_direction,
            line_length,
            start_next_edge,
            kind,
        ),
        connected_start,
    );
    let chained_end_point = select(
        screen_b,
        chained_endpoint_walk(
            screen_b,
            end_depth,
            end_direction,
            line_length,
            end_next_edge,
            kind,
        ),
        connected_end,
    );
    render_start_mut = mix(render_start_mut, chained_start_point, 0.18);
    render_end_mut = mix(render_end_mut, chained_end_point, 0.18);
    let chained_span = distance(render_start_mut, render_end_mut);
    let viability =
        emit_viability_score(
            current_repr_score,
            neighbor_repr_score,
            local_chain_centrality(connected_start, connected_end),
            line_length,
            chained_span,
        );
    var render_length = distance(render_start_mut, render_end_mut);
    let max_render_length = npr_gpu_max_render_length_px();
    if (render_length > max_render_length) {
        let trunc_direction = normalize(render_end_mut - render_start_mut);
        render_end_mut = render_start_mut + trunc_direction * max_render_length;
        render_length = max_render_length;
    }
    if (render_length < uniforms.params0.w) {
        return;
    }
    let viewport_diagonal = length(uniforms.viewport_half.xy) * 2.0;
    if (render_length > min(viewport_diagonal * 0.82, max(line_length * 3.2, line_length + 96.0))) {
        return;
    }
    let base_direction = normalize(screen_b - screen_a);
    let render_direction = normalize(render_end_mut - render_start_mut);
    if (dot(base_direction, render_direction) <= 0.2) {
        return;
    }
    if (kind == KIND_CONTACT && (!connected_start || !connected_end || chain_quality < 0.72)) {
        return;
    }
    if (
        render_length > max(line_length * 2.8, line_length + 40.0)
        && (!connected_start || !connected_end || chain_quality < 0.82)
    ) {
        return;
    }
    let importance = importance_from_depth(kind, line_depth);
    let search_pass_count = u32(uniforms.params5.y);
    let total_pass_count = primary_pass_count + search_pass_count;

    for (var pass_index: u32 = 0u; pass_index < total_pass_count; pass_index = pass_index + 1u) {
        var pass_render_start = render_start_mut;
        var pass_render_end = render_end_mut;
        var pass_render_length = render_length;
        if (pass_index >= primary_pass_count) {
            let search_max_length = npr_gpu_search_max_render_length_px();
            if (pass_render_length > search_max_length) {
                let trunc_direction = normalize(pass_render_end - pass_render_start);
                pass_render_end = pass_render_start + trunc_direction * search_max_length;
                pass_render_length = search_max_length;
            }
        }
        if (should_drop_segment_instance(kind, pass_index, edge_id, pass_render_length)) {
            continue;
        }
        let segment_count = select(1u, 2u, pass_render_length >= npr_gpu_max_segment_length_px());
        let out_index = atomicAdd(&indirect_args[1], segment_count);
        if (out_index + segment_count > u32(arrayLength(&stroke_segments))) {
            _ = atomicSub(&indirect_args[1], segment_count);
            return;
        }
        let width_start = pass_width(kind, pass_index, importance, 0.0);
        let width_mid = pass_width(kind, pass_index, importance, 0.5);
        let width_end = pass_width(kind, pass_index, importance, 1.0);
        let alpha_start = pass_alpha(kind, pass_index, importance, 0.0);
        let alpha_mid = pass_alpha(kind, pass_index, importance, 0.5);
        let alpha_end = pass_alpha(kind, pass_index, importance, 1.0);
        let width_noise_start =
            coherent_signed_noise_1d(seed32, edge_id, pass_index, 7.0, 503u)
            * uniforms.params10.x
            * uniforms.params6.z;
        let width_noise_mid =
            coherent_signed_noise_1d(seed32, edge_id, pass_index, 9.0, 503u)
            * uniforms.params10.x
            * uniforms.params6.z;
        let width_noise_end =
            coherent_signed_noise_1d(seed32, edge_id, pass_index, 11.0, 503u)
            * uniforms.params10.x
            * uniforms.params6.z;
        let tapering = endpoint_taper(
            max(width_start + width_noise_start, 0.25),
            alpha_start,
            line_depth,
            pass_index,
            connected_start,
            connected_end,
        );
        let pass_wobble = pass_wobble_multiplier(pass_index);
        let wobble = coherent_signed_noise_1d(
            seed32,
            edge_id,
            pass_index,
            f32(edge_index % 101u) * uniforms.params10.y + 3.0,
            919u,
        ) * kind_wobble_px(kind) * uniforms.params7.y * pass_wobble;
        let micro = coherent_signed_noise_1d(
            seed32,
            edge_id,
            pass_index,
            f32(edge_index % 71u) * uniforms.params10.w + 13.0,
            991u,
        ) * uniforms.params10.z * pass_wobble;
        let debug_color = debug_color_for_overlay(
            kind,
            pass_index,
            canonical_owner,
            edge_index,
            connected_start,
            connected_end,
            current_repr_score,
            neighbor_repr_score,
            viability,
            tapering.x,
            tapering.z,
        );
        let debug_mode = debug_overlay_mode();
        let debug_widths = debug_overlay_segment_width(
            debug_mode,
            tapering.x,
            max(select(max(width_end + width_noise_end, 0.25), tapering.y, connected_end), 0.25),
        );
        let raw_offset =
            (pass_offset(edge_id, pass_index) + wobble + micro)
            * connection_offset_multiplier(connected_start, connected_end, pass_index, pass_render_length);
        let base_overshoot = pass_overshoot(kind, pass_index);
        let render_direction = normalize(pass_render_end - pass_render_start);
        let drift_start = endpoint_tangent_drift_px(
            kind,
            edge_id,
            pass_index,
            connected_start,
            pass_render_length,
            17u,
            bitcast<f32>(uniforms.seed.z),
        );
        let drift_end = endpoint_tangent_drift_px(
            kind,
            edge_id,
            pass_index,
            connected_end,
            pass_render_length,
            41u,
            bitcast<f32>(uniforms.seed.w),
        );
        let stylized_start = pass_render_start + render_direction * drift_start;
        let stylized_end = pass_render_end + render_direction * drift_end;
        let render_normal = normalize(vec2<f32>(-render_direction.y, render_direction.x));
        let curve_noise = coherent_signed_noise_1d(
            seed32,
            edge_id,
            pass_index,
            f32(edge_index % 131u) * uniforms.params10.y + 19.0,
            1237u,
        );
        let curve_offset =
            curve_noise
            * kind_wobble_px(kind)
            * uniforms.params7.y
            * pass_wobble
            * connection_offset_multiplier(connected_start, connected_end, pass_index, pass_render_length)
            * select(0.35, 0.12, debug_mode != 0u);
        let stylized_mid = (stylized_start + stylized_end) * 0.5 + render_normal * curve_offset;
        let mid_width = max(width_mid + width_noise_mid, 0.25);
        let mid_alpha = select(alpha_mid, debug_color.a, debug_mode != 0u);
        let overshoot = debug_overlay_overshoot(debug_mode, base_overshoot);
        stroke_segments[out_index].start = stylized_start;
        stroke_segments[out_index].end = select(stylized_end, stylized_mid, segment_count > 1u);
        stroke_segments[out_index].color = vec4<f32>(debug_color.rgb, debug_color.a);
        stroke_segments[out_index].width_px = debug_widths.x;
        stroke_segments[out_index].offset_px = debug_overlay_offset(debug_mode, raw_offset);
        stroke_segments[out_index].overshoot_start_px = select(overshoot, 0.0, connected_start);
        stroke_segments[out_index].overshoot_end_px = select(select(overshoot, 0.0, connected_end), 0.0, segment_count > 1u);
        stroke_segments[out_index].viewport_half = uniforms.viewport_half.xy;
        stroke_segments[out_index].end_width_px = select(debug_widths.y, mid_width, segment_count > 1u);
        stroke_segments[out_index].end_alpha = select(
            select(select(alpha_end, tapering.w, connected_end), mid_alpha, segment_count > 1u),
            debug_color.a,
            debug_mode != 0u,
        );
        if (segment_count > 1u) {
            let second_index = out_index + 1u;
            stroke_segments[second_index].start = stylized_mid;
            stroke_segments[second_index].end = stylized_end;
            stroke_segments[second_index].color = vec4<f32>(debug_color.rgb, mid_alpha);
            stroke_segments[second_index].width_px = mid_width;
            stroke_segments[second_index].offset_px = debug_overlay_offset(debug_mode, raw_offset);
            stroke_segments[second_index].overshoot_start_px = 0.0;
            stroke_segments[second_index].overshoot_end_px = select(overshoot, 0.0, connected_end);
            stroke_segments[second_index].viewport_half = uniforms.viewport_half.xy;
            stroke_segments[second_index].end_width_px = debug_widths.y;
            stroke_segments[second_index].end_alpha = select(
                select(alpha_end, tapering.w, connected_end),
                debug_color.a,
                debug_mode != 0u,
            );
        }
    }
}
