pub(crate) const SCAN_OUTPUT_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct ScanOutputUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    iso: f32,
    grain_chroma: f32,
    grain_padding0: f32,
    flicker: f32,
    vignette: f32,
    print_fade: f32,
    dust: f32,
    scratches: f32,
    gate_weave: f32,
    scan_softness: f32,
    opacity: f32,
    seed: f32,
    grain_luma: f32,
    shadow_grain: f32,
    midtone_grain: f32,
    highlight_grain: f32,
    highlight_suppression: f32,
    fine_grain_px: f32,
    medium_grain_px: f32,
    coarse_grain_px: f32,
    clumpiness: f32,
    grain_softness: f32,
    underexposure_grain_boost: f32,
    push_process_boost: f32,
    density_pivot: f32,
    temporal_jitter: f32,
    grain_regenerate_per_frame: f32,
    grain_frame: f32,
    channel_balance_r: f32,
    channel_balance_g: f32,
    channel_balance_b: f32,
    padding0: f32,
    padding1: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: ScanOutputUniform;

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + vec3<f32>(33.33));
    return fract((p3.x + p3.y) * p3.z);
}

fn hash_phase(p: vec2<f32>, phase: f32) -> f32 {
    return hash12(p + vec2<f32>(phase * 0.011, phase * 0.017));
}

