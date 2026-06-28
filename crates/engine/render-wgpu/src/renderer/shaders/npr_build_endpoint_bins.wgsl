struct GpuNprVisibleSegment3d {
    start: vec4<f32>,
    end: vec4<f32>,
    kind_edge: vec4<u32>,
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
    ink_color: vec4<f32>,
    seed: vec4<u32>,
    pipeline0: vec4<u32>,
    pipeline1: vec4<u32>,
    material_roles0: vec4<u32>,
}

const KIND_NONE: u32 = 0u;
const ENDPOINT_FLAG_MATCHED_START: u32 = 1u;

@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;
@group(0) @binding(11) var<storage, read_write> endpoint_heads: array<atomic<u32>>;
@group(0) @binding(12) var<storage, read_write> endpoint_entries: array<GpuNprEndpointEntry3d>;

fn active_edge_count() -> u32 {
    return min(uniforms.pipeline1.w, u32(arrayLength(&visible_segments)));
}

fn quantized_anchor_bin(point: vec2<f32>) -> vec2<i32> {
    let quant = max(uniforms.params12.w, 0.5);
    return vec2<i32>(round(point / quant));
}

fn endpoint_bucket_index(kind: u32, bin: vec2<i32>) -> u32 {
    let head_count = max(u32(arrayLength(&endpoint_heads)), 1u);
    let hx = bitcast<u32>(bin.x) * 0x9E3779B1u;
    let hy = bitcast<u32>(bin.y) * 0x85EBCA77u;
    let hk = kind * 0xC2B2AE3Du;
    return (hx ^ hy ^ hk) & (head_count - 1u);
}

fn write_endpoint_entry(
    entry_index: u32,
    edge_index: u32,
    kind: u32,
    point: vec2<f32>,
    matched_start: bool,
    endpoint_vertex: u32,
) {
    let bin = quantized_anchor_bin(point);
    let bucket_index = endpoint_bucket_index(kind, bin);
    let previous_head = atomicExchange(&endpoint_heads[bucket_index], entry_index + 1u);
    endpoint_entries[entry_index] = GpuNprEndpointEntry3d(
        edge_index,
        select(0u, ENDPOINT_FLAG_MATCHED_START, matched_start),
        previous_head,
        kind,
        bin,
        endpoint_vertex,
        0u,
    );
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let item = id.x;
    let entry_count = u32(arrayLength(&endpoint_entries));
    let edge_count = active_edge_count();

    if (item >= edge_count) {
        return;
    }

    let visible = visible_segments[item];
    let kind = visible.kind_edge.x;
    if (kind == KIND_NONE || visible.start.w <= 0.5 || visible.end.w <= 0.5) {
        return;
    }

    let base_entry = item * 2u;
    if (base_entry + 1u >= entry_count) {
        return;
    }

    if (visible.kind_edge.z != 0xffffffffu) {
        write_endpoint_entry(base_entry, item, kind, visible.start.xy, true, visible.kind_edge.z);
    }
    if (visible.kind_edge.w != 0xffffffffu) {
        write_endpoint_entry(base_entry + 1u, item, kind, visible.end.xy, false, visible.kind_edge.w);
    }
}
