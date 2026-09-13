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
const KIND_CREASE: u32 = 3u;
const KIND_SEAM: u32 = 4u;
const KIND_FEATURE: u32 = 5u;
const KIND_CONTACT: u32 = 6u;
const SEGMENT_TRAIT_MATERIAL_DETAIL: u32 = 1u;
const SEGMENT_TRAIT_MATERIAL_SEAM: u32 = 2u;
const CANDIDATE_CHARACTER_SEMANTIC: u32 = 1u;
const STROKE_AKIRA_INK: u32 = 1u;
const STROKE_CONFIDENT_MANGA_INK: u32 = 4u;
const BUDGET_FACE_SILHOUETTE_PRIORITY: u32 = 1u;
const BUDGET_CHARACTER_READABILITY: u32 = 2u;

@group(0) @binding(0) var<storage, read> vertices: array<GpuNprVertex3d>;
@group(0) @binding(1) var<storage, read> triangles: array<GpuNprTriangle3d>;
@group(0) @binding(2) var<storage, read> edges: array<GpuNprEdge3d>;
@group(0) @binding(3) var<storage, read> projected_vertices: array<GpuNprProjectedVertex3d>;
@group(0) @binding(4) var face_id_texture: texture_2d<u32>;
@group(0) @binding(5) var<storage, read_write> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;

fn clear_visible_segment(edge_index: u32) {
    visible_segments[edge_index].start = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    visible_segments[edge_index].end = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    visible_segments[edge_index].kind_edge = vec4<u32>(0u, 0u, 0u, 0u);
    visible_segments[edge_index].metrics = vec4<f32>(0.0);
}

fn camera_response_enabled() -> bool {
    return uniforms.params18.x > 0.5;
}

fn camera_response_depth01(view_depth: f32, fallback_depth01: f32) -> f32 {
    if (!camera_response_enabled()) {
        return fallback_depth01;
    }
    let near_distance = max(uniforms.params20.y, 0.001);
    let far_distance = max(uniforms.params20.z, near_distance + 0.001);
    return clamp((view_depth - near_distance) / (far_distance - near_distance), 0.0, 1.0);
}

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

fn triangle_view_alignment(face_index: u32) -> f32 {
    let world_normal = transformed_normal(face_index);
    let to_camera = normalize(uniforms.camera_translation.xyz - triangle_center(face_index));
    return abs(dot(world_normal, to_camera));
}

fn feature_edge(edge: GpuNprEdge3d) -> bool {
    if (edge.face_count < 2u || edge.face0 >= u32(arrayLength(&triangles)) || edge.face1 >= u32(arrayLength(&triangles))) {
        return false;
    }
    let left = transformed_normal(edge.face0);
    let right = transformed_normal(edge.face1);
    return dot(left, right) <= uniforms.params2.w;
}

fn uses_character_semantic_candidates() -> bool {
    return uniforms.pipeline0.x == CANDIDATE_CHARACTER_SEMANTIC;
}

