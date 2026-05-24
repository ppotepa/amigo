pub(crate) const CRT_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CrtUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    scanline_opacity: f32,
    scanline_frequency_px: f32,
    rgb_split_px: f32,
    curvature: f32,
    vignette: f32,
    phosphor_mask: f32,
    brightness_compensation: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: CrtUniform;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let centered = input.uv * 2.0 - vec2<f32>(1.0);
    let curve = dot(centered, centered) * uniforms.curvature;
    let uv = input.uv + centered * curve;
    let inside = step(0.0, uv.x) * step(uv.x, 1.0) * step(0.0, uv.y) * step(uv.y, 1.0);
    let split = vec2<f32>(uniforms.rgb_split_px / max(uniforms.resolution.x, 1.0), 0.0);
    let r = textureSample(source_tex, source_sampler, clamp(uv + split, vec2<f32>(0.0), vec2<f32>(1.0))).r;
    let g = textureSample(source_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).g;
    let b = textureSample(source_tex, source_sampler, clamp(uv - split, vec2<f32>(0.0), vec2<f32>(1.0))).b;
    var rgb = vec3<f32>(r, g, b);
    let scanline = 1.0 - uniforms.scanline_opacity * (0.5 + 0.5 * sin(uv.y * uniforms.resolution.y * 6.2831853 / max(uniforms.scanline_frequency_px, 0.5)));
    let mask = 1.0 - uniforms.phosphor_mask * (0.5 + 0.5 * sin(uv.x * uniforms.resolution.x * 2.0943951));
    let vignette_distance = distance(input.uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.35, 0.82, vignette_distance) * uniforms.vignette;
    rgb = clamp(rgb * scanline * mask * vignette * uniforms.brightness_compensation * inside, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, 1.0);
}
"#;
