pub(crate) const CAMERA_EXPOSURE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CameraExposureUniform {
    iso: f32,
    compensation: f32,
    white_balance: f32,
    nd_stops: f32,
    target_luma: f32,
    adaptation_speed: f32,
    min_iso: f32,
    max_iso: f32,
    opacity: f32,
    mode: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: CameraExposureUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn average_luma() -> f32 {
    var sum = 0.0;
    for (var y = 0; y < 4; y = y + 1) {
        for (var x = 0; x < 4; x = x + 1) {
            let uv = vec2<f32>((f32(x) + 0.5) / 4.0, (f32(y) + 0.5) / 4.0);
            sum += luminance(textureSample(source_tex, source_sampler, uv).rgb);
        }
    }
    return max(sum / 16.0, 0.001);
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
    let avg_luma = average_luma();
    let manual_scale = pow(max(uniforms.iso, 25.0) / 400.0, 0.35) * exp2(uniforms.compensation - uniforms.nd_stops);
    let desired_iso = clamp((uniforms.target_luma / avg_luma) * 400.0, uniforms.min_iso, uniforms.max_iso);
    let auto_scale = pow(desired_iso / 400.0, 0.35) * exp2(uniforms.compensation - uniforms.nd_stops);
    let mode_scale = mix(manual_scale, auto_scale, uniforms.mode);
    let response = clamp(uniforms.adaptation_speed / 8.0, 0.0, 1.0);
    let exposure_scale = mix(1.0, mode_scale, response);
    let white_balance = clamp((uniforms.white_balance - 5600.0) / 5600.0, -1.0, 1.0);
    let wb = vec3<f32>(1.0 + white_balance * 0.08, 1.0, 1.0 - white_balance * 0.12);
    let exposed = clamp(base.rgb * exposure_scale * wb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(mix(base.rgb, exposed, uniforms.opacity), base.a);
}
"#;