fn triangle_hash(p: vec2<f32>, phase: f32) -> f32 {
    return hash_phase(p, phase) + hash_phase(p + vec2<f32>(37.2, 17.7), phase + 19.0) - 1.0;
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
    let film_amount = clamp(uniforms.opacity, 0.0, 1.0);
    let gate_offset = vec2<f32>(
        sin(uniforms.time_seconds * 19.0 + uniforms.seed) * uniforms.gate_weave,
        cos(uniforms.time_seconds * 17.0 + uniforms.seed * 0.5) * uniforms.gate_weave
    ) / uniforms.resolution;
    let scan_uv = clamp(input.uv + gate_offset, vec2<f32>(0.0), vec2<f32>(1.0));
    let scanned = textureSample(source_tex, source_sampler, scan_uv).rgb;

    let luma = luminance(scanned);
    let iso_stops = max(log2(max(uniforms.iso, 50.0) / 100.0), 0.0);
    let iso_gain = 1.0 + iso_stops * 0.12;
    let shadow_mask = 1.0 - smoothstep(0.04, uniforms.density_pivot, luma);
    let mid_mask = smoothstep(0.08, uniforms.density_pivot, luma)
        * (1.0 - smoothstep(uniforms.density_pivot, 0.82, luma));
    let highlight_mask = smoothstep(0.52, 1.0, luma);
    let highlight_term = uniforms.highlight_grain
        * highlight_mask
        * (1.0 - uniforms.highlight_suppression * highlight_mask);
    let density_response =
        uniforms.shadow_grain * shadow_mask + uniforms.midtone_grain * mid_mask + highlight_term;
    let exposure_response = 1.0
        + uniforms.underexposure_grain_boost * shadow_mask * iso_stops * 0.18
        + uniforms.push_process_boost * max(uniforms.iso / 800.0 - 1.0, 0.0) * 0.16;
    let grain_strength = uniforms.grain_luma * density_response * iso_gain * exposure_response * 0.18;

    let pixel = floor(input.uv * uniforms.resolution);
    let regenerate = uniforms.grain_regenerate_per_frame > 0.5;
    let frame_phase = select(0.0, uniforms.grain_frame + 1.0, regenerate);
    let stable_phase = floor(uniforms.seed * 13.0);
    let animated_phase = frame_phase * 131.0 + uniforms.seed * 17.0;
    let time_phase = select(stable_phase, animated_phase, regenerate);
    let frame_jitter = vec2<f32>(
        hash_phase(vec2<f32>(uniforms.seed, frame_phase), time_phase),
        hash_phase(vec2<f32>(frame_phase, uniforms.seed), time_phase + 7.0)
    ) * 4096.0;
    let grain_pixel = pixel + frame_jitter;

    let fine = triangle_hash(grain_pixel / max(uniforms.fine_grain_px, 0.5), time_phase);
    let medium_domain = vec2<f32>(
        grain_pixel.x * 0.871 + grain_pixel.y * 0.247,
        -grain_pixel.x * 0.247 + grain_pixel.y * 0.871
    ) / max(uniforms.medium_grain_px, 0.5);
    let coarse_domain = vec2<f32>(
        grain_pixel.x * 0.613 - grain_pixel.y * 0.421,
        grain_pixel.x * 0.421 + grain_pixel.y * 0.613
    ) / max(uniforms.coarse_grain_px, 0.5);
    let medium = triangle_hash(medium_domain, time_phase + 23.0);
    let coarse = triangle_hash(coarse_domain, time_phase + 71.0);
    let clumpy = mix(medium, coarse, uniforms.clumpiness);
    let structure_mix = clamp(uniforms.grain_softness * 0.45 + uniforms.clumpiness * 0.18, 0.0, 0.82);
    let micro = triangle_hash(grain_pixel * 1.73 + vec2<f32>(11.0, 29.0), time_phase + 97.0)
        * (1.0 - uniforms.grain_softness) * 0.20;
    let raw_grain = clamp(mix(fine, clumpy, structure_mix) + micro, -1.0, 1.0);
    let temporal_noise = select(0.0, hash_phase(vec2<f32>(frame_phase, uniforms.seed), time_phase) - 0.5, regenerate);
    let flicker = 1.0 + temporal_noise * uniforms.flicker * uniforms.temporal_jitter * 0.18;
    let carrier = mix(0.22, max(luma, 0.06), 0.58);
    let density_grain = raw_grain * grain_strength * flicker * carrier;

    let chroma_r = hash_phase(grain_pixel + vec2<f32>(uniforms.seed * 1.7, 13.0), time_phase + 31.0) - 0.5;
    let chroma_g = hash_phase(grain_pixel + vec2<f32>(19.0, uniforms.seed * 1.1), time_phase + 43.0) - 0.5;
    let chroma_b = hash_phase(grain_pixel + vec2<f32>(time_phase * 0.41, uniforms.seed * 2.3), time_phase + 59.0) - 0.5;
    let channel_balance = vec3<f32>(
        uniforms.channel_balance_r,
        uniforms.channel_balance_g,
        uniforms.channel_balance_b
    );
    let chroma = vec3<f32>(chroma_r, chroma_g, chroma_b)
        * uniforms.grain_chroma
        * channel_balance
        * density_response
        * iso_gain
        * 0.045;
    let vignette_distance = distance(input.uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.34, 0.78, vignette_distance) * uniforms.vignette;
    let dust_cell = floor(pixel / 16.0);
    let dust = smoothstep(0.992 - uniforms.dust * 0.04, 1.0, hash12(dust_cell + vec2<f32>(7.0, 19.0)));
    let scratch_line = smoothstep(0.995 - uniforms.scratches * 0.06, 1.0, hash12(vec2<f32>(dust_cell.x * 0.13, 91.0 + uniforms.seed)));
    var film_rgb = clamp(scanned + vec3<f32>(density_grain) + chroma, vec3<f32>(0.0), vec3<f32>(1.0));
    film_rgb = mix(film_rgb, film_rgb * vignette, uniforms.vignette);
    film_rgb = mix(film_rgb, vec3<f32>(0.58, 0.54, 0.48), uniforms.print_fade * 0.12);
    film_rgb = mix(film_rgb, film_rgb * 0.82, dust * uniforms.dust * 0.45);
    film_rgb = mix(film_rgb, vec3<f32>(0.88, 0.85, 0.8), scratch_line * uniforms.scratches * 0.18);
    if (uniforms.scan_softness > 0.001) {
        let softness_px = uniforms.scan_softness * 2.5 / uniforms.resolution;
        let soft = (
            textureSample(source_tex, source_sampler, clamp(scan_uv + vec2<f32>( softness_px.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))).rgb +
            textureSample(source_tex, source_sampler, clamp(scan_uv - vec2<f32>( softness_px.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))).rgb +
            textureSample(source_tex, source_sampler, clamp(scan_uv + vec2<f32>( 0.0, softness_px.y), vec2<f32>(0.0), vec2<f32>(1.0))).rgb +
            textureSample(source_tex, source_sampler, clamp(scan_uv - vec2<f32>( 0.0, softness_px.y), vec2<f32>(0.0), vec2<f32>(1.0))).rgb
        ) * 0.25;
        film_rgb = mix(film_rgb, soft, uniforms.scan_softness * 0.25);
    }
    let rgb = mix(base.rgb, film_rgb, film_amount);
    return vec4<f32>(rgb, base.a);
}
"#;
