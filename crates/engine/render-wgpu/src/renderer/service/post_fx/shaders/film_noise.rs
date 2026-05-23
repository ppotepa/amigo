pub(crate) const FILM_NOISE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct FilmNoiseUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    iso: f32,
    grain_size: f32,
    chroma_noise: f32,
    color_shift: f32,
    contrast: f32,
    saturation: f32,
    flicker: f32,
    vignette: f32,
    toe: f32,
    shoulder: f32,
    black_lift: f32,
    print_fade: f32,
    dust: f32,
    scratches: f32,
    push_pull: f32,
    gate_weave: f32,
    scan_softness: f32,
    opacity: f32,
    seed: f32,
    _pad0: vec3<f32>,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: FilmNoiseUniform;

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
    let luma = luminance(base.rgb);
    let iso_stops = max(log2(max(uniforms.iso, 50.0) / 100.0), 0.0);
    let grain_strength = iso_stops * (0.045 + uniforms.push_pull * 0.01);
    let shadow_weight = 1.0 - smoothstep(0.58, 1.0, luma);
    let grain_density = max(20.0, 320.0 / max(uniforms.grain_size, 0.25));
    let cell = floor(input.uv * uniforms.resolution / grain_density);
    let time_phase = floor(fract(uniforms.time_seconds / 4096.0) * 98304.0);
    let mono = hash12(cell + vec2<f32>(time_phase + uniforms.seed, uniforms.seed * 0.17)) - 0.5;
    let chroma_r = hash12(cell + vec2<f32>(uniforms.seed * 1.7, time_phase)) - 0.5;
    let chroma_b = hash12(cell + vec2<f32>(time_phase, uniforms.seed * 2.3)) - 0.5;
    let flicker = 1.0 + (hash12(vec2<f32>(time_phase, uniforms.seed)) - 0.5) * uniforms.flicker;
    let vignette_distance = distance(input.uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.34, 0.78, vignette_distance) * uniforms.vignette;
    let gate_offset = vec2<f32>(
        sin(uniforms.time_seconds * 19.0 + uniforms.seed) * uniforms.gate_weave,
        cos(uniforms.time_seconds * 17.0 + uniforms.seed * 0.5) * uniforms.gate_weave
    ) / uniforms.resolution;
    let scan_uv = clamp(input.uv + gate_offset, vec2<f32>(0.0), vec2<f32>(1.0));
    let scanned = textureSample(source_tex, source_sampler, scan_uv).rgb;

    var graded = (scanned - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);
    let gray = vec3<f32>(luminance(graded));
    graded = mix(gray, graded, uniforms.saturation);
    graded.r += uniforms.color_shift * 0.16 * shadow_weight;
    graded.g += uniforms.color_shift * 0.035 * (1.0 - shadow_weight);
    graded.b -= uniforms.color_shift * 0.12 * shadow_weight;
    graded = pow(clamp(graded, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(mix(1.3, 0.75, 1.0 - uniforms.toe)));
    let shoulder_rolloff = 1.0 - exp(-graded * (1.0 + uniforms.shoulder * 3.5));
    graded = mix(graded, shoulder_rolloff, uniforms.shoulder * 0.65);
    graded = mix(graded, vec3<f32>(0.58, 0.54, 0.48), uniforms.print_fade * 0.12);
    graded += vec3<f32>(uniforms.black_lift);

    let grain = mono * grain_strength * shadow_weight * flicker;
    let chroma = vec3<f32>(chroma_r, 0.0, chroma_b) * uniforms.chroma_noise * grain_strength;
    let dust = smoothstep(0.992 - uniforms.dust * 0.04, 1.0, hash12(cell + vec2<f32>(7.0, 19.0)));
    let scratch_line = smoothstep(0.995 - uniforms.scratches * 0.06, 1.0, hash12(vec2<f32>(cell.x * 0.13, 91.0 + uniforms.seed)));
    var film_rgb = clamp((graded + vec3<f32>(grain) + chroma) * vignette, vec3<f32>(0.0), vec3<f32>(1.0));
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
    let rgb = mix(base.rgb, film_rgb, uniforms.opacity);
    return vec4<f32>(rgb, base.a);
}
"#;
