pub(crate) const FILM_EMULSION_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct FilmEmulsionUniform {
    color_shift: f32,
    contrast: f32,
    saturation: f32,
    toe: f32,
    shoulder: f32,
    black_lift: f32,
    push_pull: f32,
    opacity: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var normal_tex: texture_2d<f32>;
@group(0) @binding(2) var wetness_tex: texture_2d<f32>;
@group(0) @binding(3) var highlight_tex: texture_2d<f32>;
@group(0) @binding(4) var emissive_tex: texture_2d<f32>;
@group(0) @binding(5) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: FilmEmulsionUniform;

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
    let source_energy = max(luminance(highlight_source), luminance(emissive_source));
    let luma = max(luminance(base.rgb), source_energy);
    let shadow_weight = 1.0 - smoothstep(0.58, 1.0, luma);

    var graded = (base.rgb - vec3<f32>(0.5)) * uniforms.contrast + vec3<f32>(0.5);
    let gray = vec3<f32>(luminance(graded));
    graded = mix(gray, graded, uniforms.saturation);
    graded.r += uniforms.color_shift * 0.16 * shadow_weight;
    graded.g += uniforms.color_shift * 0.035 * (1.0 - shadow_weight);
    graded.b -= uniforms.color_shift * 0.12 * shadow_weight;
    graded += vec3<f32>(uniforms.push_pull * 0.006);
    graded = pow(clamp(graded, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(mix(1.3, 0.75, 1.0 - uniforms.toe)));
    let shoulder_rolloff = 1.0 - exp(-graded * (1.0 + uniforms.shoulder * 3.5));
    graded = mix(graded, shoulder_rolloff, uniforms.shoulder * (0.65 + source_energy * 0.22));
    graded += vec3<f32>(uniforms.black_lift);
    graded += highlight_source * uniforms.shoulder * 0.10 + emissive_source * max(uniforms.push_pull, 0.0) * 0.035;

    let rgb = mix(base.rgb, clamp(graded, vec3<f32>(0.0), vec3<f32>(1.0)), uniforms.opacity);
    return vec4<f32>(rgb, base.a);
}
"#;
