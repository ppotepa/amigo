@group(0) @binding(9) var<storage, read_write> indirect_args: array<atomic<u32>>;

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x != 0u || u32(arrayLength(&indirect_args)) < 6u) {
        return;
    }

    let stroke_capacity = atomicLoad(&indirect_args[5]);
    let instance_count = atomicLoad(&indirect_args[1]);
    if (instance_count > stroke_capacity) {
        atomicStore(&indirect_args[1], stroke_capacity);
    }
}
