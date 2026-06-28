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

struct GpuNprPathState3d {
    owner_segment: u32,
    path_id: u32,
    kind: u32,
    flags: u32,
    segment_count: u32,
    _pad0: vec3<u32>,
}

const KIND_NONE: u32 = 0u;

@group(0) @binding(5) var<storage, read> visible_segments: array<GpuNprVisibleSegment3d>;
@group(0) @binding(10) var<storage, read> path_links: array<GpuNprPathLink3d>;
@group(0) @binding(14) var<storage, read_write> path_states: array<GpuNprPathState3d>;

fn hash_u32(value: u32) -> u32 {
    var x = value;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    x = ((x >> 16u) ^ x) * 0x45d9f3bu;
    return (x >> 16u) ^ x;
}

fn valid_state(index: u32, kind: u32) -> bool {
    if (index == 0xffffffffu || index >= u32(arrayLength(&path_states)) || index >= u32(arrayLength(&visible_segments))) {
        return false;
    }
    let state = path_states[index];
    let seg = visible_segments[index];
    return state.kind == kind && seg.kind_edge.x == kind && seg.start.w > 0.5 && seg.end.w > 0.5;
}

fn maybe_relax_owner(current_owner: u32, neighbor_index: u32, kind: u32) -> u32 {
    if (!valid_state(neighbor_index, kind)) {
        return current_owner;
    }
    return min(current_owner, path_states[neighbor_index].owner_segment);
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let edge_index = id.x;
    if (edge_index >= u32(arrayLength(&path_states)) || edge_index >= u32(arrayLength(&visible_segments))) {
        return;
    }

    let state = path_states[edge_index];
    let seg = visible_segments[edge_index];
    if (state.kind == KIND_NONE || seg.kind_edge.x == KIND_NONE || seg.start.w <= 0.5 || seg.end.w <= 0.5) {
        return;
    }

    let link = path_links[edge_index];
    var owner = state.owner_segment;
    owner = maybe_relax_owner(owner, link.start_next, state.kind);
    owner = maybe_relax_owner(owner, link.end_next, state.kind);

    let segment_count =
        1u
        + select(0u, 1u, valid_state(link.start_next, state.kind))
        + select(0u, 1u, valid_state(link.end_next, state.kind));
    let edge_id = seg.kind_edge.y;
    let path_id = hash_u32(owner ^ (state.kind * 0x9E3779B1u) ^ (edge_id * 0x85EBCA77u));
    path_states[edge_index] = GpuNprPathState3d(
        owner,
        path_id,
        state.kind,
        state.flags,
        max(state.segment_count, segment_count),
        vec3<u32>(0u),
    );
}
