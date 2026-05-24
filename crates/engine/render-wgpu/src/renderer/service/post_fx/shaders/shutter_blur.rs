pub(crate) const SHUTTER_BLUR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct ShutterBlurUniform {
    resolution: vec2<f32>,
    opacity: f32,
    shutter_fraction: f32,
    exposure_seconds: f32,
    history_mix: f32,
    history_mix_2: f32,
    edge_rejection: f32,
    luma_threshold: f32,
    dt: f32,
    target_dt: f32,
    history_ready_a: f32,
    history_ready_b: f32,
    frame_hold: f32,
    debug_motion: f32,
    padding: f32,
}

@group(0) @binding(0) var current_texture: texture_2d<f32>;
@group(0) @binding(1) var previous_texture: texture_2d<f32>;
@group(0) @binding(2) var previous_texture_2: texture_2d<f32>;
@group(0) @binding(3) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: ShutterBlurUniform;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let current = textureSample(current_texture, source_sampler, input.uv);
    let previous = textureSample(previous_texture, source_sampler, input.uv);
    let previous2 = textureSample(previous_texture_2, source_sampler, input.uv);
    let delta1 = abs(luma(current.rgb) - luma(previous.rgb));
    let delta2 = abs(luma(current.rgb) - luma(previous2.rgb));

    if (uniforms.debug_motion > 0.5) {
        let trail = clamp(delta1 * 3.5, 0.0, 1.0);
        let echo = clamp(delta2 * 2.5, 0.0, 1.0);
        let exposure_frames = clamp(uniforms.exposure_seconds / max(uniforms.dt, 0.0001), 0.0, 12.0) / 12.0;
        let rgb = vec3<f32>(trail, exposure_frames, echo);
        let alpha = max(uniforms.history_ready_a, uniforms.history_ready_b);
        return vec4<f32>(rgb, alpha);
    }

    if (uniforms.history_mix > 0.0) {
        let w1 = clamp(uniforms.history_mix * uniforms.opacity, 0.0, 1.0) * uniforms.history_ready_a;
        let w2 = clamp(uniforms.history_mix_2 * uniforms.opacity, 0.0, 1.0) * uniforms.history_ready_b;
        let gate1 = smoothstep(
            uniforms.luma_threshold,
            uniforms.luma_threshold + max(uniforms.edge_rejection, 0.001),
            delta1
        );
        let gate2 = smoothstep(
            uniforms.luma_threshold,
            uniforms.luma_threshold + max(uniforms.edge_rejection, 0.001),
            delta2
        );
        let trail = clamp(
            previous.rgb * w1 * gate1 + previous2.rgb * w2 * gate2,
            vec3<f32>(0.0),
            vec3<f32>(1.0)
        );
        let color = vec3<f32>(1.0) - (vec3<f32>(1.0) - current.rgb) * (vec3<f32>(1.0) - trail);
        let alpha = max(current.a, max(previous.a * w1 * gate1, previous2.a * w2 * gate2));
        return vec4<f32>(color, alpha);
    }

    let retention = exp(-uniforms.dt / max(uniforms.exposure_seconds, 0.0001));
    let history_weight = clamp(retention * uniforms.opacity, 0.0, 0.98) * uniforms.history_ready_a;
    let current_weight = 1.0 - history_weight;
    let exposure_frames = uniforms.exposure_seconds / max(uniforms.dt, 0.0001);
    let trail_strength = clamp((exposure_frames - 1.0) / 8.0, 0.0, 0.65) * uniforms.opacity * uniforms.history_ready_a;
    let accumulated = current.rgb * current_weight + previous.rgb * history_weight;
    let lifted_history = clamp(previous.rgb * (0.65 + trail_strength), vec3<f32>(0.0), vec3<f32>(1.0));
    let trail = vec3<f32>(1.0) - (vec3<f32>(1.0) - current.rgb) * (vec3<f32>(1.0) - lifted_history);
    let color = mix(accumulated, trail, trail_strength);
    let alpha = max(current.a, previous.a * history_weight);

    return vec4<f32>(color, alpha);
}
"#;
