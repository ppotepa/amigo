pub(crate) const PLATE_RELIGHT_SHADER: &str = r#"
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
