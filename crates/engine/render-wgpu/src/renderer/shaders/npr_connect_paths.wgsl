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
const PATH_FLAG_EMIT: u32 = 1u;

@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(10) var<storage, read> path_links: array<GpuNprPathLink3d>;
@group(0) @binding(14) var<storage, read_write> path_states: array<GpuNprPathState3d>;

fn active_edge_count() -> u32 {
    return min(uniforms.pipeline1.w, min(u32(arrayLength(&path_states)), u32(arrayLength(&visible_segments))));
}

fn hash_u32(value: u32) -> u32 {
    var x = value;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    return (x >> 16u) ^ x;
}

fn valid_edge(index: u32, kind: u32) -> bool {
    if (index == 0xffffffffu || index >= active_edge_count()) {
        return false;
    }
    let seg = visible_segments[index];
    return seg.kind_edge.x == kind && seg.start.w > 0.5 && seg.end.w > 0.5;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= active_edge_count()) {
        return;
    }

    let visible = visible_segments[edge_index];
    if (visible.kind_edge.x == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        path_states[edge_index] = GpuNprPathState3d(0xffffffffu, 0u, KIND_NONE, 0u, 0u, vec3<u32>(0u));
        return;
    }

    let link = path_links[edge_index];
    let kind = visible.kind_edge.x;
    var owner = select(edge_index, link.owner_edge, valid_edge(link.owner_edge, kind));

    if ((link.flags & PATH_FLAG_EMIT) == 0u) {
        owner = edge_index;
    }

    var segment_count = 1u;
    if (valid_edge(link.start_next, kind)) {
        segment_count = segment_count + 1u;
    }
    if (valid_edge(link.end_next, kind)) {
        segment_count = segment_count + 1u;
    }

    let path_id =
        hash_u32(owner ^ (kind * 0x9E3779B1u) ^ (segment_count * 0x85EBCA77u));
    path_states[edge_index] = GpuNprPathState3d(
        owner,
        path_id,
        kind,
        link.flags,
        segment_count,
        vec3<u32>(0u),
    );
}