fn uses_character_budget() -> bool {
    return uniforms.pipeline1.y == BUDGET_FACE_SILHOUETTE_PRIORITY
        || uniforms.pipeline1.y == BUDGET_CHARACTER_READABILITY;
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

fn is_internal_feature_kind(kind: u32) -> bool {
    return kind == KIND_CREASE || kind == KIND_SEAM || kind == KIND_FEATURE;
}

fn material_id_in_mask(material_id: u32, mask: u32) -> bool {
    return material_id < 32u && (mask & (1u << material_id)) != 0u;
}

fn edge_touches_material_mask(edge: GpuNprEdge3d, mask: u32) -> bool {
    if (mask == 0u) {
        return false;
    }
    if (edge.face0 < u32(arrayLength(&triangles)) && material_id_in_mask(triangles[edge.face0].material_id, mask)) {
        return true;
    }
    if (edge.face_count > 1u && edge.face1 < u32(arrayLength(&triangles)) && material_id_in_mask(triangles[edge.face1].material_id, mask)) {
        return true;
    }
    return false;
}

fn edge_touches_ink_detail_material(edge: GpuNprEdge3d) -> bool {
    return edge_touches_material_mask(edge, uniforms.material_roles0.y);
}

fn kind_min_screen_length_px(kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE) {
        return uniforms.params36.x;
    }
    if (kind == KIND_BOUNDARY) {
        return uniforms.params36.y;
    }
    if (kind == KIND_CREASE) {
        return uniforms.params36.w;
    }
    if (kind == KIND_SEAM) {
        return uniforms.params37.x;
    }
    if (kind == KIND_CONTACT) {
        return uniforms.params37.y;
    }
    return uniforms.params36.z;
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

fn feature_min_length_px() -> f32 {
    let semantic_scale = select(1.0, 1.35, uses_character_semantic_candidates() || uses_character_budget());
    return max(
        kind_min_screen_length_px(KIND_FEATURE) * max(uniforms.params16.x, 0.1) * 1.20 * semantic_scale,
        kind_min_screen_length_px(KIND_FEATURE),
    );
}

fn edge_feature_min_length_px(edge: GpuNprEdge3d) -> f32 {
    return feature_min_length_px() * select(1.0, 0.55, edge_touches_ink_detail_material(edge));
}

fn silhouette_min_length_px() -> f32 {
    let semantic_scale = select(1.0, 0.82, uses_character_semantic_candidates() || uses_character_budget());
    return max(
        kind_min_screen_length_px(KIND_SILHOUETTE) * max(uniforms.params16.z, 0.1) * semantic_scale,
        kind_min_screen_length_px(KIND_SILHOUETTE),
    );
}

fn contact_min_length_px() -> f32 {
    return max(
        max(kind_min_screen_length_px(KIND_CONTACT), feature_min_length_px())
            * select(1.0, 1.45, uses_character_budget()),
        6.0,
    );
}

fn edge_curvature_proxy(edge: GpuNprEdge3d, kind: u32) -> f32 {
    if (kind == KIND_SILHOUETTE || kind == KIND_BOUNDARY || kind == KIND_CONTACT) {
        return 1.0;
    }
    if (edge.face_count < 2u || edge.face0 >= u32(arrayLength(&triangles)) || edge.face1 >= u32(arrayLength(&triangles))) {
        return 0.0;
    }
    let left = transformed_normal(edge.face0);
    let right = transformed_normal(edge.face1);
    return clamp(1.0 - abs(dot(left, right)), 0.0, 1.0);
}

fn edge_semantic_importance(edge: GpuNprEdge3d, kind: u32, line_length: f32, depth01: f32) -> f32 {
    var base = 0.34;
    if (kind == KIND_SILHOUETTE) {
        base = 1.0;
    } else if (kind == KIND_BOUNDARY) {
        base = 0.86;
    } else if (kind == KIND_SEAM) {
        base = 0.66;
    } else if (kind == KIND_CREASE) {
        base = 0.46;
    } else if (kind == KIND_FEATURE) {
        base = 0.42;
    } else if (kind == KIND_CONTACT) {
        base = 0.50;
    }

    let curvature = edge_curvature_proxy(edge, kind);
    let technical_pref = kind_technical_detail_preference(kind);
    let material_pref = kind_ink_detail_material_preference(kind);
    let seam_pref = kind_material_seam_preference(kind);
    let material_detail = select(0.0, 0.10 + material_pref * 0.24, edge_touches_ink_detail_material(edge));
    let seam_boost = select(0.0, 0.06 + seam_pref * 0.20, edge.material_seam > 0u || kind == KIND_SEAM);
    let preferred_length = max(kind_preferred_stroke_length_px(kind), 24.0);
    let length_boost = clamp(line_length / max(preferred_length * 0.8, 1.0), 0.0, 1.0) * 0.16;
    let near_boost = (1.0 - clamp(depth01, 0.0, 1.0)) * select(0.04, 0.18, is_internal_feature_kind(kind));
    let far_penalty = clamp(depth01, 0.0, 1.0) * select(0.0, 0.18, is_internal_feature_kind(kind) || kind == KIND_CONTACT);
    let technical_bias = select(0.0, technical_pref * 0.16, is_internal_feature_kind(kind) || kind == KIND_CONTACT);
    return clamp(
        base + curvature * 0.28 + technical_bias + material_detail + seam_boost + length_boost + near_boost - far_penalty,
        0.0,
        1.28,
    );
}

fn edge_trait_flags(edge: GpuNprEdge3d, kind: u32) -> u32 {
    var flags = 0u;
    if (edge_touches_ink_detail_material(edge)) {
        flags = flags | SEGMENT_TRAIT_MATERIAL_DETAIL;
    }
    if (edge.material_seam > 0u || kind == KIND_SEAM) {
        flags = flags | SEGMENT_TRAIT_MATERIAL_SEAM;
    }
    return flags;
}

fn should_reject_character_edge(edge: GpuNprEdge3d, kind: u32, importance: f32, depth01: f32) -> bool {
    if (!(uses_character_semantic_candidates() || uses_character_budget())) {
        return false;
    }
    if (kind == KIND_SILHOUETTE || kind == KIND_BOUNDARY) {
        return false;
    }

    let material_detail = edge_touches_ink_detail_material(edge);
    let protected_seam = kind == KIND_SEAM || edge.material_seam > 0u;
    if (is_internal_feature_kind(kind) && !material_detail && !protected_seam) {
        let curvature = edge_curvature_proxy(edge, kind);
        let flat_threshold = select(0.030, 0.055, uses_confident_manga_ink() || uses_akira_ink());
        if (curvature < flat_threshold) {
            return true;
        }
    }
    let budget_threshold = select(0.50, 0.40, uses_akira_ink());
    let base_threshold = select(0.38, budget_threshold, uses_character_budget());
    let material_relief = select(0.0, 0.08 + kind_ink_detail_material_preference(kind) * 0.22, material_detail);
    let seam_relief =
        select(0.0, 0.04 + kind_material_seam_preference(kind) * 0.18, protected_seam);
    let near_relief = (1.0 - clamp(depth01, 0.0, 1.0)) * 0.14;
    let family_relief = kind_technical_detail_keep(kind) * 0.18 + kind_technical_detail_preference(kind) * 0.14;
    let threshold = clamp(base_threshold - material_relief - seam_relief - near_relief - family_relief, 0.18, 0.62);
    return importance < threshold;
}

fn screen_to_face_id_texel(screen: vec2<f32>) -> vec2<i32> {
    let dims = vec2<i32>(textureDimensions(face_id_texture));
    let normalized = (screen / uniforms.viewport_half.xy + vec2<f32>(1.0, 1.0)) * 0.5;
    let texel = vec2<i32>(vec2<f32>(normalized.x, 1.0 - normalized.y) * vec2<f32>(dims));
    return clamp(texel, vec2<i32>(0, 0), max(dims - vec2<i32>(1, 1), vec2<i32>(0, 0)));
}

fn face_id_base() -> u32 {
    return u32(max(uniforms.params16.w, 0.0));
}

fn face_id_matches(edge: GpuNprEdge3d, face_id: u32) -> bool {
    let base = face_id_base();
    let face0 = base + edge.face0 + 1u;
    let face1 = base + edge.face1 + 1u;
    return face_id == face0 || (edge.face_count > 1u && face_id == face1);
}

fn edge_texel_has_owned_face(edge: GpuNprEdge3d, texel: vec2<i32>) -> bool {
    let dims = vec2<i32>(textureDimensions(face_id_texture));
    let max_texel = max(dims - vec2<i32>(1, 1), vec2<i32>(0, 0));
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            let sample_texel = clamp(texel + vec2<i32>(dx, dy), vec2<i32>(0, 0), max_texel);
            if (face_id_matches(edge, textureLoad(face_id_texture, sample_texel, 0).x)) {
                return true;
            }
        }
    }
    return false;
}

