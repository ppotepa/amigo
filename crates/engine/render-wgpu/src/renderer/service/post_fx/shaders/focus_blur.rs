pub(crate) const FOCUS_BLUR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct FocusBlurUniform {
    focus: vec4<f32>,
    optics: vec4<f32>,
    boost: vec4<f32>,
    flags: vec4<f32>,
    aperture: vec4<f32>,
    highlight: vec4<f32>,
    depth_override: vec4<f32>,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_2d<f32>;
@group(0) @binding(2) var highlight_tex: texture_2d<f32>;
@group(0) @binding(3) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: FocusBlurUniform;

const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 6.28318530718;
const GOLDEN_ANGLE: f32 = 2.39996322973;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn sample_depth(uv: vec2<f32>) -> f32 {
    if uniforms.depth_override.x > 0.5 {
        return clamp(uniforms.depth_override.y, 0.0, 1.0);
    }
    let clamped_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let raw = dot(
        textureSample(depth_tex, source_sampler, clamped_uv).rgb,
        vec3<f32>(0.299, 0.587, 0.114)
    );
    let white_near = select(raw, 1.0 - raw, uniforms.flags.x > 0.5);
    return clamp((white_near - 0.5) * uniforms.optics.z + 0.5, 0.0, 1.0);
}

fn resolved_focus_depth() -> f32 {
    let explicit_focus_depth = uniforms.focus.z;
    let sampled = sample_depth(clamp(uniforms.focus.xy, vec2<f32>(0.0), vec2<f32>(1.0)));
    return select(clamp(explicit_focus_depth, 0.0, 1.0), sampled, explicit_focus_depth < -0.5);
}

fn signed_coc(depth: f32, focus_depth: f32) -> f32 {
    let delta = depth - focus_depth;
    let ad = abs(delta);
    let focus_width = max(uniforms.optics.w, 0.001);
    let outside_focus = smoothstep(focus_width * 0.25, focus_width, ad);
    let lens_scale = pow(max(uniforms.optics.x, 1.0) / 50.0, 1.22);
    let aperture_scale = 2.8 / max(uniforms.focus.w, 0.1);
    let side_boost = select(uniforms.boost.y, uniforms.boost.x, delta > 0.0);
    let amount = clamp((ad / focus_width) * outside_focus * lens_scale * aperture_scale * side_boost * 0.36, 0.0, 1.0);
    return select(-amount, amount, delta >= 0.0);
}

fn polygon_radius(theta: f32, blades: f32) -> f32 {
    if blades < 3.0 {
        return 1.0;
    }
    let sector = TWO_PI / blades;
    let local = theta - sector * floor((theta + sector * 0.5) / sector);
    let denom = max(cos(local), 0.08);
    return clamp(cos(PI / blades) / denom, 0.25, 1.25);
}

fn aperture_offset(i: u32, sample_count: f32, uv: vec2<f32>, rotation: f32) -> vec2<f32> {
    let fi = f32(i) + 0.5;
    let n = max(sample_count, 1.0);
    let t = fi / n;
    let r = sqrt(t);
    let jitter = hash12(floor(uv * vec2<f32>(1920.0, 1080.0)) + vec2<f32>(17.0, 91.0));
    let theta = fi * GOLDEN_ANGLE + rotation + jitter * 0.21;
    let blade_r = mix(polygon_radius(theta, uniforms.aperture.x), 1.0, uniforms.aperture.y);
    var p = vec2<f32>(cos(theta), sin(theta)) * r * blade_r;
    p.x *= max(uniforms.flags.z, 0.25);
    let edge = saturate(length((uv - vec2<f32>(0.5)) * vec2<f32>(2.0, 2.0)));
    let cat = saturate(uniforms.flags.w);
    let squeeze = mix(1.0, max(0.34, 1.0 - edge * edge * 0.72), cat);
    p.x *= squeeze;
    return p;
}

fn highlight_boost(rgb: vec3<f32>, highlight_rgb: vec3<f32>) -> vec3<f32> {
    let threshold = uniforms.highlight.x;
    let knee = max(uniforms.highlight.y, 0.001);
    let gain = uniforms.highlight.z;
    let saturation = uniforms.highlight.w;
    let luma = max(luminance(rgb), luminance(highlight_rgb));
    let mask = smoothstep(threshold - knee, threshold + knee, luma);
    let gray = vec3<f32>(luma);
    let saturated = mix(gray, rgb, saturation);
    return rgb + saturated * mask * gain + highlight_rgb * mask * gain * 0.25;
}

fn highlight_mask(rgb: vec3<f32>, highlight_rgb: vec3<f32>) -> f32 {
    let threshold = uniforms.highlight.x;
    let knee = max(uniforms.highlight.y, 0.001);
    return smoothstep(threshold - knee, threshold + knee, max(luminance(rgb), luminance(highlight_rgb)));
}

fn debug_color(uv: vec2<f32>, depth: f32, coc: f32, focus_depth: f32, base: vec4<f32>, blurred: vec3<f32>) -> vec4<f32> {
    let debug_view = uniforms.flags.y;
    if debug_view > 0.5 && debug_view < 1.5 {
        return vec4<f32>(vec3<f32>(depth), base.a);
    }
    if debug_view >= 1.5 && debug_view < 2.5 {
        let near_c = saturate(coc);
        let far_c = saturate(-coc);
        let focus = 1.0 - saturate(abs(coc) * 8.0);
        return vec4<f32>(near_c, focus, far_c, base.a);
    }
    if debug_view >= 2.5 && debug_view < 3.5 {
        let band = 1.0 - smoothstep(uniforms.optics.w * 0.45, uniforms.optics.w, abs(depth - focus_depth));
        return vec4<f32>(mix(base.rgb * 0.22, vec3<f32>(0.12, 0.95, 0.38), band), base.a);
    }
    if debug_view >= 3.5 && debug_view < 4.5 {
        if uv.x < 0.5 {
            return base;
        }
        return vec4<f32>(blurred, base.a);
    }
    if debug_view >= 4.5 && debug_view < 5.5 {
        var highlight_rgb = vec3<f32>(0.0);
        if uniforms.depth_override.w > 0.5 {
            highlight_rgb = textureSample(highlight_tex, source_sampler, uv).rgb;
        }
        let mask = highlight_mask(base.rgb, highlight_rgb);
        return vec4<f32>(vec3<f32>(mask), base.a);
    }
    return vec4<f32>(blurred, base.a);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let base = textureSample(source_tex, source_sampler, input.uv);
    let dims = vec2<f32>(textureDimensions(source_tex, 0));
    let center_depth = sample_depth(input.uv);
    let focus_depth = resolved_focus_depth();
    let center_coc = signed_coc(center_depth, focus_depth);
    let abs_center_coc = abs(center_coc);
    let radius_px = abs_center_coc * uniforms.optics.y * uniforms.depth_override.z;

    if radius_px < 0.35 && uniforms.flags.y < 0.5 {
        return base;
    }

    let sample_count = clamp(uniforms.aperture.w, 12.0, 64.0);
    let texel_radius = vec2<f32>(radius_px / max(dims.x, 1.0), radius_px / max(dims.y, 1.0));

    var accum_rgb = base.rgb * base.a;
    var accum_alpha = base.a;
    var weight_sum = max(base.a, 0.0001);
    for (var i = 0u; i < 64u; i = i + 1u) {
        if f32(i) >= sample_count {
            continue;
        }
        let unit = aperture_offset(i, sample_count, input.uv, uniforms.aperture.z);
        let sample_uv = clamp(input.uv + unit * texel_radius, vec2<f32>(0.0), vec2<f32>(1.0));
        let sample_depth_value = sample_depth(sample_uv);
        let sample_coc = signed_coc(sample_depth_value, focus_depth);
        let sample = textureSample(source_tex, source_sampler, sample_uv);
        var sample_highlight = vec3<f32>(0.0);
        if uniforms.depth_override.w > 0.5 {
            sample_highlight = textureSample(highlight_tex, source_sampler, sample_uv).rgb;
        }
        let sample_rgb_raw = sample.rgb;
        let sample_rgb = highlight_boost(sample_rgb_raw, sample_highlight);
        let depth_gap = abs(sample_depth_value - center_depth);
        var edge_weight = 1.0;
        if uniforms.boost.w > 0.5 {
            edge_weight = max(0.08, 1.0 - smoothstep(0.045, 0.32, depth_gap));
        }
        let same_side = center_coc * sample_coc >= -0.0005;
        let side_weight = select(0.22, 1.0, same_side);
        let radial = saturate(length(unit));
        let aperture_weight = mix(1.0, 0.58, radial);
        let h = highlight_mask(sample_rgb_raw, sample_highlight);
        let weight = max(edge_weight * side_weight * aperture_weight, h * 0.34);
        accum_rgb += sample_rgb * sample.a * weight;
        accum_alpha += sample.a * weight;
        weight_sum += weight;
    }

    let blurred_alpha = accum_alpha / max(weight_sum, 0.0001);
    let blurred = accum_rgb / max(accum_alpha, 0.0001);
    if uniforms.flags.y > 0.5 {
        return debug_color(
            input.uv,
            center_depth,
            center_coc,
            focus_depth,
            vec4<f32>(base.rgb, blurred_alpha),
            blurred,
        );
    }

    let blend = smoothstep(0.012, 0.78, abs_center_coc) * uniforms.boost.z;
    return vec4<f32>(mix(base.rgb, blurred, blend), mix(base.a, blurred_alpha, blend));
}
"#;
