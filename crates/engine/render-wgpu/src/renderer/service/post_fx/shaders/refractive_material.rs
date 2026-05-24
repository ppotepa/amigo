pub(crate) const REFRACTIVE_MATERIAL_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct RefractiveMaterialUniform {
    resolution: vec2<f32>,
    transmission: f32,
    refraction_px: f32,
    distortion: f32,
    dispersion: f32,
    roughness: f32,
    edge_boost: f32,
    opacity: f32,
    highlight: f32,
}

@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var mask_tex: texture_2d<f32>;
@group(0) @binding(2) var spare_tex: texture_2d<f32>;
@group(0) @binding(3) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: RefractiveMaterialUniform;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

fn mask_at(uv: vec2<f32>) -> f32 {
    return textureSample(mask_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).a;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(scene_tex, source_sampler, input.uv);
    let dims = max(uniforms.resolution, vec2<f32>(1.0, 1.0));
    let px = vec2<f32>(1.0 / dims.x, 1.0 / dims.y);
    let mask = clamp(mask_at(input.uv) * uniforms.opacity, 0.0, 1.0);

    let gx = mask_at(input.uv + vec2<f32>(px.x, 0.0)) - mask_at(input.uv - vec2<f32>(px.x, 0.0));
    let gy = mask_at(input.uv + vec2<f32>(0.0, px.y)) - mask_at(input.uv - vec2<f32>(0.0, px.y));
    let grad = vec2<f32>(gx, gy);
    let edge = clamp(length(grad) * 2.5, 0.0, 1.0);

    let wave = vec2<f32>(
        sin((input.uv.y + mask * 0.17) * 92.0),
        cos((input.uv.x - mask * 0.11) * 74.0)
    ) * uniforms.distortion;
    let direction = grad + wave * 0.32;
    let offset = direction * uniforms.refraction_px * px * mask;
    let rough_uv = wave * uniforms.roughness * px * 1.5 * mask;
    let uv = clamp(input.uv + offset + rough_uv, vec2<f32>(0.001), vec2<f32>(0.999));

    let dispersion = uniforms.dispersion * uniforms.refraction_px * px * mask;
    let r = textureSample(scene_tex, source_sampler, clamp(uv + vec2<f32>(dispersion.x, 0.0), vec2<f32>(0.001), vec2<f32>(0.999))).r;
    let g = textureSample(scene_tex, source_sampler, uv).g;
    let b = textureSample(scene_tex, source_sampler, clamp(uv - vec2<f32>(dispersion.x, 0.0), vec2<f32>(0.001), vec2<f32>(0.999))).b;
    let refracted = vec3<f32>(r, g, b);

    let transmission = clamp(uniforms.transmission, 0.0, 1.0);
    let transmitted = mix(base.rgb, refracted, transmission);
    let base_luma = dot(base.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let dark_boost = mix(1.25, 0.45, clamp(base_luma * 1.6, 0.0, 1.0));
    let interior = smoothstep(0.02, 0.85, mask) * (1.0 - edge * 0.45);
    let glass_lift = vec3<f32>(0.62, 0.82, 1.0)
        * interior
        * clamp(uniforms.highlight, 0.0, 2.0)
        * dark_boost
        * 0.24;
    let edge_light = vec3<f32>(0.82, 0.94, 1.0) * edge * uniforms.edge_boost * 0.22;
    let material_rgb = clamp(transmitted + glass_lift + edge_light, vec3<f32>(0.0), vec3<f32>(1.0));
    let rgb = mix(base.rgb, material_rgb, mask);
    return vec4<f32>(rgb, base.a);
}
"#;