fn edge_endpoints_near_contact_ground(edge: GpuNprEdge3d) -> bool {
    let threshold = max(uniforms.params13.z, 0.0);
    if (threshold <= 0.0001) {
        return false;
    }
    let ground_y = uniforms.params13.y;
    let world_a = transform_vertex(edge.a);
    let world_b = transform_vertex(edge.b);
    return abs(world_a.y - ground_y) <= threshold && abs(world_b.y - ground_y) <= threshold;
}

fn edge_contact_candidate(edge: GpuNprEdge3d, front0: bool, front1: bool) -> bool {
    if (!edge_endpoints_near_contact_ground(edge)) {
        return false;
    }
    if (edge.face_count == 1u) {
        return front0;
    }
    if (edge.face_count >= 2u) {
        return front0 && front1 && (edge.material_seam > 0u || feature_edge(edge));
    }
    return false;
}

fn edge_visible_run(
    edge: GpuNprEdge3d,
    start: vec2<f32>,
    end: vec2<f32>,
    line_length: f32,
    max_gap_samples: u32,
) -> vec3<f32> {
    let sample_count = u32(clamp(ceil(line_length / 4.0), 7.0, 96.0));
    let step_t = 1.0 / max(f32(sample_count - 1u), 1.0);
    var best_t0 = 0.0;
    var best_t1 = 0.0;
    var best_len = -1.0;
    var run_active = false;
    var run_t0 = 0.0;
    var run_last_t = 0.0;
    var gap_samples = 0u;

    for (var i: u32 = 0u; i < sample_count; i = i + 1u) {
        let t = f32(i) * step_t;
        let sample_screen = mix(start, end, t);
        let texel = screen_to_face_id_texel(sample_screen);
        let visible = edge_texel_has_owned_face(edge, texel);
        if (visible) {
            if (!run_active) {
                run_active = true;
                run_t0 = t;
            }
            run_last_t = t;
            gap_samples = 0u;
        } else if (run_active) {
            gap_samples = gap_samples + 1u;
            if (gap_samples > max_gap_samples) {
                let run_t1 = run_last_t;
                let run_len = run_t1 - run_t0;
                if (run_len > best_len) {
                    best_len = run_len;
                    best_t0 = run_t0;
                    best_t1 = run_t1;
                }
                run_active = false;
                gap_samples = 0u;
            }
        }
    }

    if (run_active) {
        let run_t1 = run_last_t;
        let run_len = run_t1 - run_t0;
        if (run_len > best_len) {
            best_len = run_len;
            best_t0 = run_t0;
            best_t1 = run_t1;
        }
    }

    if (best_len < 0.0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    let pad = step_t * 0.5;
    return vec3<f32>(
        clamp(best_t0 - pad, 0.0, 1.0),
        clamp(best_t1 + pad, 0.0, 1.0),
        1.0,
    );
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= u32(arrayLength(&edges))) {
        return;
    }
    clear_visible_segment(edge_index);

    let edge = edges[edge_index];
    let a = projected_vertices[edge.a];
    let b = projected_vertices[edge.b];
    if (a.ndc_depth.w < 0.5 || b.ndc_depth.w < 0.5) {
        return;
    }
    if (abs(a.ndc_depth.x) > 1.25 || abs(a.ndc_depth.y) > 1.25 || abs(b.ndc_depth.x) > 1.25 || abs(b.ndc_depth.y) > 1.25) {
        return;
    }
    let screen_a = a.screen.xy;
    let screen_b = b.screen.xy;
    let line_length = distance(screen_a, screen_b);
    let early_min_length = max(
        min(
            kind_min_screen_length_px(KIND_BOUNDARY),
            min(
                kind_min_screen_length_px(KIND_SILHOUETTE),
                min(kind_min_screen_length_px(KIND_FEATURE), kind_min_screen_length_px(KIND_CONTACT)),
            ),
        ),
        0.5,
    );
    if (line_length < early_min_length) {
        return;
    }

    let show_boundary = uniforms.params1.z > 0.5;
    let show_silhouette = uniforms.params1.w > 0.5;
    let show_feature = uniforms.params2.x > 0.5;
    let show_contact = uniforms.params2.y > 0.5;
    let show_suggestive = uniforms.params2.z > 0.5;

    var kind: u32 = KIND_NONE;
    let front0 = select(false, triangle_front(edge.face0), edge.face0 < u32(arrayLength(&triangles)));
    let front1 = select(false, triangle_front(edge.face1), edge.face1 < u32(arrayLength(&triangles)));
    let vis0 = edge.face_count >= 1u;
    let vis1 = edge.face_count >= 2u;
    let contact = show_contact && (vis0 || vis1) && edge_contact_candidate(edge, front0, front1);

    if (edge.face_count == 1u && show_boundary && front0) {
        kind = KIND_BOUNDARY;
    } else if (edge.face_count >= 2u) {
        if (show_silhouette && front0 != front1) {
            kind = KIND_SILHOUETTE;
        } else if (show_feature && front0 && front1 && edge.material_seam > 0u && line_length >= edge_feature_min_length_px(edge)) {
            kind = KIND_SEAM;
        } else if (show_feature && front0 && front1 && feature_edge(edge) && line_length >= edge_feature_min_length_px(edge)) {
            kind = KIND_CREASE;
        } else if (
            show_suggestive
            && front0
            && front1
            && line_length >= max(edge_feature_min_length_px(edge) * 1.1, 8.0)
        ) {
            let alignment = min(triangle_view_alignment(edge.face0), triangle_view_alignment(edge.face1));
            let shallow_view = alignment <= 0.18;
            let not_contact_like = !edge_contact_candidate(edge, front0, front1);
            if (shallow_view && not_contact_like) {
                kind = KIND_FEATURE;
            }
        }
    }

    if (kind == KIND_NONE && contact && line_length >= contact_min_length_px()) {
        kind = KIND_CONTACT;
    }

    if (kind == KIND_NONE) {
        return;
    }

    let primary_contour = kind == KIND_SILHOUETTE || kind == KIND_BOUNDARY;
    let max_gap_samples =
        select(0u, select(2u, 3u, kind == KIND_SILHOUETTE), uses_manga_ink() && primary_contour);
    var run = edge_visible_run(edge, screen_a, screen_b, line_length, max_gap_samples);
    if (run.z < 0.5) {
        return;
    }
    let run_coverage = run.y - run.x;
    if (
        uses_confident_manga_ink()
        && primary_contour
        && run_coverage >= select(0.56, 0.48, kind == KIND_SILHOUETTE)
    ) {
        run = vec3<f32>(0.0, 1.0, run.z);
    }
    let endpoint_tolerance =
        select(
            select(0.001, 0.06, primary_contour),
            select(0.001, select(0.34, 0.42, kind == KIND_SILHOUETTE), primary_contour),
            uses_manga_ink(),
        );
    let raw_run_start = mix(screen_a, screen_b, run.x);
    let raw_run_end = mix(screen_a, screen_b, run.y);
    let run_start = select(raw_run_start, screen_a, run.x <= endpoint_tolerance);
    let run_end = select(raw_run_end, screen_b, run.y >= 1.0 - endpoint_tolerance);
    var run_min_length = edge_feature_min_length_px(edge);
    if (kind == KIND_SILHOUETTE) {
        run_min_length = silhouette_min_length_px();
    } else if (kind == KIND_BOUNDARY) {
        run_min_length = kind_min_screen_length_px(KIND_BOUNDARY);
    } else if (kind == KIND_SEAM) {
        run_min_length = max(kind_min_screen_length_px(KIND_SEAM), edge_feature_min_length_px(edge));
    } else if (kind == KIND_CREASE) {
        run_min_length = max(kind_min_screen_length_px(KIND_CREASE), edge_feature_min_length_px(edge));
    } else if (kind == KIND_CONTACT) {
        run_min_length = contact_min_length_px();
    } else if (kind == KIND_FEATURE) {
        run_min_length = max(kind_min_screen_length_px(KIND_FEATURE), edge_feature_min_length_px(edge));
    }
    if (distance(run_start, run_end) < run_min_length) {
        return;
    }

    let depth_start = mix(a.ndc_depth.z, b.ndc_depth.z, run.x);
    let depth_end = mix(a.ndc_depth.z, b.ndc_depth.z, run.y);
    let view_depth_start = mix(a.screen.w, b.screen.w, run.x);
    let view_depth_end = mix(a.screen.w, b.screen.w, run.y);
    let style_depth_start = camera_response_depth01(view_depth_start, depth_start);
    let style_depth_end = camera_response_depth01(view_depth_end, depth_end);
    let style_depth_mid = (style_depth_start + style_depth_end) * 0.5;
    let semantic_importance = edge_semantic_importance(edge, kind, distance(run_start, run_end), style_depth_mid);
    if (should_reject_character_edge(edge, kind, semantic_importance, style_depth_mid)) {
        return;
    }
    let start_vertex = select(0xffffffffu, edge.a, run.x <= endpoint_tolerance);
    let end_vertex = select(0xffffffffu, edge.b, run.y >= 1.0 - endpoint_tolerance);
    visible_segments[edge_index].start = vec4<f32>(run_start, depth_start, 1.0);
    visible_segments[edge_index].end = vec4<f32>(run_end, depth_end, 1.0);
    visible_segments[edge_index].kind_edge = vec4<u32>(kind, edge.edge_id, start_vertex, end_vertex);
    visible_segments[edge_index].metrics =
        vec4<f32>(style_depth_start, style_depth_end, semantic_importance, f32(edge_trait_flags(edge, kind)));
}
