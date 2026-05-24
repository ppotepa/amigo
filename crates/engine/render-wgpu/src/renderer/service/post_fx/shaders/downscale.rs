pub(crate) const DOWNSCALE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct DownscaleUniform {
    resolution: vec2<f32>,
    factor: f32,
    opacity: f32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: DownscaleUniform;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let original = textureSample(source_texture, source_sampler, input.uv);
    let factor = max(uniforms.factor, 1.0);
    let sample_pixel = floor(input.uv * uniforms.resolution / factor) * factor + vec2<f32>(0.5) * factor;
    let sample_uv = clamp(sample_pixel / uniforms.resolution, vec2<f32>(0.0), vec2<f32>(1.0));
    let downscaled = textureSample(source_texture, source_sampler, sample_uv);
    return mix(original, downscaled, clamp(uniforms.opacity, 0.0, 1.0));
}
"#;
