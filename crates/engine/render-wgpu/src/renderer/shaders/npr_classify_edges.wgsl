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

@group(0) @binding(0) var<storage, read> vertices: array<GpuNprVertex3d>;
@group(0) @binding(1) var<storage, read> triangles: array<GpuNprTriangle3d>;
@group(0) @binding(2) var<storage, read> edges: array<GpuNprEdge3d>;
@group(0) @binding(3) var<storage, read_write> projected_vertices: array<GpuNprProjectedVertex3d>;
@group(0) @binding(4) var face_id_texture: texture_2d<u32>;
@group(0) @binding(5) var<storage, read_write> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;

fn clear_visible_segment(edge_index: u32) {
    visible_segments[edge_index].start = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    visible_segments[edge_index].end = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    visible_segments[edge_index].kind_edge = vec4<u32>(0u, 0u, 0u, 0u);
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

fn feature_min_length_px() -> f32 {
    return max(uniforms.params0.w * max(uniforms.params16.x, 0.1) * 1.20, uniforms.params0.w);
}

fn silhouette_min_length_px() -> f32 {
    return max(uniforms.params0.w * max(uniforms.params16.z, 0.1), uniforms.params0.w);
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

fn edge_visible_run(edge: GpuNprEdge3d, start: vec2<f32>, end: vec2<f32>, line_length: f32) -> vec3<f32> {
    let sample_count = u32(clamp(ceil(line_length / 4.0), 7.0, 96.0));
    let step_t = 1.0 / max(f32(sample_count - 1u), 1.0);
    var best_t0 = 0.0;
    var best_t1 = 0.0;
    var best_len = -1.0;
    var run_active = false;
    var run_t0 = 0.0;
    var run_last_t = 0.0;

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
        } else if (run_active) {
            let run_t1 = run_last_t;
            let run_len = run_t1 - run_t0;
            if (run_len > best_len) {
                best_len = run_len;
                best_t0 = run_t0;
                best_t1 = run_t1;
            }
            run_active = false;
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
    if (line_length < uniforms.params0.w) {
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
        } else if (show_feature && front0 && front1 && edge.material_seam > 0u && line_length >= feature_min_length_px()) {
            kind = KIND_SEAM;
        } else if (show_feature && front0 && front1 && feature_edge(edge) && line_length >= feature_min_length_px()) {
            kind = KIND_CREASE;
        } else if (
            show_suggestive
            && front0
            && front1
            && line_length >= max(feature_min_length_px() * 1.1, 8.0)
        ) {
            let alignment = min(triangle_view_alignment(edge.face0), triangle_view_alignment(edge.face1));
            let shallow_view = alignment <= 0.18;
            let not_contact_like = !edge_contact_candidate(edge, front0, front1);
            if (shallow_view && not_contact_like) {
                kind = KIND_FEATURE;
            }
        }
    }

    if (kind == KIND_NONE && contact && line_length >= max(feature_min_length_px(), 5.0)) {
        kind = KIND_CONTACT;
    }

    if (kind == KIND_NONE) {
        return;
    }

    var run = edge_visible_run(edge, screen_a, screen_b, line_length);
    if (run.z < 0.5) {
        return;
    }
    let run_start = mix(screen_a, screen_b, run.x);
    let run_end = mix(screen_a, screen_b, run.y);
    let run_min_length = select(
        feature_min_length_px(),
        silhouette_min_length_px(),
        kind == KIND_SILHOUETTE,
    );
    if (distance(run_start, run_end) < run_min_length) {
        return;
    }

    let depth_start = mix(a.ndc_depth.z, b.ndc_depth.z, run.x);
    let depth_end = mix(a.ndc_depth.z, b.ndc_depth.z, run.y);
    visible_segments[edge_index].start = vec4<f32>(run_start, depth_start, 1.0);
    visible_segments[edge_index].end = vec4<f32>(run_end, depth_end, 1.0);
    visible_segments[edge_index].kind_edge = vec4<u32>(kind, edge.edge_id, edge.a, edge.b);
}
