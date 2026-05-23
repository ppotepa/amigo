use crate::renderer::*;

const PLATE_RELIGHT_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct PlateRelightUniform {
    canvas: vec4<f32>,
    params0: vec4<f32>,
    params1: vec4<f32>,
    params2: vec4<f32>,
    params3: vec4<f32>,
    params4: vec4<f32>,
    light_pos_rad: array<vec4<f32>, 16>,
    light_color_intensity: array<vec4<f32>, 16>,
    light_dir_type: array<vec4<f32>, 16>,
    light_extra: array<vec4<f32>, 16>,
}

@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var surface_tex: texture_2d<f32>;
@group(0) @binding(2) var depth_aux_tex: texture_2d<f32>;
@group(0) @binding(3) var depth_tex: texture_2d<f32>;
@group(0) @binding(4) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: PlateRelightUniform;

struct LightEval {
    contribution: vec3<f32>,
    light_mask: f32,
    ndl: f32,
    specular: f32,
    material_gate: f32,
    shadow: f32,
}

fn height_scale() -> f32 { return uniforms.params2.x; }
fn normal_strength() -> f32 { return uniforms.params2.y; }
fn ao_strength() -> f32 { return uniforms.params2.z; }
fn reflection_strength() -> f32 { return uniforms.params2.w; }
fn ambient_light() -> f32 { return uniforms.params0.x; }
fn plate_preserve() -> f32 { return uniforms.params0.y; }
fn relight_blend() -> f32 { return uniforms.params0.z; }
fn base_darkness() -> f32 { return uniforms.params0.w; }
fn albedo_gain() -> f32 { return uniforms.params1.x; }
fn computed_light_gain() -> f32 { return uniforms.params1.y; }
fn shadow_lift() -> f32 { return uniforms.params1.z; }
fn highlight_suppress() -> f32 { return uniforms.params1.w; }
fn specular_boost() -> f32 { return uniforms.params3.x; }
fn shadow_strength() -> f32 { return uniforms.params3.y; }
fn shadow_bias() -> f32 { return uniforms.params3.z; }
fn shadow_softness() -> f32 { return uniforms.params3.w; }
fn shadow_steps() -> i32 { return i32(uniforms.params4.x + 0.5); }
fn debug_mode() -> f32 { return uniforms.params4.y; }

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn debug_gray(v: f32) -> vec4<f32> {
    let g = clamp(v, 0.0, 1.0);
    return vec4<f32>(vec3<f32>(g), 1.0);
}

fn debug_normal(n: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(n * 0.5 + vec3<f32>(0.5), 1.0);
}

fn safe_uv(uv: vec2<f32>) -> vec2<f32> {
    return clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
}

fn texel_size() -> vec2<f32> {
    return vec2<f32>(1.0) / max(uniforms.canvas.xy, vec2<f32>(1.0));
}

fn sample_aux(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(depth_aux_tex, source_sampler, safe_uv(uv));
}

fn sample_base_depth(uv: vec2<f32>) -> f32 {
    let aux = sample_aux(uv);
    var depth_sample = textureSample(depth_tex, source_sampler, safe_uv(uv)).r;
    let mode = uniforms.canvas.z;
    if (mode < -0.5) {
        depth_sample = 1.0 - depth_sample;
    }
    var use_depth = 0.0;
    if (abs(mode) > 0.5) {
        use_depth = 1.0;
    }
    return mix(aux.r, depth_sample, use_depth);
}

fn effective_depth(uv: vec2<f32>) -> f32 {
    let aux = sample_aux(uv);
    return clamp(sample_base_depth(uv) - aux.g * height_scale() * aux.a, 0.0, 1.0);
}

fn normal_at(uv: vec2<f32>) -> vec3<f32> {
    let t = texel_size();
    let dl = effective_depth(uv - vec2<f32>(t.x, 0.0));
    let dr = effective_depth(uv + vec2<f32>(t.x, 0.0));
    let dt = effective_depth(uv - vec2<f32>(0.0, t.y));
    let db = effective_depth(uv + vec2<f32>(0.0, t.y));
    return normalize(vec3<f32>((dl - dr) * normal_strength(), (dt - db) * normal_strength(), 1.0));
}

