pub(crate) const DIRTY_BLOOM_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct DirtyBloomUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    threshold: f32,
    strength: f32,
    small_radius_px: f32,
    medium_radius_px: f32,
    large_radius_px: f32,
    dirty_noise: f32,
    halation_strength: f32,
    reflection_smear_x_px: f32,
    reflection_smear_y_px: f32,
    seed: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: DirtyBloomUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn bright_sample(uv: vec2<f32>) -> vec3<f32> {
    let color = textureSample(source_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
    let lift = smoothstep(uniforms.threshold, 1.0, luminance(color));
    return color * lift;
}

fn blur_ring(uv: vec2<f32>, radius_px: f32) -> vec3<f32> {
    let texel = vec2<f32>(1.0) / max(uniforms.resolution, vec2<f32>(1.0));
    let r = radius_px * texel;
    var color = bright_sample(uv) * 0.2;
    color += bright_sample(uv + vec2<f32>( r.x,  0.0)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x,  0.0)) * 0.1;
    color += bright_sample(uv + vec2<f32>( 0.0,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( 0.0, -r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( r.x,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( r.x, -r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x, -r.y)) * 0.1;
    return color;
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(source_tex, source_sampler, input.uv);
    let bloom_small = blur_ring(input.uv, uniforms.small_radius_px) * 0.45;
    let bloom_medium = blur_ring(input.uv, uniforms.medium_radius_px) * 0.65;
    let bloom_large = blur_ring(input.uv, uniforms.large_radius_px) * 0.35;
    let texel = vec2<f32>(1.0) / max(uniforms.resolution, vec2<f32>(1.0));
    let smear = (
        bright_sample(input.uv + vec2<f32>(uniforms.reflection_smear_x_px, uniforms.reflection_smear_y_px) * texel) +
        bright_sample(input.uv + vec2<f32>(-uniforms.reflection_smear_x_px, uniforms.reflection_smear_y_px) * texel) +
        bright_sample(input.uv + vec2<f32>(0.0, uniforms.reflection_smear_y_px * 1.6) * texel)
    ) * 0.18;
    let dirty = 1.0 - uniforms.dirty_noise + hash12(floor(input.uv * uniforms.resolution * 0.18) + vec2<f32>(uniforms.seed, floor(uniforms.time_seconds * 12.0))) * uniforms.dirty_noise;
    let hot = blur_ring(input.uv, 6.0) * smoothstep(0.78, 1.0, luminance(bright_sample(input.uv)));
    let halation = hot * vec3<f32>(1.0, 0.25, 0.35) * uniforms.halation_strength;
    let rgb = clamp(base.rgb + ((bloom_small + bloom_medium + bloom_large + smear) * dirty * uniforms.strength) + halation, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, base.a);
}
"#;
