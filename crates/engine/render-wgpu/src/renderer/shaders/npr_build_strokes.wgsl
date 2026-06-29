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

struct GpuNprPathSegment3d {
    start: vec4<f32>,
    end: vec4<f32>,
    path: vec4<u32>,
    metrics: vec4<f32>,
    style_metrics: vec4<f32>,
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
const PATH_STRATEGY_DIRECT_VISIBLE_SEGMENTS: u32 = 1u;
const CANDIDATE_CHARACTER_SEMANTIC: u32 = 1u;
const STROKE_AKIRA_INK: u32 = 1u;
const HATCHING_SPARSE_CHARACTER: u32 = 1u;
const BUDGET_FACE_SILHOUETTE_PRIORITY: u32 = 1u;
const BUDGET_CHARACTER_READABILITY: u32 = 2u;

@group(0) @binding(2) var<storage, read> edges: array<GpuNprEdge3d>;
@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(6) var<storage, read_write> stroke_segments: array<NprStrokeSegmentVertex>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(9) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(13) var<storage, read> path_segments: array<GpuNprPathSegment3d>;

fn active_edge_count() -> u32 {
    return min(uniforms.pipeline1.w, u32(arrayLength(&visible_segments)));
}

fn uses_direct_visible_segments() -> bool {
    return uniforms.pipeline0.y == PATH_STRATEGY_DIRECT_VISIBLE_SEGMENTS;
}

fn path_segment_base() -> u32 {
    return uniforms.material_roles0.z;
}

fn path_segment_slot_count() -> u32 {
    return uniforms.material_roles0.w;
}

fn uses_character_semantic_candidates() -> bool {
    return uniforms.pipeline0.x == CANDIDATE_CHARACTER_SEMANTIC;
}

fn uses_akira_ink() -> bool {
    return uniforms.pipeline0.z == STROKE_AKIRA_INK;
}

fn uses_character_budget() -> bool {
    return uniforms.pipeline1.y == BUDGET_FACE_SILHOUETTE_PRIORITY
        || uniforms.pipeline1.y == BUDGET_CHARACTER_READABILITY;
}

fn uses_sparse_character_hatching() -> bool {
    return uniforms.pipeline1.x == HATCHING_SPARSE_CHARACTER;
}

fn is_internal_feature_kind(kind: u32) -> bool {
    return kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE;
}

fn camera_response_enabled() -> bool {
    return uniforms.params18.x > 0.5;
}

fn camera_near_strength(depth01: f32) -> f32 {
    if (!camera_response_enabled()) {
        return 0.0;
    }
    return pow(1.0 - clamp(depth01, 0.0, 1.0), 0.82);
}

fn camera_far_strength(depth01: f32) -> f32 {
    if (!camera_response_enabled()) {
        return 0.0;
    }
    return pow(clamp(depth01, 0.0, 1.0), 0.92);
}

fn camera_focus_near_strength() -> f32 {
    if (!camera_response_enabled()) {
        return 0.0;
    }
    return pow(1.0 - clamp(uniforms.params20.w, 0.0, 1.0), 0.82);
}

fn camera_focus_far_strength() -> f32 {
    if (!camera_response_enabled()) {
        return 0.0;
    }
    return pow(clamp(uniforms.params20.w, 0.0, 1.0), 0.88);
}

fn camera_detail_keep_strength(kind: u32, depth01: f32) -> f32 {
    if (!is_internal_feature_kind(kind)) {
        return 0.0;
    }
    let near = max(camera_near_strength(depth01), camera_focus_near_strength() * 0.58);
    return near * clamp(uniforms.params18.z, 0.0, 2.0);
}

fn camera_far_detail_suppression(kind: u32, depth01: f32) -> f32 {
    if (!is_internal_feature_kind(kind) && kind != KIND_CONTACT) {
        return 0.0;
    }
    let far = max(camera_far_strength(depth01), camera_focus_far_strength() * 0.68);
    return far * clamp(uniforms.params19.z, 0.0, 3.0);
}

fn camera_width_multiplier(kind: u32, depth01: f32) -> f32 {
    let near = max(camera_near_strength(depth01), camera_focus_near_strength() * 0.54);
    let far = max(camera_far_strength(depth01), camera_focus_far_strength() * 0.82);
    let near_boost = near * clamp(uniforms.params18.y, 0.0, 2.0);
    let rim_boost = select(0.0, near * clamp(uniforms.params19.w, 0.0, 2.0), kind == KIND_SILHOUETTE);
    let far_scale = select(0.65, 1.0, is_internal_feature_kind(kind) || kind == KIND_CONTACT);
    let far_falloff = far * clamp(uniforms.params19.x, 0.0, 2.0) * far_scale;
    return clamp(1.0 + near_boost + rim_boost - far_falloff, 0.12, 2.35);
}

fn camera_alpha_multiplier(kind: u32, depth01: f32) -> f32 {
    let near = max(camera_near_strength(depth01), camera_focus_near_strength() * 0.36);
    let far = max(camera_far_strength(depth01), camera_focus_far_strength() * 0.88);
    let detail_scale = select(0.34, 1.0, is_internal_feature_kind(kind) || kind == KIND_CONTACT);
    let far_falloff = far * clamp(uniforms.params19.y, 0.0, 2.0) * detail_scale;
    return clamp(1.0 + near * 0.14 - far_falloff, 0.02, 1.22);
}

fn camera_hatch_chance_multiplier(depth01: f32) -> f32 {
    let near = max(camera_near_strength(depth01), camera_focus_near_strength() * 0.78);
    let far = max(camera_far_strength(depth01), camera_focus_far_strength());
    return clamp(1.0 + near * clamp(uniforms.params18.w, 0.0, 3.0) - far * 0.72, 0.0, 3.4);
}

fn camera_front_feature_suppression(kind: u32, depth01: f32, path_coherence: f32) -> f32 {
    if (!is_internal_feature_kind(kind)) {
        return 0.0;
    }
    let front_like = clamp((path_coherence - 0.70) / 0.42, 0.0, 1.0);
    let far_weight = 0.35 + max(camera_far_strength(depth01), camera_focus_far_strength() * 0.72) * 0.65;
    return front_like * far_weight * clamp(uniforms.params20.x, 0.0, 2.0);
}

fn should_emit_sparse_character_hatch(
    kind: u32,
    render_length: f32,
    connected_start: bool,
    connected_end: bool,
    chain_quality: f32,
    path_coherence: f32,
    path_t_mid: f32,
    path_id: u32,
    depth01: f32,
) -> bool {
    if (!uses_sparse_character_hatching()) {
        return false;
    }
    if (!(uses_character_semantic_candidates() || uses_akira_ink() || uses_character_budget())) {
        return false;
    }
    if (!is_internal_feature_kind(kind)) {
        return false;
    }
    let near = camera_near_strength(depth01);
    if (render_length < mix(8.0, 5.0, near) || render_length > mix(44.0, 68.0, near)) {
        return false;
    }
    let connected_count = u32(connected_start) + u32(connected_end);
    if (connected_count == 2u && chain_quality > 0.82) {
        return false;
    }
    if (path_coherence < 0.52) {
        return false;
    }

    let cell = u32(floor(clamp(path_t_mid, 0.0, 1.0) * 37.0));
    let roll = signed_noise_01(
        uniforms.seed.x
            ^ uniforms.seed.y
            ^ path_id
            ^ (cell * 0x9e37u)
            ^ (kind * 0x85ebu)
    );
    let chance =
        select(0.18, 0.30, uses_akira_ink())
        * clamp(1.1 - chain_quality * 0.34, 0.55, 1.0)
        * camera_hatch_chance_multiplier(depth01);
    return roll < chance;
}

fn should_suppress_detail_segment(
    kind: u32,
    render_length: f32,
    path_length: f32,
    connected_start: bool,
    connected_end: bool,
    chain_quality: f32,
    path_coherence: f32,
    depth01: f32,
) -> bool {
    if (!(is_internal_feature_kind(kind) || kind == KIND_CONTACT)) {
        return false;
    }

    let connected_count = u32(connected_start) + u32(connected_end);
    let strict_budget = uses_character_budget() || uses_akira_ink();
    let near_keep = camera_detail_keep_strength(kind, depth01);
    let far_suppress = camera_far_detail_suppression(kind, depth01);
    let camera_length_scale = clamp(1.0 - near_keep * 0.46 + far_suppress * 0.55, 0.42, 2.35);
    let min_detail_length =
        max(uniforms.params0.w * select(1.8, 2.6, strict_budget), select(5.0, 8.0, strict_budget))
        * camera_length_scale;
    if (render_length < min_detail_length && connected_count < 2u) {
        return true;
    }

    if (strict_budget && connected_count == 0u && path_length < max(render_length * 1.35, render_length + 8.0)) {
        return true;
    }

    if (strict_budget && chain_quality < 0.58 && path_coherence < 0.72 && render_length < 18.0 * clamp(1.0 - near_keep * 0.35, 0.55, 1.0)) {
        return true;
    }

    if (kind == KIND_CONTACT && (connected_count < 2u || chain_quality < 0.72)) {
        return true;
    }

    return false;
}

fn artist_selection_strength(kind: u32, path_coherence: f32, importance: f32, depth01: f32) -> f32 {
    if (debug_overlay_mode() != 0u) {
        return 0.0;
    }
    if (kind == KIND_SILHOUETTE) {
        return 0.0;
    }

    let artist_amount = clamp(uniforms.params17.x, 0.0, 3.0);
    if (artist_amount <= 0.001) {
        return 0.0;
    }
    let human = clamp(uniforms.params7.y, 0.0, 1.0);
    let uncertainty = clamp(1.0 - uniforms.params11.y, 0.0, 1.0);
    let semantic_scale = select(0.82, 1.12, uses_character_semantic_candidates() || uses_character_budget() || uses_akira_ink());
    let coherence_scale = clamp(1.12 - path_coherence * 0.36, 0.58, 1.0);
    let importance_guard = clamp(1.12 - importance * 0.78, 0.24, 1.0);
    let camera_keep = camera_detail_keep_strength(kind, depth01);
    let camera_far = camera_far_detail_suppression(kind, depth01);
    let camera_front = camera_front_feature_suppression(kind, depth01, path_coherence);
    let camera_scale = clamp(1.0 - camera_keep * 0.62 + camera_far * 0.72 + camera_front * 0.46, 0.18, 2.25);
    var role_scale = 0.0;
    if (kind == KIND_BOUNDARY) {
        role_scale = 0.28;
    } else if (is_internal_feature_kind(kind)) {
        role_scale = 1.0;
    } else if (kind == KIND_CONTACT) {
        role_scale = 0.72;
    }

    let akira_bias = select(0.0, 0.055, uses_akira_ink());
    return clamp(
        clamp(
            (human * 0.18 + uncertainty * 0.16 + akira_bias)
                * role_scale
                * semantic_scale
                * coherence_scale
                * importance_guard,
            0.0,
            0.34,
        ) * artist_amount * camera_scale,
        0.0,
        0.65,
    );
}

fn should_artist_skip_detail_segment(
    kind: u32,
    path_id: u32,
    render_length: f32,
    path_coherence: f32,
    importance: f32,
    connected_start: bool,
    connected_end: bool,
    depth01: f32,
) -> bool {
    if (!(is_internal_feature_kind(kind) || kind == KIND_CONTACT)) {
        return false;
    }
    let connected_count = u32(connected_start) + u32(connected_end);
    if (connected_count == 2u && path_coherence >= 0.82) {
        return false;
    }

    let strength = artist_selection_strength(kind, path_coherence, importance, depth01);
    if (strength <= 0.001) {
        return false;
    }
    let short_scale = clamp(1.25 - render_length / 42.0, 0.28, 1.0);
    let chance = strength * short_scale * select(1.0, 0.58, connected_count == 2u);
    let roll = signed_noise_01(
        uniforms.seed.x
            ^ uniforms.seed.y
            ^ path_id
            ^ (kind * 0x45d9u)
            ^ (u32(render_length * 17.0) * 0x27d4u)
            ^ 0xa53u,
    );
    return roll < chance;
}

fn artist_gesture_trim_px(
    kind: u32,
    path_id: u32,
    render_length: f32,
    path_coherence: f32,
    importance: f32,
    connected_start: bool,
    connected_end: bool,
    depth01: f32,
) -> vec2<f32> {
    let strength = artist_selection_strength(kind, path_coherence, importance, depth01);
    if (strength <= 0.001 || render_length < max(uniforms.params0.w + 4.0, 8.0)) {
        return vec2<f32>(0.0, 0.0);
    }

    let trim_amount = clamp(uniforms.params17.y, 0.0, 3.0);
    if (trim_amount <= 0.001) {
        return vec2<f32>(0.0, 0.0);
    }
    let max_trim = min(
        render_length * select(0.06, 0.18, is_internal_feature_kind(kind) || kind == KIND_CONTACT),
        select(2.0, 7.0, is_internal_feature_kind(kind) || kind == KIND_CONTACT),
    ) * clamp(strength * 3.2, 0.0, 1.0) * trim_amount;
    var start_trim = signed_noise_01(uniforms.seed.x ^ path_id ^ (kind * 0x9e37u) ^ 0x1111u) * max_trim;
    var end_trim = signed_noise_01(uniforms.seed.y ^ path_id ^ (kind * 0x85ebu) ^ 0x2222u) * max_trim;
    if (connected_start) {
        start_trim = start_trim * 0.35;
    }
    if (connected_end) {
        end_trim = end_trim * 0.35;
    }

    let remaining = render_length - start_trim - end_trim;
    let min_remaining = max(uniforms.params0.w, 5.0);
    if (remaining < min_remaining) {
        let scale = clamp((render_length - min_remaining) / max(start_trim + end_trim, 0.001), 0.0, 1.0);
        start_trim = start_trim * scale;
        end_trim = end_trim * scale;
    }
    return vec2<f32>(max(start_trim, 0.0), max(end_trim, 0.0));
}

fn artist_lift_alpha(
    kind: u32,
    path_id: u32,
    pass_index: u32,
    path_t_mid: f32,
    path_coherence: f32,
    importance: f32,
    connected_start: bool,
    connected_end: bool,
    depth01: f32,
) -> vec3<f32> {
    let strength = artist_selection_strength(kind, path_coherence, importance, depth01);
    if (strength <= 0.001) {
        return vec3<f32>(1.0, 1.0, 1.0);
    }
    let lift_amount = clamp(uniforms.params17.z, 0.0, 3.0);
    if (lift_amount <= 0.001) {
        return vec3<f32>(1.0, 1.0, 1.0);
    }
    let endpoint_strength = clamp(strength * select(1.3, 0.72, kind == KIND_BOUNDARY) * lift_amount, 0.0, 0.42);
    let start_roll = signed_noise_01(uniforms.seed.x ^ path_id ^ pass_index ^ u32(path_t_mid * 8192.0) ^ 0x7001u);
    let end_roll = signed_noise_01(uniforms.seed.y ^ path_id ^ (pass_index * 1597u) ^ u32(path_t_mid * 4096.0) ^ 0x7002u);
    let start_scale = 1.0 - endpoint_strength * start_roll * select(1.0, 0.38, connected_start);
    let end_scale = 1.0 - endpoint_strength * end_roll * select(1.0, 0.38, connected_end);
    let mid_scale = 1.0 - endpoint_strength * 0.18 * min(start_roll, end_roll);
    return vec3<f32>(clamp(start_scale, 0.45, 1.0), clamp(mid_scale, 0.72, 1.0), clamp(end_scale, 0.45, 1.0));
}

fn kind_width_multiplier(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params3.x * select(1.0, 1.08, uses_akira_ink());
    }
    if (kind == KIND_CONTACT) {
        return max(uniforms.params3.z, 1.0) * select(1.0, 0.72, uses_character_budget());
    }
    if (is_internal_feature_kind(kind)) {
        return uniforms.params3.z * select(1.0, 0.74, uses_akira_ink() || uses_character_budget());
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
    if (is_internal_feature_kind(kind)) {
        let akira_scale = select(1.0, 0.82, uses_akira_ink());
        return uniforms.params4.z * uniforms.params16.y * 0.82 * akira_scale;
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
    if (is_internal_feature_kind(kind)) {
        return uniforms.params12.z * select(1.0, 0.65, uses_akira_ink());
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
    path_id: u32,
    render_length: f32,
    path_t0: f32,
    path_t1: f32,
    path_coherence: f32,
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
    let chance = effective * coverage * clamp(1.22 - path_coherence * 0.32, 0.58, 1.18);
    let path_t_mid = clamp((path_t0 + path_t1) * 0.5, 0.0, 1.0);
    let dropout_cells = max(uniforms.params4.w, 4.0);
    let cell = u32(floor(path_t_mid * dropout_cells));
    let roll = signed_noise_01(
        uniforms.seed.x
        ^ uniforms.seed.y
        ^ path_id
        ^ (pass_index * 92821u)
        ^ (cell * 0xA5D3u)
    );
    return roll < chance;
}

fn connection_offset_multiplier(
    connected_start: bool,
    connected_end: bool,
    pass_index: u32,
    render_length: f32,
    path_length: f32,
    path_t0: f32,
    path_t1: f32,
) -> f32 {
    let is_search = pass_index >= u32(uniforms.params5.x);
    let start_lock_px = max(bitcast<f32>(uniforms.seed.z), 0.0);
    let end_lock_px = max(bitcast<f32>(uniforms.seed.w), 0.0);
    let lock_span = clamp((start_lock_px + end_lock_px) / max(render_length, 1.0), 0.0, 1.0);
    let path_lock = path_endpoint_lock_weight(path_t0, path_t1, path_length);
    let lock_factor = (1.0 - lock_span * 0.45) * path_lock;
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
    path_length: f32,
    salt: u32,
    endpoint_lock_px: f32,
    path_t: f32,
    path_coherence: f32,
) -> f32 {
    if (connected) {
        return 0.0;
    }
    let tangent_scale = uniforms.params11.x;
    if (abs(tangent_scale) <= 0.0001) {
        return 0.0;
    }
    let lock_factor =
        (1.0 - clamp(endpoint_lock_px / max(render_length, 1.0), 0.0, 0.85) * 0.55)
        * path_endpoint_lock_at(path_t, path_length);
    let drift = coherent_signed_noise_1d(
        uniforms.seed.x ^ uniforms.seed.y,
        edge_id,
        pass_index,
        f32(edge_id % 89u) * 0.17 + f32(pass_index) * 0.31 + f32(salt) * 0.013,
        977u + salt,
    );
    return drift
        * max(kind_wobble_px(kind), 0.05)
        * tangent_scale
        * lock_factor
        * path_humanization_scale(path_coherence);
}

fn path_endpoint_lock_at(path_t: f32, path_length: f32) -> f32 {
    let start_lock_px = max(bitcast<f32>(uniforms.seed.z), 0.0);
    let end_lock_px = max(bitcast<f32>(uniforms.seed.w), 0.0);
    let start_lock_t = clamp(start_lock_px / max(path_length, 1.0), 0.0, 0.5);
    let end_lock_t = clamp(end_lock_px / max(path_length, 1.0), 0.0, 0.5);
    let t = clamp(path_t, 0.0, 1.0);
    let start_weight = smoothstep(0.0, max(start_lock_t, 0.0001), t);
    let end_weight = smoothstep(0.0, max(end_lock_t, 0.0001), 1.0 - t);
    return clamp(start_weight * end_weight, 0.0, 1.0);
}

fn path_endpoint_lock_weight(path_t0: f32, path_t1: f32, path_length: f32) -> f32 {
    let a = path_endpoint_lock_at(path_t0, path_length);
    let b = path_endpoint_lock_at(path_t1, path_length);
    let mid = path_endpoint_lock_at((path_t0 + path_t1) * 0.5, path_length);
    return clamp(min(a, min(b, mid)), 0.0, 1.0);
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

fn distance_width_multiplier(kind: u32, importance: f32, depth01: f32) -> f32 {
    let pressure_boost = 1.0 + uniforms.params7.w * (importance - 1.0);
    return clamp((1.0 - uniforms.params7.z * (1.0 - importance)) * pressure_boost, 0.62, 1.28)
        * camera_width_multiplier(kind, depth01);
}

fn depth_alpha_multiplier(kind: u32, importance: f32, depth01: f32) -> f32 {
    let near = pow(clamp(importance, 0.0, 1.35), 0.8);
    return clamp(1.0 + uniforms.params11.z * (near - 0.5), 0.35, 1.25)
        * camera_alpha_multiplier(kind, depth01);
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

fn path_coherence_score(
    path_length: f32,
    hop_count: u32,
    connected_start: bool,
    connected_end: bool,
) -> f32 {
    let length_factor = clamp(path_length / 96.0, 0.2, 1.0);
    let hop_factor = clamp(f32(min(hop_count, 5u)) / 5.0, 0.0, 1.0);
    let connection_factor =
        select(0.72, 1.0, connected_start && connected_end) * select(1.0, 0.88, connected_start != connected_end);
    return clamp(length_factor * 0.46 + hop_factor * 0.34 + connection_factor * 0.32, 0.35, 1.18);
}

fn path_humanization_scale(path_coherence: f32) -> f32 {
    return clamp(1.16 - path_coherence * 0.26, 0.78, 1.08);
}

fn pass_width(kind: u32, pass_index: u32, importance: f32, t: f32, path_coherence: f32, depth01: f32) -> f32 {
    let base = uniforms.params1.x * kind_width_multiplier(kind) * uniforms.params6.x;
    let coherence_width = clamp(0.94 + path_coherence * 0.1, 0.9, 1.08);
    let is_search = pass_index >= u32(uniforms.params5.x);
    if (is_search) {
        return max(
            base
                * 0.78
                * pressure_multiplier(t)
                * taper_multiplier(t)
                * coherence_width
                * distance_width_multiplier(kind, importance, depth01),
            0.25,
        );
    }
    return max(
        base
            * primary_pass_width_multiplier(u32(uniforms.params5.x), pass_index)
            * pressure_multiplier(t)
            * taper_multiplier(t)
            * coherence_width
            * distance_width_multiplier(kind, importance, depth01),
        0.25,
    );
}

fn pass_alpha(kind: u32, pass_index: u32, importance: f32, t: f32, path_coherence: f32, depth01: f32) -> f32 {
    let base = uniforms.ink_color.w * kind_alpha_multiplier(kind) * uniforms.params6.y;
    let coherence_alpha = clamp(0.9 + path_coherence * 0.14, 0.82, 1.08);
    let is_search = pass_index >= u32(uniforms.params5.x);
    if (is_search) {
        return clamp(
            base
                * uniforms.params5.z
                * npr_gpu_search_alpha_multiplier()
                * uniforms.params6.y
                * alpha_pressure_multiplier(t)
                * coherence_alpha
                * depth_alpha_multiplier(kind, importance, depth01),
            0.0,
            1.0,
        );
    }
    return clamp(
        base
            * primary_pass_alpha_multiplier(u32(uniforms.params5.x), pass_index)
            * alpha_pressure_multiplier(t)
            * coherence_alpha
            * depth_alpha_multiplier(kind, importance, depth01),
        0.0,
        1.0,
    );
}

fn pass_overshoot(kind: u32, pass_index: u32, path_coherence: f32) -> f32 {
    let base = select(
        uniforms.params1.y,
        uniforms.params4.w,
        kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE || kind == KIND_CONTACT,
    );
    let is_search = pass_index >= u32(uniforms.params5.x);
    let clamped = min(base, 0.5);
    let coherence_scale = clamp(0.86 + path_coherence * 0.16, 0.82, 1.05);
    return select(clamped, min(max(clamped, uniforms.params11.w), 0.15), is_search) * coherence_scale;
}

fn endpoint_connection_score(
    next_edge_index: u32,
    kind: u32,
    anchor_point: vec2<f32>,
    anchor_depth: f32,
    current_direction: vec2<f32>,
    current_length: f32,
) -> f32 {
    if (next_edge_index == 0xffffffffu || next_edge_index >= active_edge_count()) {
        return 0.0;
    }
    let next = visible_segments[next_edge_index];
    if (next.start.w <= 0.5 || next.end.w <= 0.5 || next.kind_edge.x != kind) {
        return 0.0;
    }
    let match_is_start = continuation_match_is_start(anchor_point, next_edge_index);
    if (!matched_endpoint_is_valid(next, match_is_start)) {
        return 0.0;
    }
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
    if (next_edge_index == 0xffffffffu || next_edge_index >= active_edge_count()) {
        return 0xffffffffu;
    }
    if (!matched_endpoint_is_valid(visible_segments[next_edge_index], matched_start)) {
        return 0xffffffffu;
    }
    let next_edge = edges[next_edge_index];
    return select(next_edge.next_a, next_edge.next_b, matched_start);
}

fn endpoint_degree(edge: GpuNprEdge3d, matched_start: bool) -> u32 {
    return select(edge.degree_a, edge.degree_b, matched_start);
}

fn valid_endpoint_vertex(vertex: u32) -> bool {
    return vertex != 0xffffffffu;
}

fn visible_endpoint_vertex(visible: GpuNprVisibleSegment3d, matched_start: bool) -> u32 {
    return select(visible.kind_edge.w, visible.kind_edge.z, matched_start);
}

fn matched_endpoint_is_valid(visible: GpuNprVisibleSegment3d, matched_start: bool) -> bool {
    return valid_endpoint_vertex(visible_endpoint_vertex(visible, matched_start));
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
    if (edge_index == 0xffffffffu || edge_index >= active_edge_count()) {
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
        if (edge_index == 0xffffffffu || edge_index >= active_edge_count()) {
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
        if (!matched_endpoint_is_valid(visible_segments[edge_index], matched_start)) {
            break;
        }
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

    let start_match_is_start = continuation_match_is_start(screen_a, start_next_edge);
    let end_match_is_start = continuation_match_is_start(screen_b, end_next_edge);
    if (
        !matched_endpoint_is_valid(visible_segments[start_next_edge], start_match_is_start)
        || !matched_endpoint_is_valid(visible_segments[end_next_edge], end_match_is_start)
    ) {
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
    let path_segment_count = path_segment_slot_count();
    let path_segment_index = path_segment_base() + id.x;
    if (
        id.x >= path_segment_count
        || path_segment_index >= u32(arrayLength(&path_segments))
    ) {
        return;
    }

    let segment = path_segments[path_segment_index];
    if (segment.start.w < 0.5 || segment.end.w < 0.5) {
        return;
    }

    let kind = segment.path.y;
    if (kind == KIND_NONE) {
        return;
    }
    let primary_pass_count = max(u32(uniforms.params5.x), 1u);
    let path_id = segment.path.x;
    let path_flags = segment.path.w;
    let hop_count = segment.path.z;
    let edge_id = path_id;
    let screen_a = segment.start.xy;
    let screen_b = segment.end.xy;
    let path_t0 = clamp(segment.metrics.x, 0.0, 1.0);
    let path_t1 = clamp(segment.metrics.y, 0.0, 1.0);
    let path_t_mid = clamp((path_t0 + path_t1) * 0.5, 0.0, 1.0);
    let path_length = max(segment.metrics.z, distance(screen_a, screen_b));
    let local_segment_length = distance(screen_a, screen_b);
    let line_depth = clamp(segment.style_metrics.z, 0.0, 1.0);
    var render_start_mut = screen_a;
    var render_end_mut = screen_b;
    var render_length = local_segment_length;
    if (render_length < uniforms.params0.w) {
        return;
    }
    let importance = segment.metrics.w;
    let seed32 = uniforms.seed.x ^ uniforms.seed.y;
    let connected_start = (path_flags & PATH_FLAG_CONNECTED_START) != 0u;
    let connected_end = (path_flags & PATH_FLAG_CONNECTED_END) != 0u;
    let path_coherence = path_coherence_score(
        path_length,
        hop_count,
        connected_start,
        connected_end,
    );
    let chain_quality = clamp(
        select(0.42, 0.86, connected_start && connected_end) * path_coherence,
        0.22,
        1.1,
    );
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
    if (render_length > min(viewport_diagonal * 0.82, max(local_segment_length * 3.2, local_segment_length + 96.0))) {
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
    if (should_suppress_detail_segment(
        kind,
        render_length,
        path_length,
        connected_start,
        connected_end,
        chain_quality,
        path_coherence,
        line_depth,
    )) {
        return;
    }
    if (should_artist_skip_detail_segment(
        kind,
        path_id,
        render_length,
        path_coherence,
        importance,
        connected_start,
        connected_end,
        line_depth,
    )) {
        return;
    }
    let artist_trim = artist_gesture_trim_px(
        kind,
        path_id,
        render_length,
        path_coherence,
        importance,
        connected_start,
        connected_end,
        line_depth,
    );
    if (artist_trim.x + artist_trim.y > 0.001) {
        let trim_direction = normalize(render_end_mut - render_start_mut);
        render_start_mut = render_start_mut + trim_direction * artist_trim.x;
        render_end_mut = render_end_mut - trim_direction * artist_trim.y;
        render_length = distance(render_start_mut, render_end_mut);
        if (render_length < uniforms.params0.w) {
            return;
        }
    }
    let search_enabled = should_enable_search_passes(
        kind,
        edge_id,
        edge_id,
        max(local_segment_length, 1.0),
        max(path_length, local_segment_length),
        max(local_segment_length * clamp(1.12 - path_coherence * 0.18, 0.72, 1.08), 1.0),
        chain_quality,
        connected_start,
        connected_end,
        local_segment_length,
        render_length,
        path_length,
        path_coherence - 0.72,
    );
    let search_pass_count = select(0u, u32(uniforms.params5.y), search_enabled);
    let hatch_enabled = should_emit_sparse_character_hatch(
        kind,
        render_length,
        connected_start,
        connected_end,
        chain_quality,
        path_coherence,
        path_t_mid,
        path_id,
        line_depth,
    );
    let hatch_start_index = primary_pass_count + search_pass_count;
    let hatch_pass_count = select(0u, 1u, hatch_enabled);
    let total_pass_count = hatch_start_index + hatch_pass_count;

    for (var pass_index: u32 = 0u; pass_index < total_pass_count; pass_index = pass_index + 1u) {
        let is_hatch_pass = pass_index >= hatch_start_index;
        var pass_render_start = render_start_mut;
        var pass_render_end = render_end_mut;
        var pass_render_length = render_length;
        if (pass_index >= primary_pass_count && !is_hatch_pass) {
            let search_max_length = npr_gpu_search_max_render_length_px();
            if (pass_render_length > search_max_length) {
                let trunc_direction = normalize(pass_render_end - pass_render_start);
                pass_render_end = pass_render_start + trunc_direction * search_max_length;
                pass_render_length = search_max_length;
            }
        }
        if (is_hatch_pass) {
            let hatch_noise = signed_noise_01(seed32 ^ path_id ^ u32(path_t_mid * 4096.0) ^ 0x51edu);
            let hatch_length = min(pass_render_length, mix(7.0, 16.0, hatch_noise));
            let hatch_center = mix(pass_render_start, pass_render_end, clamp(0.42 + signed_noise(seed32 ^ path_id ^ 0xabcdu) * 0.18, 0.25, 0.75));
            let hatch_direction = normalize(pass_render_end - pass_render_start);
            pass_render_start = hatch_center - hatch_direction * (hatch_length * 0.5);
            pass_render_end = hatch_center + hatch_direction * (hatch_length * 0.5);
            pass_render_length = hatch_length;
        }
        if (should_drop_segment_instance(
            kind,
            pass_index,
            path_id,
            pass_render_length,
            path_t0,
            path_t1,
            path_coherence,
        )) {
            continue;
        }
        let max_segment_length = npr_gpu_max_segment_length_px();
        let path_span = path_t1 - path_t0;
        let segment_count =
            select(
                select(
                    1u,
                    2u,
                    pass_render_length >= max_segment_length && !is_hatch_pass,
                ),
                3u,
                !is_hatch_pass
                    && (
                        pass_render_length >= max_segment_length * 1.9
                        || (pass_render_length >= max_segment_length * 1.15 && path_span >= 0.22)
                    ),
            );
        let out_index = atomicAdd(&indirect_args[1], segment_count);
        if (out_index + segment_count > u32(arrayLength(&stroke_segments))) {
            _ = atomicSub(&indirect_args[1], segment_count);
            return;
        }
        var width_start = pass_width(kind, pass_index, importance, path_t0, path_coherence, line_depth);
        var width_mid = pass_width(kind, pass_index, importance, path_t_mid, path_coherence, line_depth);
        var width_end = pass_width(kind, pass_index, importance, path_t1, path_coherence, line_depth);
        var alpha_start = pass_alpha(kind, pass_index, importance, path_t0, path_coherence, line_depth);
        var alpha_mid = pass_alpha(kind, pass_index, importance, path_t_mid, path_coherence, line_depth);
        var alpha_end = pass_alpha(kind, pass_index, importance, path_t1, path_coherence, line_depth);
        let artist_alpha = artist_lift_alpha(
            kind,
            path_id,
            pass_index,
            path_t_mid,
            path_coherence,
            importance,
            connected_start,
            connected_end,
            line_depth,
        );
        alpha_start = alpha_start * artist_alpha.x;
        alpha_mid = alpha_mid * artist_alpha.y;
        alpha_end = alpha_end * artist_alpha.z;
        if (is_hatch_pass) {
            let hatch_camera = camera_hatch_chance_multiplier(line_depth);
            let hatch_width = max(uniforms.params1.x * kind_width_multiplier(kind) * 0.24 * clamp(0.82 + hatch_camera * 0.22, 0.65, 1.65), 0.25);
            let hatch_alpha = clamp(uniforms.ink_color.w * kind_alpha_multiplier(kind) * 0.38 * clamp(0.72 + hatch_camera * 0.34, 0.25, 1.8), 0.0, 0.74);
            width_start = hatch_width * 0.72;
            width_mid = hatch_width;
            width_end = hatch_width * 0.62;
            alpha_start = hatch_alpha * 0.72;
            alpha_mid = hatch_alpha;
            alpha_end = hatch_alpha * 0.58;
        }
        let width_noise_start =
            coherent_signed_noise_1d(seed32, edge_id, pass_index, path_t0 * 13.0 + 7.0, 503u)
            * uniforms.params10.x
            * uniforms.params6.z
            * path_humanization_scale(path_coherence);
        let width_noise_mid =
            coherent_signed_noise_1d(seed32, edge_id, pass_index, path_t_mid * 13.0 + 9.0, 503u)
            * uniforms.params10.x
            * uniforms.params6.z
            * path_humanization_scale(path_coherence);
        let width_noise_end =
            coherent_signed_noise_1d(seed32, edge_id, pass_index, path_t1 * 13.0 + 11.0, 503u)
            * uniforms.params10.x
            * uniforms.params6.z
            * path_humanization_scale(path_coherence);
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
            path_t_mid * 19.0 + f32(path_id % 101u) * uniforms.params10.y + 3.0,
            919u,
        ) * kind_wobble_px(kind) * uniforms.params7.y * pass_wobble * path_humanization_scale(path_coherence);
        let micro = coherent_signed_noise_1d(
            seed32,
            edge_id,
            pass_index,
            path_t_mid * 29.0 + f32(path_id % 71u) * uniforms.params10.w + 13.0,
            991u,
        ) * uniforms.params10.z * pass_wobble * path_humanization_scale(path_coherence);
        let debug_color = debug_color_for_overlay(
            kind,
            pass_index,
            path_id,
            path_segment_index,
            connected_start,
            connected_end,
            importance,
            importance,
            chain_quality,
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
            * connection_offset_multiplier(
                connected_start,
                connected_end,
                pass_index,
                pass_render_length,
                path_length,
                path_t0,
                path_t1,
            );
        let base_overshoot = pass_overshoot(kind, pass_index, path_coherence);
        let path_lock_weight = path_endpoint_lock_weight(path_t0, path_t1, path_length);
        let render_direction = normalize(pass_render_end - pass_render_start);
        let drift_start = endpoint_tangent_drift_px(
            kind,
            edge_id,
            pass_index,
            connected_start,
            pass_render_length,
            path_length,
            17u,
            bitcast<f32>(uniforms.seed.z),
            path_t0,
            path_coherence,
        );
        let drift_end = endpoint_tangent_drift_px(
            kind,
            edge_id,
            pass_index,
            connected_end,
            pass_render_length,
            path_length,
            41u,
            bitcast<f32>(uniforms.seed.w),
            path_t1,
            path_coherence,
        );
        let stylized_start = pass_render_start + render_direction * drift_start;
        let stylized_end = pass_render_end + render_direction * drift_end;
        let render_normal = normalize(vec2<f32>(-render_direction.y, render_direction.x));
        let curve_noise = coherent_signed_noise_1d(
            seed32,
            edge_id,
            pass_index,
            path_t_mid * 37.0 + f32(path_id % 131u) * uniforms.params10.y + 19.0,
            1237u,
        );
        let gesture_span_scale =
            clamp(pass_render_length / max(uniforms.params0.w * 14.0, 24.0), 0.55, 1.65);
        let connected_gesture_scale =
            select(0.92, select(1.12, 1.28, connected_start && connected_end), connected_start || connected_end);
        let curve_offset =
            curve_noise
            * kind_wobble_px(kind)
            * uniforms.params7.y
            * pass_wobble
            * path_humanization_scale(path_coherence)
            * gesture_span_scale
            * connected_gesture_scale
            * connection_offset_multiplier(
                connected_start,
                connected_end,
                pass_index,
                pass_render_length,
                path_length,
                path_t0,
                path_t1,
            )
            * select(0.62, 0.12, debug_mode != 0u);
        let stylized_mid = (stylized_start + stylized_end) * 0.5 + render_normal * curve_offset;
        let stylized_mid_a =
            mix(stylized_start, stylized_mid, 0.5) + render_normal * (curve_offset * 0.35);
        let stylized_mid_b =
            mix(stylized_mid, stylized_end, 0.5) + render_normal * (curve_offset * 0.35);
        let mid_width = max(width_mid + width_noise_mid, 0.25);
        let end_width_value = max(select(max(width_end + width_noise_end, 0.25), tapering.y, connected_end), 0.25);
        let mid_a_width = max(mix(tapering.x, mid_width, 0.5), 0.25);
        let mid_b_width = max(mix(mid_width, end_width_value, 0.5), 0.25);
        let mid_alpha = select(alpha_mid, debug_color.a, debug_mode != 0u);
        let end_alpha_value = select(alpha_end, tapering.w, connected_end);
        let mid_a_alpha = select(mix(tapering.z, mid_alpha, 0.5), debug_color.a, debug_mode != 0u);
        let mid_b_alpha = select(mix(mid_alpha, end_alpha_value, 0.5), debug_color.a, debug_mode != 0u);
        let overshoot = debug_overlay_overshoot(debug_mode, base_overshoot * path_lock_weight);
        stroke_segments[out_index].start = stylized_start;
        stroke_segments[out_index].end = select(
            select(stylized_end, stylized_mid, segment_count > 1u),
            stylized_mid_a,
            segment_count > 2u,
        );
        stroke_segments[out_index].color = vec4<f32>(debug_color.rgb, debug_color.a);
        stroke_segments[out_index].width_px = debug_widths.x;
        stroke_segments[out_index].offset_px = debug_overlay_offset(debug_mode, raw_offset);
        stroke_segments[out_index].overshoot_start_px = select(overshoot, 0.0, connected_start);
        stroke_segments[out_index].overshoot_end_px = select(select(overshoot, 0.0, connected_end), 0.0, segment_count > 1u);
        stroke_segments[out_index].viewport_half = uniforms.viewport_half.xy;
        stroke_segments[out_index].end_width_px = select(
            select(debug_widths.y, mid_width, segment_count > 1u),
            mid_a_width,
            segment_count > 2u,
        );
        stroke_segments[out_index].end_alpha = select(
            select(select(end_alpha_value, mid_alpha, segment_count > 1u), mid_a_alpha, segment_count > 2u),
            debug_color.a,
            debug_mode != 0u,
        );
        if (segment_count > 1u) {
            let second_index = out_index + 1u;
            stroke_segments[second_index].start = select(stylized_mid, stylized_mid_a, segment_count > 2u);
            stroke_segments[second_index].end = select(stylized_end, stylized_mid_b, segment_count > 2u);
            stroke_segments[second_index].color = vec4<f32>(debug_color.rgb, select(mid_alpha, mid_a_alpha, segment_count > 2u));
            stroke_segments[second_index].width_px = select(mid_width, mid_a_width, segment_count > 2u);
            stroke_segments[second_index].offset_px = debug_overlay_offset(debug_mode, raw_offset);
            stroke_segments[second_index].overshoot_start_px = 0.0;
            stroke_segments[second_index].overshoot_end_px = select(overshoot, 0.0, connected_end);
            stroke_segments[second_index].viewport_half = uniforms.viewport_half.xy;
            stroke_segments[second_index].end_width_px = select(debug_widths.y, mid_b_width, segment_count > 2u);
            stroke_segments[second_index].end_alpha = select(
                select(end_alpha_value, mid_b_alpha, segment_count > 2u),
                debug_color.a,
                debug_mode != 0u,
            );
            if (segment_count > 2u) {
                let third_index = out_index + 2u;
                stroke_segments[third_index].start = stylized_mid_b;
                stroke_segments[third_index].end = stylized_end;
                stroke_segments[third_index].color = vec4<f32>(debug_color.rgb, mid_b_alpha);
                stroke_segments[third_index].width_px = mid_b_width;
                stroke_segments[third_index].offset_px = debug_overlay_offset(debug_mode, raw_offset);
                stroke_segments[third_index].overshoot_start_px = 0.0;
                stroke_segments[third_index].overshoot_end_px = select(overshoot, 0.0, connected_end);
                stroke_segments[third_index].viewport_half = uniforms.viewport_half.xy;
                stroke_segments[third_index].end_width_px = debug_widths.y;
                stroke_segments[third_index].end_alpha = select(
                    end_alpha_value,
                    debug_color.a,
                    debug_mode != 0u,
                );
            }
        }
    }
}