fn local_ao(uv: vec2<f32>, d0: f32) -> f32 {
    let t = texel_size() * 2.0;
    var occ = 0.0;
    var count = 0.0;
    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            if (x != 0 || y != 0) {
                let off = vec2<f32>(f32(x), f32(y)) * t;
                let sd = effective_depth(uv + off);
                occ = occ + max(0.0, d0 - sd);
                count = count + 1.0;
            }
        }
    }
    return clamp(1.0 - (occ / max(count, 1.0)) * 8.0 * ao_strength(), 0.0, 1.0);
}

fn light_z_from_distance(distance_m: f32) -> f32 {
    let distance = max(distance_m, 0.05);
    return clamp(0.18 + 0.62 / (1.0 + distance * 0.85), 0.18, 0.80);
}

fn shadow_ray(uv: vec2<f32>, d0: f32, light_pos_rad: vec4<f32>, light_extra: vec4<f32>) -> f32 {
    if (light_extra.z < 0.5) {
        return 1.0;
    }
    let light_uv = light_pos_rad.xy;
    var ray = light_uv - uv;
    let dist2d = length(ray);
    if (dist2d < 0.0005) {
        return 1.0;
    }
    ray = ray / dist2d;

    var blocked = 0.0;
    let steps_i = max(shadow_steps(), 1);
    let steps = f32(steps_i);

    for (var s: i32 = 1; s <= 32; s = s + 1) {
        if (s > steps_i) {
            break;
        }
        let t = f32(s) / (steps + 1.0);
        let suv = safe_uv(uv + ray * dist2d * t);
        let a = sample_aux(suv);
        let sd = effective_depth(suv);
        let expected = mix(d0, light_pos_rad.z, t);
        let pen = max(0.0, expected - sd - shadow_bias());
        let blocker = clamp(a.b * a.a, 0.0, 1.0);
        let soft_hit = smoothstep(0.0004, 0.0004 + shadow_softness() * 0.040, pen);
        blocked = blocked + soft_hit * blocker;
    }

    let shade = 1.0 - (blocked / max(steps, 1.0)) * shadow_strength() * light_extra.x;
    return clamp(shade, shadow_lift(), 1.0);
}

fn shadow_debug_value(uv: vec2<f32>, d0: f32, count: i32) -> f32 {
    var s = 1.0;
    for (var i: i32 = 0; i < 16; i = i + 1) {
        if (i >= count) {
            break;
        }
        s = min(s, shadow_ray(uv, d0, uniforms.light_pos_rad[i], uniforms.light_extra[i]));
    }
    return s;
}

