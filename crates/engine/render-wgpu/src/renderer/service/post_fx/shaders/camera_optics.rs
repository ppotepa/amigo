pub(crate) const CAMERA_OPTICS_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CameraOpticsUniform {
    focal_length_mm: f32,
    aberration_px: f32,
    distortion: f32,
    vignette: f32,
    edge_softness_px: f32,
    glare_strength: f32,
    lens_bloom: f32,
    flare_ghosts: f32,
    anamorphic_squeeze: f32,
    coma: f32,
    dirt: f32,
    halation_bias: f32,
    opacity: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var normal_tex: texture_2d<f32>;
@group(0) @binding(2) var wetness_tex: texture_2d<f32>;
@group(0) @binding(3) var highlight_tex: texture_2d<f32>;
@group(0) @binding(4) var emissive_tex: texture_2d<f32>;
@group(0) @binding(5) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: CameraOpticsUniform;

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
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
    let normal_source = textureSample(normal_tex, source_sampler, input.uv).xy * 2.0 - vec2<f32>(1.0);
    let wetness_source = textureSample(wetness_tex, source_sampler, input.uv).rgb;
    let highlight_source = textureSample(highlight_tex, source_sampler, input.uv).rgb;
    let emissive_source = textureSample(emissive_tex, source_sampler, input.uv).rgb;
    let dims = vec2<f32>(textureDimensions(source_tex, 0));
    let centered = input.uv - vec2<f32>(0.5, 0.5);
    let radius = length(centered);
    let squeeze = max(uniforms.anamorphic_squeeze, 1.0);
    let stretched = vec2<f32>(centered.x / squeeze, centered.y * squeeze);
    let focal_norm = clamp((uniforms.focal_length_mm - 24.0) / 61.0, 0.0, 1.0);
    let wide_factor = 1.0 - focal_norm;
    let tele_factor = focal_norm;
    let distortion_amount = uniforms.distortion * (0.65 + wide_factor * 0.75);
    let edge_softness_amount = uniforms.edge_softness_px * (0.65 + tele_factor * 0.45);
    let wetness_amount = clamp(dot(wetness_source, vec3<f32>(0.18, 0.48, 0.34)) * 2.2, 0.0, 1.0);
    let normal_warp = normal_source * (0.0015 + wetness_amount * 0.0055) * (1.0 + uniforms.distortion * 8.0);
    let distorted_uv = clamp(
        input.uv + stretched * distortion_amount * radius * radius + normal_warp,
        vec2<f32>(0.001, 0.001),
        vec2<f32>(0.999, 0.999)
    );
    let dir = select(vec2<f32>(0.0, 0.0), normalize(centered), radius > 0.0001);
    let px = vec2<f32>(
        uniforms.aberration_px / max(dims.x, 1.0),
        uniforms.aberration_px / max(dims.y, 1.0)
    );

    let r = textureSample(source_tex, source_sampler, distorted_uv + dir * px).r;
    let g = textureSample(source_tex, source_sampler, distorted_uv).g;
    let b = textureSample(source_tex, source_sampler, distorted_uv - dir * px).b;
    var optics_rgb = vec3<f32>(r, g, b);

    let vignette_mask = 1.0 - smoothstep(0.20, 0.85, radius) * uniforms.vignette * 0.5;
    optics_rgb *= vignette_mask;

    let edge_mask = smoothstep(0.45, 0.95, radius);
    let softness_uv = clamp(
        distorted_uv - stretched * (edge_softness_amount / max(dims.x, dims.y)) * (0.02 + uniforms.coma * 0.015),
        vec2<f32>(0.001, 0.001),
        vec2<f32>(0.999, 0.999)
    );
    let soft_rgb = textureSample(source_tex, source_sampler, softness_uv).rgb;
    optics_rgb = mix(optics_rgb, soft_rgb, edge_mask * clamp(edge_softness_amount / 16.0 + wetness_amount * 0.16, 0.0, 1.0));

    let luma = luminance(optics_rgb);
    let dirt_cell = floor(input.uv * vec2<f32>(96.0, 54.0));
    let dirt_noise = hash12(dirt_cell);
    let dirt_mask = smoothstep(0.58, 1.0, dirt_noise) * smoothstep(0.15, 0.9, radius);
    let dirt_veil = dirt_mask * uniforms.dirt * luma * 0.12;

    let streak = exp(-abs(centered.y) * 22.0 * squeeze) * smoothstep(0.18, 0.92, abs(centered.x));
    let source_energy = max(luminance(highlight_source), luminance(emissive_source));
    let flare = max(luma, source_energy) * (uniforms.glare_strength + uniforms.lens_bloom * 0.45) * (0.03 + streak * (0.12 + uniforms.flare_ghosts * 0.18));
    let ghost_uv_1 = clamp(vec2<f32>(1.0, 1.0) - distorted_uv, vec2<f32>(0.001), vec2<f32>(0.999));
    let ghost_uv_2 = clamp(vec2<f32>(0.5, 0.5) + (vec2<f32>(0.5, 0.5) - centered * 1.35), vec2<f32>(0.001), vec2<f32>(0.999));
    let ghost_1 = max(textureSample(source_tex, source_sampler, ghost_uv_1).rgb, textureSample(highlight_tex, source_sampler, ghost_uv_1).rgb);
    let ghost_2 = max(textureSample(source_tex, source_sampler, ghost_uv_2).rgb, textureSample(emissive_tex, source_sampler, ghost_uv_2).rgb);
    let ghost_mask_1 = max(luminance(ghost_1) - 0.62, 0.0);
    let ghost_mask_2 = max(luminance(ghost_2) - 0.70, 0.0);
    let ghost_rgb = ghost_1 * ghost_mask_1 * uniforms.flare_ghosts * 0.12
        + ghost_2 * ghost_mask_2 * uniforms.flare_ghosts * 0.07;

    let halo_uv = clamp(
        distorted_uv - dir * px * (1.2 + uniforms.halation_bias * 1.8),
        vec2<f32>(0.001, 0.001),
        vec2<f32>(0.999, 0.999)
    );
    let halo_rgb = max(textureSample(source_tex, source_sampler, halo_uv).rgb, textureSample(highlight_tex, source_sampler, halo_uv).rgb);
    let halation = vec3<f32>(1.0, 0.24, 0.12)
        * max(luminance(halo_rgb) - 0.55, 0.0)
        * uniforms.halation_bias
        * 0.16;

    optics_rgb += vec3<f32>(flare + dirt_veil) + halation + ghost_rgb + highlight_source * uniforms.lens_bloom * (0.08 + wetness_amount * 0.10) + emissive_source * uniforms.glare_strength * (0.05 + wetness_amount * 0.06);
    optics_rgb = mix(optics_rgb, soft_rgb, uniforms.lens_bloom * 0.12);

    let rgb = mix(base.rgb, clamp(optics_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), uniforms.opacity);
    return vec4<f32>(rgb, base.a);
}
"#;
