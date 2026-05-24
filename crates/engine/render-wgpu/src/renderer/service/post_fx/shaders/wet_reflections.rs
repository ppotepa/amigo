pub(crate) const WET_REFLECTIONS_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct WetReflectionsUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    mask_invert: f32,
    blur_px: f32,
    distortion_px: f32,
    shimmer_strength: f32,
    ripple_strength: f32,
    wet_darken: f32,
    specular_boost: f32,
    edge_power: f32,
    light_reflection_strength: f32,
    foreground_strength: f32,
    background_strength: f32,
    horizon_y: f32,
    noise_scale: f32,
    noise_speed: f32,
    ripple_speed: f32,
    _pad0: vec2<f32>,
}

@group(0) @binding(0) var world_tex: texture_2d<f32>;
@group(0) @binding(1) var mask_tex: texture_2d<f32>;
@group(0) @binding(2) var edge_tex: texture_2d<f32>;
@group(0) @binding(3) var reflection_color_tex: texture_2d<f32>;
@group(0) @binding(4) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: WetReflectionsUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + vec3<f32>(33.33));
    return fract((p3.x + p3.y) * p3.z);
}

fn hash_noise(uv: vec2<f32>, time: f32) -> vec2<f32> {
    let n1 = fract(sin(dot(uv + vec2<f32>(time, 0.0), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let n2 = fract(sin(dot(uv + vec2<f32>(0.0, time), vec2<f32>(39.3468, 11.135))) * 24634.6345);
    return vec2<f32>(n1, n2);
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
    let base = textureSample(world_tex, source_sampler, input.uv);
    let mask_sample = textureSample(mask_tex, source_sampler, input.uv);
    let edge_sample = textureSample(edge_tex, source_sampler, input.uv);
    let reflection_color = textureSample(reflection_color_tex, source_sampler, input.uv);

    var coverage = luminance(mask_sample.rgb);
    if (uniforms.mask_invert > 0.5) {
        coverage = 1.0 - coverage;
    }
    coverage = clamp(coverage * mask_sample.a, 0.0, 1.0);

    let edge = pow(clamp(luminance(edge_sample.rgb), 0.0, 1.0), uniforms.edge_power);
    let perspective = mix(uniforms.background_strength, uniforms.foreground_strength, smoothstep(uniforms.horizon_y, 1.0, input.uv.y));
    coverage = clamp(pow(coverage, 0.45) * perspective, 0.0, 1.0);

    let noise = hash_noise(input.uv * uniforms.noise_scale, uniforms.time_seconds * uniforms.noise_speed);
    let ripple = sin((input.uv.y + uniforms.time_seconds * uniforms.ripple_speed) * 40.0) * uniforms.ripple_strength;
    let offset = (noise - vec2<f32>(0.5, 0.5)) * uniforms.distortion_px / uniforms.resolution;
    let reflection_uv = vec2<f32>(
        clamp(input.uv.x + offset.x * coverage, 0.0, 1.0),
        clamp(1.0 - input.uv.y + offset.y * coverage + ripple * coverage, 0.0, 1.0),
    );

    var blurred = textureSample(world_tex, source_sampler, reflection_uv);
    blurred += textureSample(world_tex, source_sampler, reflection_uv + vec2<f32>(1.0, 0.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv - vec2<f32>(1.0, 0.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv + vec2<f32>(0.0, 1.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv - vec2<f32>(0.0, 1.0) / uniforms.resolution * uniforms.blur_px);
    blurred *= 0.2;

    let light_source = mix(blurred.rgb, reflection_color.rgb, reflection_color.a);
    let light_strength = luminance(light_source);
    let specular = coverage * edge * light_strength * uniforms.specular_boost;
    let wet_mix = clamp(coverage * (0.92 + edge * 0.82), 0.0, 1.0);

    var final_rgb = mix(base.rgb, blurred.rgb, wet_mix);
    final_rgb += light_source * coverage * edge * uniforms.light_reflection_strength;
    final_rgb += vec3<f32>(specular * (1.0 + noise.x * uniforms.shimmer_strength * 2.0));
    let wet_brighten = vec3<f32>(0.06, 0.07, 0.08) + light_source * 0.12;
    final_rgb = mix(final_rgb, final_rgb * (1.0 - uniforms.wet_darken) + wet_brighten, coverage);

    return vec4<f32>(final_rgb, 1.0);
}
"#;