fn eval_light(
    uv: vec2<f32>,
    surface: vec4<f32>,
    aux: vec4<f32>,
    normal: vec3<f32>,
    d0: f32,
    light_pos_rad: vec4<f32>,
    light_color_intensity: vec4<f32>,
    light_dir_type: vec4<f32>,
    light_extra: vec4<f32>,
) -> LightEval {
    let canvas = max(uniforms.canvas.xy, vec2<f32>(1.0));
    let radius = max(light_pos_rad.w, 0.001);

    let light_uv = light_pos_rad.xy;
    let aspect = canvas.x / max(canvas.y, 1.0);
    let delta = vec2<f32>((light_uv.x - uv.x) * aspect, light_uv.y - uv.y);
    let signed_depth = light_pos_rad.z - d0;
    let dz = signed_depth * 0.38;
    let dist3 = length(vec3<f32>(delta, dz));
    let falloff = smoothstep(radius, 0.0, dist3);
    let shaped_falloff = falloff * falloff;
    let lamp_height = abs(signed_depth) * 0.45 + 0.18;
    let light_vec = vec3<f32>(
        delta.x,
        delta.y,
        lamp_height
    );
    let l = normalize(light_vec);
    var att = shaped_falloff;
    if (light_dir_type.w > 0.5) {
        let to_pixel = normalize(uv - light_uv);
        let spot_align = dot(to_pixel, normalize(light_dir_type.xy));
        att = att * smoothstep(light_dir_type.z, min(light_dir_type.z + 0.18, 1.0), spot_align);
    }
    let shadow = shadow_ray(uv, d0, light_pos_rad, light_extra);

    let reflectivity = clamp(surface.r, 0.0, 1.0);
    let roughness = clamp(surface.g, 0.02, 1.0);
    let glass = clamp(surface.b, 0.0, 1.0);
    let mask = clamp(surface.a, 0.0, 1.0);
    let material_gate = mix(0.50, 1.0, mask);
    let valid_gate = mix(0.55, 1.0, clamp(aux.a, 0.0, 1.0));
    let occlusion = mix(1.0, 0.62, clamp(aux.b * aux.a, 0.0, 1.0));

    let ndl_raw = max(0.0, dot(normal, l));
    let wrapped_ndl = clamp(dot(normal, l) * 0.5 + 0.5, 0.0, 1.0);
    let ndl = ndl_raw * 0.72 + wrapped_ndl * 0.28;
    let diffuse = ndl * (0.34 + reflectivity * 0.22 + glass * 0.10 + aux.g * 0.20);

    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_dir = normalize(l + view_dir);
    let gloss = mix(18.0, 176.0, clamp(1.0 - roughness, 0.0, 1.0));
    let spec = pow(max(dot(normal, half_dir), 0.0), gloss)
        * (reflectivity * 0.95 + glass * 1.25)
        * specular_boost()
        * light_extra.w;

    let fresnel = pow(clamp(1.0 - normal.z, 0.0, 1.0), 2.5)
        * (reflectivity + glass * 0.8)
        * 0.24;
    let height_edge = smoothstep(0.05, 0.74, clamp(aux.g, 0.0, 1.0)) * 0.22;
    let base_probe = 0.08 + aux.g * 0.08 + reflectivity * 0.04 + glass * 0.04;
    let response = diffuse + spec + fresnel + height_edge + base_probe;
    let contribution = light_color_intensity.rgb
        * light_color_intensity.a
        * att
        * response
        * shadow
        * occlusion
        * valid_gate
        * material_gate
        * computed_light_gain();
    return LightEval(
        contribution,
        att * light_color_intensity.a * light_extra.y,
        ndl_raw * att,
        spec * att * light_color_intensity.a,
        material_gate * valid_gate,
        shadow,
    );
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
    let scene = textureSample(scene_tex, source_sampler, input.uv);
    let surface = textureSample(surface_tex, source_sampler, input.uv);
    let aux = textureSample(depth_aux_tex, source_sampler, input.uv);
    let d0 = effective_depth(input.uv);
    let normal = normal_at(input.uv);
    let ao = local_ao(input.uv, d0);
    let count = i32(uniforms.canvas.w + 0.5);
    let debug_mode = debug_mode();

    if (debug_mode > 0.5 && debug_mode < 1.5) {
        return debug_gray(aux.r);
    }
    if (debug_mode > 1.5 && debug_mode < 2.5) {
        return debug_gray(aux.g);
    }
    if (debug_mode > 2.5 && debug_mode < 3.5) {
        return debug_gray(aux.b);
    }
    if (debug_mode > 3.5 && debug_mode < 4.5) {
        return debug_gray(aux.a);
    }
    if (debug_mode > 4.5 && debug_mode < 5.5) {
        return debug_gray(surface.r);
    }
    if (debug_mode > 5.5 && debug_mode < 6.5) {
        return debug_gray(surface.g);
    }
    if (debug_mode > 6.5 && debug_mode < 7.5) {
        return debug_gray(surface.b);
    }
    if (debug_mode > 7.5 && debug_mode < 8.5) {
        return debug_gray(surface.a);
    }
    if (debug_mode > 8.5 && debug_mode < 9.5) {
        return debug_gray(d0);
    }
    if (debug_mode > 9.5 && debug_mode < 10.5) {
        return debug_normal(normal);
    }
    if (debug_mode > 10.5 && debug_mode < 11.5) {
        return debug_gray(ao * mix(1.0, 0.90, clamp(aux.b * aux.a, 0.0, 1.0)));
    }
    if (debug_mode > 12.5 && debug_mode < 13.5) {
        return debug_gray(shadow_debug_value(input.uv, d0, count));
    }

    var relight = vec3<f32>(0.0);
    var light_mask = 0.0;
    var ndl_accum = 0.0;
    var spec_accum = 0.0;
    var material_gate_accum = 0.0;
    var shadow_accum = 1.0;
    for (var i: i32 = 0; i < 16; i = i + 1) {
        if (i >= count) {
            break;
        }
        let e = eval_light(
            input.uv,
            surface,
            aux,
            normal,
            d0,
            uniforms.light_pos_rad[i],
            uniforms.light_color_intensity[i],
            uniforms.light_dir_type[i],
            uniforms.light_extra[i],
        );
        relight = relight + e.contribution;
        light_mask = light_mask + e.light_mask;
        ndl_accum = ndl_accum + e.ndl;
        spec_accum = spec_accum + e.specular;
        material_gate_accum = max(material_gate_accum, e.material_gate);
        shadow_accum = min(shadow_accum, e.shadow);
    }
    if (debug_mode > 11.5 && debug_mode < 12.5) {
        let c = clamp(luminance(relight) * 1.5, 0.0, 1.0);
        let peak = max(max(relight.r, max(relight.g, relight.b)), 0.001);
        return vec4<f32>(relight / peak * c, 1.0);
    }
    if (debug_mode > 13.5 && debug_mode < 14.5) {
        return debug_gray(light_mask * 0.25);
    }
    if (debug_mode > 14.5 && debug_mode < 15.5) {
        return debug_gray(ndl_accum);
    }
    if (debug_mode > 15.5 && debug_mode < 16.5) {
        return debug_gray(spec_accum * 1.4);
    }
    if (debug_mode > 16.5 && debug_mode < 17.5) {
        return debug_gray(material_gate_accum);
    }

    let valid = mix(0.45, 1.0, clamp(aux.a, 0.0, 1.0));
    let blocker = clamp(aux.b * aux.a, 0.0, 1.0);
    let static_occ = mix(1.0, 0.82, blocker);
    let lifted = mix(scene.rgb, sqrt(max(scene.rgb, vec3<f32>(0.0))), shadow_lift());
    let compressed = lifted / (vec3<f32>(1.0) + lifted * highlight_suppress() * 1.6);
    let base = mix(lifted, compressed, highlight_suppress());
    let plate = scene.rgb * base_darkness() * plate_preserve() * valid * static_occ;
    let albedo = base * base_darkness() * albedo_gain() * valid * static_occ;
    let hot = smoothstep(0.22, 0.95, luminance(relight));
    let shadow_region = clamp(light_mask * 0.28, 0.0, 1.0);
    let dynamic_shadow = mix(1.0, shadow_accum, shadow_region * 0.55 * shadow_strength());

    let reflectivity = clamp(surface.r, 0.0, 1.0);
    let roughness = clamp(surface.g, 0.02, 1.0);
    let glass = clamp(surface.b, 0.0, 1.0);
    let refl_uv = safe_uv(input.uv + normal.xy * (0.010 + aux.g * 0.018) * (reflectivity + glass * 0.55));
    let reflection = textureSample(scene_tex, source_sampler, refl_uv).rgb
        * (reflectivity + glass * 0.40)
        * (1.0 - roughness)
        * luminance(relight)
        * reflection_strength();

    let lit = plate * dynamic_shadow
        + albedo * ambient_light() * ao * mix(1.0, dynamic_shadow, 0.35)
        + albedo * relight * (1.10 * relight_blend())
        + relight * (0.42 + hot * 0.34)
        + scene.rgb * clamp(light_mask * 0.020, 0.0, 0.22) * relight_blend()
        + reflection;
    if (debug_mode > 17.5 && debug_mode < 18.5) {
        return vec4<f32>(min(lit, vec3<f32>(4.0)), scene.a);
    }
    return vec4<f32>(min(lit, vec3<f32>(4.0)), scene.a);
}
"#;

