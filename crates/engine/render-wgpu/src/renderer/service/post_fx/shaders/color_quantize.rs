pub(crate) const COLOR_QUANTIZE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct ColorQuantizeUniform {
    resolution: vec2<f32>,
    palette_size: f32,
    dither_strength: f32,
    dither_scale: f32,
    layered_dither: f32,
    opacity: f32,
    luma_preserve: f32,
    highlight_bias: f32,
    shadow_bias: f32,
    contrast: f32,
    saturation: f32,
    gamma: f32,
    seed: f32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: ColorQuantizeUniform;
@group(2) @binding(0) var palette_texture: texture_2d<f32>;
@group(2) @binding(1) var palette_sampler: sampler;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn bayer4_value(x: u32, y: u32) -> u32 {
    var v = 0u;
    if (y == 0u) {
        if (x == 0u) { v = 0u; }
        if (x == 1u) { v = 8u; }
        if (x == 2u) { v = 2u; }
        if (x == 3u) { v = 10u; }
    } else if (y == 1u) {
        if (x == 0u) { v = 12u; }
        if (x == 1u) { v = 4u; }
        if (x == 2u) { v = 14u; }
        if (x == 3u) { v = 6u; }
    } else if (y == 2u) {
        if (x == 0u) { v = 3u; }
        if (x == 1u) { v = 11u; }
        if (x == 2u) { v = 1u; }
        if (x == 3u) { v = 9u; }
    } else {
        if (x == 0u) { v = 15u; }
        if (x == 1u) { v = 7u; }
        if (x == 2u) { v = 13u; }
        if (x == 3u) { v = 5u; }
    }
    return v;
}

fn bayer8(pixel: vec2<u32>) -> f32 {
    let x = pixel.x & 7u;
    let y = pixel.y & 7u;
    let base = bayer4_value(x & 3u, y & 3u) * 4u;
    let quadrant = ((x >> 2u) & 1u) + (((y >> 2u) & 1u) * 2u);
    let offset = array<u32, 4>(0u, 2u, 3u, 1u)[quadrant];
    return ((f32(base + offset) + 0.5) / 64.0) - 0.5;
}

fn hash_u32(value: u32) -> u32 {
    var x = value;
    x = ((x >> 16u) ^ x) * 73244475u;
    x = ((x >> 16u) ^ x) * 73244475u;
    x = (x >> 16u) ^ x;
    return x;
}

fn hash21(pixel: vec2<u32>) -> f32 {
    let mixed = pixel.x * 1973u + pixel.y * 9277u + u32(uniforms.seed) * 26699u + 911u;
    return f32(hash_u32(mixed) & 65535u) / 65535.0;
}

fn sample_palette(value: f32, palette_size: f32) -> f32 {
    let levels = max(2.0, palette_size);
    let index = floor(clamp(value, 0.0, 1.0) * (levels - 1.0) + 0.5);
    let u = (index + 0.5) / levels;
    return textureSampleLevel(palette_texture, palette_sampler, vec2<f32>(u, 0.5), 0.0).r;
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(source_texture, source_sampler, input.uv);
    let palette_size = clamp(uniforms.palette_size, 2.0, 256.0);
    let gamma = clamp(uniforms.gamma, 1.0, 3.0);
    let pixel = max(input.uv * uniforms.resolution, vec2<f32>(0.0));
    let dither_scale = max(uniforms.dither_scale, 1.0);
    let dither_pixel = floor(pixel / dither_scale);
    let px = vec2<u32>(dither_pixel) + vec2<u32>(u32(uniforms.seed));
    let base_luma = luminance(base.rgb);
    let highlight_bias = clamp(uniforms.highlight_bias, 0.0, 1.0);
    let shadow_bias = clamp(uniforms.shadow_bias, 0.0, 1.0);
    let contrast = clamp(uniforms.contrast, 0.25, 2.0);
    let saturation = clamp(uniforms.saturation, 0.0, 2.0);
    let ordered_primary = bayer8(px);
    let ordered_secondary = bayer8(px * vec2<u32>(3u, 5u) + vec2<u32>(17u, 29u));
    let grain = (hash21(px) - 0.5) * 0.35;
    let layered = mix(ordered_primary, ordered_secondary + grain, clamp(uniforms.layered_dither, 0.0, 1.0));
    let shadow_weight = mix(1.0, 1.0 - smoothstep(0.16, 0.84, base_luma), shadow_bias);
    let dither = layered * shadow_weight * clamp(uniforms.dither_strength, 0.0, 1.0) / max(palette_size - 1.0, 1.0);

    let contrasted = clamp((base.rgb - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    let gray = vec3<f32>(luminance(contrasted));
    let graded = clamp(mix(gray, contrasted, saturation), vec3<f32>(0.0), vec3<f32>(1.0));
    let encoded = pow(graded, vec3<f32>(1.0 / gamma));
    let biased = clamp(encoded + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0));
    let rounded = vec3<f32>(
        sample_palette(biased.r, palette_size),
        sample_palette(biased.g, palette_size),
        sample_palette(biased.b, palette_size),
    );
    let ceiling = vec3<f32>(
        sample_palette(ceil(biased.r * (palette_size - 1.0)) / (palette_size - 1.0), palette_size),
        sample_palette(ceil(biased.g * (palette_size - 1.0)) / (palette_size - 1.0), palette_size),
        sample_palette(ceil(biased.b * (palette_size - 1.0)) / (palette_size - 1.0), palette_size),
    );
    let highlight_weight = smoothstep(0.30, 0.92, base_luma) * highlight_bias;
    let quantized_encoded = mix(rounded, ceiling, highlight_weight);
    var quantized = pow(quantized_encoded, vec3<f32>(gamma));

    let original_luma = luminance(graded);
    let quantized_luma = max(luminance(quantized), 0.001);
    let luma_matched = clamp(quantized * (original_luma / quantized_luma), vec3<f32>(0.0), vec3<f32>(1.0));
    quantized = mix(quantized, luma_matched, clamp(uniforms.luma_preserve, 0.0, 1.0));

    return vec4<f32>(mix(base.rgb, quantized, clamp(uniforms.opacity, 0.0, 1.0)), base.a);
}
"#;
