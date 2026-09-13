@group(0) @binding(11) var<storage, read_write> endpoint_heads: array<atomic<u32>>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= u32(arrayLength(&endpoint_heads))) {
        return;
    }
    atomicStore(&endpoint_heads[index], 0u);
}