const WET_REFLECTIONS_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct WetReflectionsUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    mask_invert: f32,
    blur_px: f32,
    distortion_px: f32,
    shimmer_strength: f32,
    ripple_strength: f32,
    wet_darken: f32,
    specular_boost: f32,
    edge_power: f32,
    light_reflection_strength: f32,
    foreground_strength: f32,
    background_strength: f32,
    horizon_y: f32,
    noise_scale: f32,
    noise_speed: f32,
    ripple_speed: f32,
    _pad0: vec2<f32>,
}

@group(0) @binding(0) var world_tex: texture_2d<f32>;
@group(0) @binding(1) var mask_tex: texture_2d<f32>;
@group(0) @binding(2) var edge_tex: texture_2d<f32>;
@group(0) @binding(3) var reflection_color_tex: texture_2d<f32>;
@group(0) @binding(4) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: WetReflectionsUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + vec3<f32>(33.33));
    return fract((p3.x + p3.y) * p3.z);
}

fn hash_noise(uv: vec2<f32>, time: f32) -> vec2<f32> {
    let n1 = fract(sin(dot(uv + vec2<f32>(time, 0.0), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let n2 = fract(sin(dot(uv + vec2<f32>(0.0, time), vec2<f32>(39.3468, 11.135))) * 24634.6345);
    return vec2<f32>(n1, n2);
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
    let base = textureSample(world_tex, source_sampler, input.uv);
    let mask_sample = textureSample(mask_tex, source_sampler, input.uv);
    let edge_sample = textureSample(edge_tex, source_sampler, input.uv);
    let reflection_color = textureSample(reflection_color_tex, source_sampler, input.uv);

    var coverage = luminance(mask_sample.rgb);
    if (uniforms.mask_invert > 0.5) {
        coverage = 1.0 - coverage;
    }
    coverage = clamp(coverage * mask_sample.a, 0.0, 1.0);

    let edge = pow(clamp(luminance(edge_sample.rgb), 0.0, 1.0), uniforms.edge_power);
    let perspective = mix(uniforms.background_strength, uniforms.foreground_strength, smoothstep(uniforms.horizon_y, 1.0, input.uv.y));
    coverage = clamp(pow(coverage, 0.45) * perspective, 0.0, 1.0);

    let noise = hash_noise(input.uv * uniforms.noise_scale, uniforms.time_seconds * uniforms.noise_speed);
    let ripple = sin((input.uv.y + uniforms.time_seconds * uniforms.ripple_speed) * 40.0) * uniforms.ripple_strength;
    let offset = (noise - vec2<f32>(0.5, 0.5)) * uniforms.distortion_px / uniforms.resolution;
    let reflection_uv = vec2<f32>(
        clamp(input.uv.x + offset.x * coverage, 0.0, 1.0),
        clamp(1.0 - input.uv.y + offset.y * coverage + ripple * coverage, 0.0, 1.0),
    );

    var blurred = textureSample(world_tex, source_sampler, reflection_uv);
    blurred += textureSample(world_tex, source_sampler, reflection_uv + vec2<f32>(1.0, 0.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv - vec2<f32>(1.0, 0.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv + vec2<f32>(0.0, 1.0) / uniforms.resolution * uniforms.blur_px);
    blurred += textureSample(world_tex, source_sampler, reflection_uv - vec2<f32>(0.0, 1.0) / uniforms.resolution * uniforms.blur_px);
    blurred *= 0.2;

    let light_source = mix(blurred.rgb, reflection_color.rgb, reflection_color.a);
    let light_strength = luminance(light_source);
    let specular = coverage * edge * light_strength * uniforms.specular_boost;
    let wet_mix = clamp(coverage * (0.92 + edge * 0.82), 0.0, 1.0);

    var final_rgb = mix(base.rgb, blurred.rgb, wet_mix);
    final_rgb += light_source * coverage * edge * uniforms.light_reflection_strength;
    final_rgb += vec3<f32>(specular * (1.0 + noise.x * uniforms.shimmer_strength * 2.0));
    let wet_brighten = vec3<f32>(0.06, 0.07, 0.08) + light_source * 0.12;
    final_rgb = mix(final_rgb, final_rgb * (1.0 - uniforms.wet_darken) + wet_brighten, coverage);

    return vec4<f32>(final_rgb, 1.0);
}
"#;


const REFRACTIVE_MATERIAL_SHADER: &str = r#"
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

const COLOR_QUANTIZE_SHADER: &str = r#"
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

const DOWNSCALE_SHADER: &str = r#"
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

const SHUTTER_BLUR_SHADER: &str = r#"
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

const DIRTY_BLOOM_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct DirtyBloomUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    threshold: f32,
    strength: f32,
    small_radius_px: f32,
    medium_radius_px: f32,
    large_radius_px: f32,
    dirty_noise: f32,
    halation_strength: f32,
    reflection_smear_x_px: f32,
    reflection_smear_y_px: f32,
    seed: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: DirtyBloomUniform;

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn bright_sample(uv: vec2<f32>) -> vec3<f32> {
    let color = textureSample(source_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
    let lift = smoothstep(uniforms.threshold, 1.0, luminance(color));
    return color * lift;
}

fn blur_ring(uv: vec2<f32>, radius_px: f32) -> vec3<f32> {
    let texel = vec2<f32>(1.0) / max(uniforms.resolution, vec2<f32>(1.0));
    let r = radius_px * texel;
    var color = bright_sample(uv) * 0.2;
    color += bright_sample(uv + vec2<f32>( r.x,  0.0)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x,  0.0)) * 0.1;
    color += bright_sample(uv + vec2<f32>( 0.0,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( 0.0, -r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( r.x,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x,  r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>( r.x, -r.y)) * 0.1;
    color += bright_sample(uv + vec2<f32>(-r.x, -r.y)) * 0.1;
    return color;
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
    let bloom_small = blur_ring(input.uv, uniforms.small_radius_px) * 0.45;
    let bloom_medium = blur_ring(input.uv, uniforms.medium_radius_px) * 0.65;
    let bloom_large = blur_ring(input.uv, uniforms.large_radius_px) * 0.35;
    let texel = vec2<f32>(1.0) / max(uniforms.resolution, vec2<f32>(1.0));
    let smear = (
        bright_sample(input.uv + vec2<f32>(uniforms.reflection_smear_x_px, uniforms.reflection_smear_y_px) * texel) +
        bright_sample(input.uv + vec2<f32>(-uniforms.reflection_smear_x_px, uniforms.reflection_smear_y_px) * texel) +
        bright_sample(input.uv + vec2<f32>(0.0, uniforms.reflection_smear_y_px * 1.6) * texel)
    ) * 0.18;
    let dirty = 1.0 - uniforms.dirty_noise + hash12(floor(input.uv * uniforms.resolution * 0.18) + vec2<f32>(uniforms.seed, floor(uniforms.time_seconds * 12.0))) * uniforms.dirty_noise;
    let hot = blur_ring(input.uv, 6.0) * smoothstep(0.78, 1.0, luminance(bright_sample(input.uv)));
    let halation = hot * vec3<f32>(1.0, 0.25, 0.35) * uniforms.halation_strength;
    let rgb = clamp(base.rgb + ((bloom_small + bloom_medium + bloom_large + smear) * dirty * uniforms.strength) + halation, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, base.a);
}
"#;

const HIGHLIGHT_EXTRACT_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct HighlightExtractUniform {
    resolution: vec2<f32>,
    threshold: f32,
    softness: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: HighlightExtractUniform;

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
    let softness = max(uniforms.softness, 0.0001);
    let lift = smoothstep(uniforms.threshold, uniforms.threshold + softness, luminance(base.rgb));
    let rgb = base.rgb * lift;
    return vec4<f32>(rgb, lift);
}
"#;

const CRT_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CrtUniform {
    resolution: vec2<f32>,
    time_seconds: f32,
    scanline_opacity: f32,
    scanline_frequency_px: f32,
    rgb_split_px: f32,
    curvature: f32,
    vignette: f32,
    phosphor_mask: f32,
    brightness_compensation: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: CrtUniform;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let centered = input.uv * 2.0 - vec2<f32>(1.0);
    let curve = dot(centered, centered) * uniforms.curvature;
    let uv = input.uv + centered * curve;
    let inside = step(0.0, uv.x) * step(uv.x, 1.0) * step(0.0, uv.y) * step(uv.y, 1.0);
    let split = vec2<f32>(uniforms.rgb_split_px / max(uniforms.resolution.x, 1.0), 0.0);
    let r = textureSample(source_tex, source_sampler, clamp(uv + split, vec2<f32>(0.0), vec2<f32>(1.0))).r;
    let g = textureSample(source_tex, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).g;
    let b = textureSample(source_tex, source_sampler, clamp(uv - split, vec2<f32>(0.0), vec2<f32>(1.0))).b;
    var rgb = vec3<f32>(r, g, b);
    let scanline = 1.0 - uniforms.scanline_opacity * (0.5 + 0.5 * sin(uv.y * uniforms.resolution.y * 6.2831853 / max(uniforms.scanline_frequency_px, 0.5)));
    let mask = 1.0 - uniforms.phosphor_mask * (0.5 + 0.5 * sin(uv.x * uniforms.resolution.x * 2.0943951));
    let vignette_distance = distance(input.uv, vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.35, 0.82, vignette_distance) * uniforms.vignette;
    rgb = clamp(rgb * scanline * mask * vignette * uniforms.brightness_compensation * inside, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, 1.0);
}
"#;

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        Self::new_with_device(&surface.device, surface.config.format)
    }

    pub fn new_for_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::new_with_device(&target.device, target.format)
    }

    fn new_with_device(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-color-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
        });
        let color_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let color_alpha_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[ColorVertex::layout()],
        );
        let color_additive_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-additive-pipeline",
            additive_blend_state(),
            &[ColorVertex::layout()],
        );
        let color_multiply_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-multiply-pipeline",
            multiply_blend_state(),
            &[ColorVertex::layout()],
        );
        let color_screen_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-screen-pipeline",
            screen_blend_state(),
            &[ColorVertex::layout()],
        );

        let common_layouts =
            crate::renderer::service::layout_builders::create_common_bind_group_layouts(device);
        let texture_bind_group_layout = common_layouts.texture;
        let camera_visual_source_bind_group_layout = common_layouts.camera_visual_source;
        let focus_blur_texture_bind_group_layout = common_layouts.focus_blur_texture;
        let shutter_blur_texture_bind_group_layout = common_layouts.shutter_blur_texture;
        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-texture-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TEXTURE_SHADER)),
        });
        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });
        let texture_alpha_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[TextureVertex::layout()],
        );
        let texture_opaque_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-opaque-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            &[TextureVertex::layout()],
        );
        let texture_additive_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-additive-pipeline",
            additive_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_multiply_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-multiply-pipeline",
            multiply_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_screen_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-screen-pipeline",
            screen_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_lighten_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-lighten-pipeline",
            lighten_blend_state(),
            &[TextureVertex::layout()],
        );

        let wet_reflections_texture_bind_group_layout = common_layouts.wet_reflections_texture;
        let wet_reflections_uniform_bind_group_layout = common_layouts.wet_reflections_uniform;
        let wet_reflections_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-wet-reflections-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WET_REFLECTIONS_SHADER)),
        });
        let plate_relight_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-plate-relight-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(PLATE_RELIGHT_SHADER)),
        });
        let wet_reflections_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-wet-reflections-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&wet_reflections_texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let color_quantize_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-color-quantize-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_QUANTIZE_SHADER)),
        });
        let downscale_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-downscale-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(DOWNSCALE_SHADER)),
        });
        let shutter_blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-shutter-blur-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHUTTER_BLUR_SHADER)),
        });
        let dirty_bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-dirty-bloom-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(DIRTY_BLOOM_SHADER)),
        });
        let highlight_extract_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-highlight-extract-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(HIGHLIGHT_EXTRACT_SHADER)),
        });
        let crt_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-crt-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(CRT_SHADER)),
        });
        let refractive_material_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-scene-refractive-material-shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(REFRACTIVE_MATERIAL_SHADER)),
            });
        let focus_blur_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-focus-blur-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&focus_blur_texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let post_fx_pipeline_ctx =
            crate::renderer::service::post_fx::pipelines::WgpuPostFxPipelineCreateContext {
                device,
                format,
                texture_bind_group_layout: &texture_bind_group_layout,
                camera_visual_source_bind_group_layout: &camera_visual_source_bind_group_layout,
                focus_blur_texture_bind_group_layout: &focus_blur_texture_bind_group_layout,
                shutter_blur_texture_bind_group_layout: &shutter_blur_texture_bind_group_layout,
                wet_reflections_texture_bind_group_layout: &wet_reflections_texture_bind_group_layout,
                wet_reflections_uniform_bind_group_layout: &wet_reflections_uniform_bind_group_layout,
                wet_reflections_pipeline_layout: &wet_reflections_pipeline_layout,
                focus_blur_pipeline_layout: &focus_blur_pipeline_layout,
                color_quantize_shader: &color_quantize_shader,
                downscale_shader: &downscale_shader,
                shutter_blur_shader: &shutter_blur_shader,
                dirty_bloom_shader: &dirty_bloom_shader,
                highlight_extract_shader: &highlight_extract_shader,
                crt_shader: &crt_shader,
                wet_reflections_shader: &wet_reflections_shader,
                plate_relight_shader: &plate_relight_shader,
                refractive_material_shader: &refractive_material_shader,
            };
        let mut pipelines = crate::renderer::service::pipeline_registry::WgpuPipelineRegistry::default();
        pipelines.extend(
            crate::renderer::service::post_fx::pipelines::build_default_post_fx_pipelines(
                &post_fx_pipeline_ctx,
            ),
        );

        Self {
            color_alpha_pipeline,
            color_additive_pipeline,
            color_multiply_pipeline,
            color_screen_pipeline,
            texture_alpha_pipeline,
            texture_opaque_pipeline,
            texture_additive_pipeline,
            texture_multiply_pipeline,
            texture_screen_pipeline,
            texture_lighten_pipeline,
            texture_bind_group_layout,
            camera_visual_source_bind_group_layout,
            focus_blur_texture_bind_group_layout,
            shutter_blur_texture_bind_group_layout,
            wet_reflections_texture_bind_group_layout,
            wet_reflections_uniform_bind_group_layout,
            pipelines,
            post_fx_executors: crate::renderer::service::post_fx::default_post_fx_executor_registry(),
            shutter_blur_runtimes: BTreeMap::new(),
            rain_glass_runtimes: BTreeMap::new(),
            texture_cache: BTreeMap::new(),
            lightmap_2d_image_cache: BTreeMap::new(),
            font_atlas_cache: BTreeMap::new(),
            font_fallback_warnings: BTreeSet::new(),
            frame_graph_executor: crate::renderer::graph::WgpuFrameGraphExecutor::default(),
            emergency_overlay_lines: Vec::new(),
            visual_source_targets_2d: crate::renderer::service::WgpuVisualSourceTargets2d::default(
            ),
            visual_source_previous_positions_2d: BTreeMap::new(),
            plate_relight_last_summary: "plate_relight: not run yet".to_owned(),
            render_materials_last_summary: "render.materials: not run yet".to_owned(),
        }
    }
}
