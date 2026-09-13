struct MaskTerm {
    // kind, invert, seed low, seed high
    flags: vec4<u32>,
    values: vec4<f32>,
    direction: vec4<f32>,
};
struct CoverageProgram {
    header: vec4<u32>,
    terms: array<MaskTerm, 128>,
};
@group(0) @binding(0) var<uniform> coverage_program: CoverageProgram;

fn mask_hash(seed: vec2<u32>, cell: vec3<i32>) -> f32 {
    var hash = seed.x ^ ((seed.y << 16u) | (seed.y >> 16u)) ^ 0x9e3779b9u;
    for (var axis = 0u; axis < 3u; axis += 1u) {
        hash ^= bitcast<u32>(cell[axis]);
        hash = (hash ^ (hash >> 16u)) * 0x7feb352du;
        hash = (hash ^ (hash >> 15u)) * 0x846ca68bu;
        hash ^= hash >> 16u;
    }
    return f32(hash >> 8u) / 16777215.0;
}
fn mask_noise(position: vec3<f32>, seed: vec2<u32>) -> f32 {
    let point = position * 8.0;
    let cell = floor(point);
    let fraction = point-cell;
    let easing = fraction*fraction*(vec3<f32>(3.0)-fraction*2.0);
    var value = 0.0;
    for (var z = 0; z < 2; z += 1) {
        for (var y = 0; y < 2; y += 1) {
            for (var x = 0; x < 2; x += 1) {
                let weight = select(1.0-easing.x,easing.x,x==1)
                    * select(1.0-easing.y,easing.y,y==1)
                    * select(1.0-easing.z,easing.z,z==1);
                value += mask_hash(seed,vec3<i32>(cell)+vec3<i32>(x,y,z))*weight;
            }
        }
    }
    return clamp(value,0.0,1.0);
}
fn surface_coverage(projected_position: vec4<f32>, projected_normal: vec4<f32>, depth: f32) -> f32 {
    // Geometry positions are already screen-space. Reconstruct perspective
    // interpolation of local surface attributes using the declared depth.
    let q = max(1.0-depth,0.00000001);
    let position = projected_position/q;
    let normal_tone = projected_normal/q;
    var coverage = 1.0;
    for (var i = 0u; i < coverage_program.header.x; i += 1u) {
        let term = coverage_program.terms[i];
        var value = 1.0;
        switch term.flags.x {
            case 1u: { value = select(0.0,1.0,normal_tone.w >= term.values.x && normal_tone.w <= term.values.y); }
            case 2u: { value = select(0.0,1.0,position.w >= term.values.x && position.w <= term.values.y); }
            case 3u: {
                let normal = normal_tone.xyz / max(length(normal_tone.xyz),0.000001);
                let facing = dot(normal,term.direction.xyz);
                if term.values.x >= 1.0 { value = select(0.0,1.0,facing >= 0.999999); }
                else { value = clamp((facing-term.values.x)/(1.0-term.values.x),0.0,1.0); }
            }
            case 4u: { value = 1.0-term.values.x+term.values.x*mask_noise(position.xyz,term.flags.zw); }
            default: {}
        }
        coverage *= select(value,1.0-value,term.flags.y != 0u);
    }
    return coverage;
}
