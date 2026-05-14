pub(crate) const RAIN_GLASS_STAMP_SHADER: &str = r#"
struct InstanceIn {
    @location(0) center_size: vec4<f32>,
    @location(1) params: vec4<f32>,
}

struct Uniforms {
    params0: vec4<f32>,
    params1: vec4<f32>,
    params2: vec4<f32>,
    params3: vec4<f32>,
    params4: vec4<f32>,
    params5: vec4<f32>,
    params6: vec4<f32>,
    params7: vec4<f32>,
    params8: vec4<f32>,
    params9: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) params: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

fn quad_pos(index: u32) -> vec2<f32> {
    let x = array<f32, 6>(-1.0, 1.0, 1.0, -1.0, 1.0, -1.0);
    let y = array<f32, 6>(-1.0, -1.0, 1.0, -1.0, 1.0, 1.0);
    return vec2<f32>(x[index], y[index]);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: InstanceIn) -> VertexOut {
    let local = quad_pos(vertex_index);
    let pixel = instance.center_size.xy + local * instance.center_size.zw;
    let uv = pixel / uniforms.params0.xy;
    var out: VertexOut;
    out.clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    out.local = local;
    out.params = instance.params;
    return out;
}

fn organic_radius(angle: f32, seed: f32, kind: f32) -> f32 {
    let h1 = sin(angle * 3.0 + seed * 17.0) * 0.05;
    let h2 = sin(angle * 7.0 + seed * 31.0) * 0.035;
    let h3 = sin(angle * 13.0 + seed * 71.0) * 0.015;
    let bottom = max(0.0, sin(angle));
    let gravity_bulb = pow(bottom, 2.0) * mix(0.10, 0.04, clamp(kind, 0.0, 1.0));
    return clamp(1.0 + h1 + h2 + h3 + gravity_bulb, 0.72, 1.35);
}

fn profile_noise(local: vec2<f32>, seed: f32) -> f32 {
    let n1 = sin(dot(local, vec2<f32>(12.9898, 78.233)) + seed * 43.13);
    let n2 = sin(dot(local, vec2<f32>(39.3467, 11.135)) + seed * 17.77);
    return n1 * 0.5 + n2 * 0.25;
}

fn droplet_thickness(local: vec2<f32>, seed: f32, kind: f32) -> f32 {
    let angle = atan2(local.y, local.x);
    let r = length(local) / organic_radius(angle, seed, kind);
    let mask = 1.0 - smoothstep(uniforms.params2.x, uniforms.params2.y, r);
    if (mask <= 0.0) {
        return 0.0;
    }

    // Reference-style droplet profile: a soft body with a stronger optical rim
    // and small asymmetric profile noise. This approximates the raindrop-fx
    // lookup texture without adding an asset dependency.
    let body = pow(max(0.0, 1.0 - r * r), 0.42);
    let inner_lens = pow(max(0.0, 1.0 - r), 0.18) * 0.42;
    let rim = exp(-pow((r - 0.78) * 5.5, 2.0)) * 0.20;
    let lower_pull = smoothstep(-0.20, 0.95, local.y) * smoothstep(0.15, 0.95, r) * 0.10;
    let grain = profile_noise(local * 2.1, seed) * 0.025;
    let profile = body * 0.72 + inner_lens + rim + lower_pull + grain;
    return clamp(profile * mask, 0.0, 1.0);
}

fn trail_thickness(local: vec2<f32>, seed: f32) -> f32 {
    let y = clamp(local.y, -0.96, 0.96);
    let axis = vec2<f32>(0.0, y);
    let d = length(local - axis);

    let along = smoothstep(-0.96, 0.96, local.y);

    // Trail is a thin water film, not a fat dark capsule.
    let taper = mix(0.78, 0.18, along);

    let wobble =
        1.0
        + sin((local.y + seed * 11.0) * 5.0) * 0.13
        + sin((local.y + seed * 29.0) * 13.0) * 0.055;

    let width = 0.24 * taper * wobble;

    let mask = 1.0 - smoothstep(width, width + 0.105, d);
    let inner = pow(max(0.0, 1.0 - d / max(width, 0.001)), 0.42);

    let breakup =
        0.70
        + sin(local.y * 19.0 + seed * 17.0) * 0.16
        + sin(local.y * 43.0 + seed * 5.0) * 0.08;

    return clamp(mask * inner * breakup, 0.0, 1.0);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let is_trail = input.params.w > 0.5 && input.params.w < 1.5;
    // Procedural stamp path: avoids external texture dependency and keeps
    // normals/depth fully generated in shader space.
    var thickness: f32;
    if (is_trail) {
        thickness = trail_thickness(input.local, input.params.z);
    } else {
        thickness = droplet_thickness(input.local, input.params.z, input.params.w);
    }
    let mask = thickness * input.params.x;
    if (mask <= 0.001) { discard; }
    let eps = 0.018;
    var txp: f32;
    var txm: f32;
    var typ: f32;
    var tym: f32;
    if (is_trail) {
        txp = trail_thickness(input.local + vec2<f32>(eps, 0.0), input.params.z);
        txm = trail_thickness(input.local - vec2<f32>(eps, 0.0), input.params.z);
        typ = trail_thickness(input.local + vec2<f32>(0.0, eps), input.params.z);
        tym = trail_thickness(input.local - vec2<f32>(0.0, eps), input.params.z);
    } else {
        txp = droplet_thickness(input.local + vec2<f32>(eps, 0.0), input.params.z, input.params.w);
        txm = droplet_thickness(input.local - vec2<f32>(eps, 0.0), input.params.z, input.params.w);
        typ = droplet_thickness(input.local + vec2<f32>(0.0, eps), input.params.z, input.params.w);
        tym = droplet_thickness(input.local - vec2<f32>(0.0, eps), input.params.z, input.params.w);
    }
    let hx = txp - txm;
    let hy = typ - tym;
    let normal_xy = clamp(
        vec2<f32>(hx, hy) * uniforms.params5.y + vec2<f32>(0.5),
        vec2<f32>(0.0),
        vec2<f32>(1.0)
    );
    var alpha_scale = 1.0;
    var depth_scale = 1.0;
    if (is_trail) {
        alpha_scale = uniforms.params6.w;
        depth_scale = uniforms.params6.z * 0.78;
    }
    let mass_depth = mix(0.55, 1.35, clamp(input.params.y, 0.0, 1.0));
    let alpha = clamp(thickness * input.params.x * alpha_scale, 0.0, 1.0);
    let depth = clamp(thickness * mass_depth * depth_scale, 0.0, 1.0);
    let encoded = vec3<f32>(normal_xy, depth);
    return vec4<f32>(encoded * alpha, alpha);
}
"#;

pub(crate) const RAIN_GLASS_FADE_SHADER: &str = r#"
struct Uniforms { params0: vec4<f32>, params1: vec4<f32>, params2: vec4<f32>, params3: vec4<f32>, params4: vec4<f32>, params5: vec4<f32>, params6: vec4<f32>, params7: vec4<f32>, params8: vec4<f32>, params9: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32> }
struct VertexOut { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32> }
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;
fn quad(index: u32) -> vec2<f32> { let x=array<f32,6>(-1.0,1.0,1.0,-1.0,1.0,-1.0); let y=array<f32,6>(-1.0,-1.0,1.0,-1.0,1.0,1.0); return vec2<f32>(x[index], y[index]); }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->VertexOut{ let p=quad(i); var o:VertexOut; o.clip_position=vec4<f32>(p,0.0,1.0); o.uv=p*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return o; }
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{ let dt=uniforms.diffuse.w; let fade=exp(-max(uniforms.params1.z,0.0)*dt*0.22); return textureSample(source_tex, source_sampler, input.uv)*fade; }
"#;

pub(crate) const RAIN_GLASS_ERASE_SHADER: &str = r#"
struct Uniforms { params0: vec4<f32>, params1: vec4<f32>, params2: vec4<f32>, params3: vec4<f32>, params4: vec4<f32>, params5: vec4<f32>, params6: vec4<f32>, params7: vec4<f32>, params8: vec4<f32>, params9: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32> }
struct VertexOut { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32> }
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var eraser_tex: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;
fn quad(index: u32) -> vec2<f32> { let x=array<f32,6>(-1.0,1.0,1.0,-1.0,1.0,-1.0); let y=array<f32,6>(-1.0,-1.0,1.0,-1.0,1.0,1.0); return vec2<f32>(x[index], y[index]); }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->VertexOut{ let p=quad(i); var o:VertexOut; o.clip_position=vec4<f32>(p,0.0,1.0); o.uv=p*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return o; }
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{ let src=textureSample(source_tex,source_sampler,input.uv); let erase=textureSample(eraser_tex,source_sampler,input.uv).a; let keep=1.0-smoothstep(uniforms.params9.x,uniforms.params9.y,erase); return src*keep; }
"#;

pub(crate) const RAIN_GLASS_MIST_SHADER: &str = r#"
struct Uniforms {
    params0: vec4<f32>,
    params1: vec4<f32>,
    params2: vec4<f32>,
    params3: vec4<f32>,
    params4: vec4<f32>,
    params5: vec4<f32>,
    params6: vec4<f32>,
    params7: vec4<f32>,
    params8: vec4<f32>,
    params9: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

fn quad(index: u32) -> vec2<f32> {
    let x = array<f32, 6>(-1.0, 1.0, 1.0, -1.0, 1.0, -1.0);
    let y = array<f32, 6>(-1.0, -1.0, 1.0, -1.0, 1.0, 1.0);
    return vec2<f32>(x[index], y[index]);
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    let p = quad(i);
    var o: VertexOut;
    o.clip_position = vec4<f32>(p, 0.0, 1.0);
    o.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return o;
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        v += noise(q) * a;
        q *= 2.03;
        a *= 0.5;
    }
    return v;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let enabled = uniforms.params7.y;
    if (enabled < 0.5) {
        return textureSample(source_tex, source_sampler, input.uv) * 0.985;
    }

    let dt = uniforms.diffuse.w;
    let mist_time = max(uniforms.params8.x, 0.001);
    let mist_blur_px = max(uniforms.params8.w, uniforms.params7.x);
    let px = uniforms.params0.zw;
    let step = px * mist_blur_px;
    let old_center = textureSample(source_tex, source_sampler, input.uv);

    var old_blur = old_center * 0.30;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv + vec2<f32>( step.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv - vec2<f32>( step.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv + vec2<f32>(0.0,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv - vec2<f32>(0.0,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv + vec2<f32>( step.x,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv + vec2<f32>(-step.x,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv + vec2<f32>( step.x, -step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;
    old_blur += textureSample(source_tex, source_sampler, clamp(input.uv + vec2<f32>(-step.x, -step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;

    let large = fbm(input.uv * vec2<f32>(3.2, 2.0) + vec2<f32>(0.17, 0.31));
    let medium = fbm(input.uv * vec2<f32>(9.0, 5.0) + vec2<f32>(0.71, 0.19));
    let condensation = smoothstep(0.30, 0.92, large * 0.72 + medium * 0.28);
    let add = condensation * dt / mist_time;
    let fade = exp(-dt * 0.035);
    let v = clamp(old_blur.r * fade + add, 0.0, 1.0);
    return vec4<f32>(v, v, v, v);
}
"#;

pub(crate) const RAIN_GLASS_BLUR_SHADER: &str = r#"
struct Uniforms { params0: vec4<f32>, params1: vec4<f32>, params2: vec4<f32>, params3: vec4<f32>, params4: vec4<f32>, params5: vec4<f32>, params6: vec4<f32>, params7: vec4<f32>, params8: vec4<f32>, params9: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32> }
struct VertexOut { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32> }
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;
@group(2) @binding(0) var<uniform> direction: vec4<f32>;
fn quad(index: u32) -> vec2<f32> { let x=array<f32,6>(-1.0,1.0,1.0,-1.0,1.0,-1.0); let y=array<f32,6>(-1.0,-1.0,1.0,-1.0,1.0,1.0); return vec2<f32>(x[index], y[index]); }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->VertexOut{ let p=quad(i); var o:VertexOut; o.clip_position=vec4<f32>(p,0.0,1.0); o.uv=p*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return o; }
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{ let step=direction.xy*uniforms.params2.w*uniforms.params0.zw; var c=textureSample(source_tex,source_sampler,input.uv)*0.34; c+=textureSample(source_tex,source_sampler,clamp(input.uv+step,vec2<f32>(0.0),vec2<f32>(1.0)))*0.24; c+=textureSample(source_tex,source_sampler,clamp(input.uv-step,vec2<f32>(0.0),vec2<f32>(1.0)))*0.24; c+=textureSample(source_tex,source_sampler,clamp(input.uv+step*2.0,vec2<f32>(0.0),vec2<f32>(1.0)))*0.09; c+=textureSample(source_tex,source_sampler,clamp(input.uv-step*2.0,vec2<f32>(0.0),vec2<f32>(1.0)))*0.09; return c; }
"#;

pub(crate) const RAIN_GLASS_COMPOSE_SHADER: &str = r#"
struct Uniforms {
    params0: vec4<f32>,
    params1: vec4<f32>,
    params2: vec4<f32>,
    params3: vec4<f32>,
    params4: vec4<f32>,
    params5: vec4<f32>,
    params6: vec4<f32>,
    params7: vec4<f32>,
    params8: vec4<f32>,
    params9: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var blurred_tex: texture_2d<f32>;
@group(0) @binding(2) var raindrop_tex: texture_2d<f32>;
@group(0) @binding(3) var droplet_tex: texture_2d<f32>;
@group(0) @binding(4) var trail_tex: texture_2d<f32>;
@group(0) @binding(5) var mist_tex: texture_2d<f32>;
@group(0) @binding(6) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

fn quad(index: u32) -> vec2<f32> {
    let x = array<f32, 6>(-1.0, 1.0, 1.0, -1.0, 1.0, -1.0);
    let y = array<f32, 6>(-1.0, -1.0, 1.0, -1.0, 1.0, 1.0);
    return vec2<f32>(x[index], y[index]);
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    let p = quad(i);
    var o: VertexOut;
    o.clip_position = vec4<f32>(p, 0.0, 1.0);
    o.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return o;
}

fn safe_uv(uv: vec2<f32>) -> vec2<f32> {
    return clamp(uv, vec2<f32>(0.001), vec2<f32>(0.999));
}

fn sample_scene(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(scene_tex, source_sampler, safe_uv(uv)).rgb;
}

fn sample_blur(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(blurred_tex, source_sampler, safe_uv(uv)).rgb;
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn unpremul_map(v: vec4<f32>) -> vec4<f32> {
    if (v.a <= 0.001) {
        return vec4<f32>(0.5, 0.5, 0.0, 0.0);
    }
    return vec4<f32>(v.rgb / v.a, v.a);
}

fn stronger_color(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    if (luma(a) >= luma(b)) {
        return a;
    }
    return b;
}

fn compose_optical(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    if (a.a <= 0.001) {
        return b;
    }
    if (b.a <= 0.001) {
        return a;
    }
    let wa = a.a;
    let wb = b.a;
    let coverage = 1.0 - (1.0 - wa) * (1.0 - wb);
    let rgb = (a.rgb * wa + b.rgb * wb) / max(wa + wb, 0.001);
    return vec4<f32>(rgb, coverage);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let debug = uniforms.params4.w;
    let original = textureSample(scene_tex, source_sampler, input.uv);
    let blurred = textureSample(blurred_tex, source_sampler, input.uv);
    let rain_raw = textureSample(raindrop_tex, source_sampler, input.uv);
    let droplets_raw = textureSample(droplet_tex, source_sampler, input.uv);
    let trails_raw = textureSample(trail_tex, source_sampler, input.uv);
    let mist_raw = textureSample(mist_tex, source_sampler, input.uv);

    let rain_dbg = unpremul_map(rain_raw);
    let droplets_dbg = unpremul_map(droplets_raw);
    let trails_dbg = unpremul_map(trails_raw);

    if (debug > 0.5 && debug < 1.5) { return original; }
    if (debug > 1.5 && debug < 2.5) { return blurred; }
    if (debug > 2.5 && debug < 3.5) { return vec4<f32>(rain_dbg.rgb, 1.0); }
    if (debug > 3.5 && debug < 4.5) { return vec4<f32>(droplets_dbg.rgb, 1.0); }
    if (debug > 4.5 && debug < 5.5) { return vec4<f32>(trails_dbg.rgb, 1.0); }
    if (debug > 7.5 && debug < 8.5) {
        let mist_dbg = mist_raw.r * uniforms.params4.y;
        return vec4<f32>(vec3<f32>(mist_dbg), 1.0);
    }

    let rain = rain_dbg;
    let droplets = droplets_dbg;
    let trails = trails_dbg;
    let droplet_layer = compose_optical(droplets, trails);
    let composed = compose_optical(rain, droplet_layer);
    let compose_a = composed.a;

    if (compose_a <= 0.001) {
        let mist_fog = smoothstep(0.015, 0.85, mist_raw.r) * uniforms.params4.y;
        let fog_blur = clamp(
            mist_fog * (0.42 + uniforms.params8.w * 0.030),
            0.0,
            0.82
        );
        var fogged = mix(original.rgb, blurred.rgb, fog_blur);
        let veil_color = vec3<f32>(0.62, 0.72, 0.82);
        fogged = mix(fogged, veil_color, clamp(mist_fog * uniforms.params8.y, 0.0, 0.18));
        return vec4<f32>(fogged, original.a);
    }

    let normal_encoded = composed.xy;
    let depth = composed.z;
    let normal_xy = (normal_encoded - vec2<f32>(0.5)) * 2.0;
    let mask = smoothstep(uniforms.params2.x, uniforms.params2.y, compose_a) * uniforms.params1.z;
    // Procedural stamps do not have the same RGBA semantics as the original
    // raindrop.png lookup texture, so they must use the transparent lens path.
    let reference_mode = false;

    if (debug > 5.5 && debug < 6.5) { return vec4<f32>(normal_encoded, depth, 1.0); }
    if (debug > 6.5 && debug < 7.5) { return vec4<f32>(vec3<f32>(compose_a), 1.0); }

    if (reference_mode) {
        let uv = safe_uv(
            input.uv - (normal_encoded - vec2<f32>(0.5))
            * vec2<f32>(depth * uniforms.params1.y + uniforms.params1.x)
        );
        let normal3 = normalize(vec3<f32>((normal_encoded - vec2<f32>(0.5)) * vec2<f32>(2.0), 1.0));
        let light_dir = normalize(uniforms.params3.xyz - uniforms.params3.w * vec3<f32>(input.uv, 0.0));
        let half_dir = normalize(light_dir + vec3<f32>(0.0, 0.0, 1.0));
        let lambert = clamp(dot(light_dir, normal3), 0.0, 1.0);
        let spec = pow(max(dot(normal3, half_dir), 0.0), uniforms.params4.x);
        let lit = textureSample(blurred_tex, source_sampler, uv).rgb
            + (lambert - uniforms.params2.z) * uniforms.diffuse.rgb
            + vec3<f32>(spec) * uniforms.specular.rgb;
        let mist = smoothstep(0.015, 0.85, mist_raw.r) * uniforms.params4.y;
        var base = blurred.rgb;
        base = mix(base, vec3<f32>(0.62, 0.72, 0.82), clamp(mist * uniforms.params8.y, 0.0, 0.18));
        return vec4<f32>(clamp(mix(base, lit, clamp(mask, 0.0, 1.0)), vec3<f32>(0.0), vec3<f32>(1.0)), original.a);
    }

    let distortion_uv = vec2<f32>(
        uniforms.params5.x * uniforms.params0.z,
        uniforms.params5.x * uniforms.params0.w
    );
    let refract_power = uniforms.params1.x + depth * uniforms.params1.y;
    let refraction_visibility = clamp(0.55 + depth * 1.35, 0.0, 2.25);
    let refract_offset = normal_xy * distortion_uv * refract_power * refraction_visibility;
    let refract_uv = safe_uv(input.uv - refract_offset);

    if (debug > 8.5 && debug < 9.5) {
        let mag = length(refract_offset * uniforms.params0.xy);
        return vec4<f32>(vec3<f32>(mag / max(uniforms.params5.x, 0.001)), 1.0);
    }

    let chroma = uniforms.params4.z;
    let chroma_offset = refract_offset * chroma * 0.35;
    let sharp = vec3<f32>(
        sample_scene(refract_uv + chroma_offset).r,
        sample_scene(refract_uv).g,
        sample_scene(refract_uv - chroma_offset).b
    );
    let soft = sample_blur(refract_uv);
    let droplet_focus = mask * smoothstep(0.04, 0.85, depth) * uniforms.params5.z;
    let mist = smoothstep(0.015, 0.85, mist_raw.r) * uniforms.params4.y;
    let blur_mix = clamp(
        droplet_focus + mist * (0.38 + uniforms.params8.w * 0.022),
        0.0,
        0.98
    );
    var water_color = mix(sharp, soft, blur_mix);

    let n = normalize(normal_xy + vec2<f32>(0.0001, 0.0001));
    let px = uniforms.params0.zw;
    let scene_center = sample_scene(input.uv);
    let scene_a = sample_scene(input.uv + n * 0.030 + px * 2.0);
    let scene_b = sample_scene(input.uv - n * 0.030 - px * 2.0);
    let scene_c = sample_scene(input.uv + vec2<f32>(-n.y, n.x) * 0.020);
    var neon_color = stronger_color(scene_a, scene_b);
    neon_color = stronger_color(neon_color, scene_c);

    let neon_luma = luma(neon_color);
    let center_luma = luma(scene_center);
    let scene_contrast =
        abs(luma(scene_a) - luma(scene_b)) +
        abs(luma(scene_a) - center_luma) +
        abs(luma(scene_c) - center_luma);
    let edge_factor = smoothstep(0.10, 0.86, 1.0 - depth);
    let coverage_factor = smoothstep(0.02, 0.70, compose_a);
    let rim =
        edge_factor *
        coverage_factor *
        smoothstep(0.20, 0.96, neon_luma) *
        (0.35 + smoothstep(0.01, 0.32, scene_contrast));
    let normal_rim =
        smoothstep(0.08, 0.75, length(normal_xy))
        * smoothstep(0.02, 0.85, compose_a)
        * (0.30 + smoothstep(0.18, 0.90, neon_luma));

    let final_rim = max(rim, normal_rim * 0.65);

    water_color += neon_color * final_rim * uniforms.params6.x * uniforms.params6.y;

    let normal3 = normalize(vec3<f32>(normal_xy * uniforms.params1.w, 1.0));
    let light_dir = normalize(uniforms.params3.xyz - uniforms.params3.w * vec3<f32>(input.uv, 0.0));
    let half_dir = normalize(light_dir + vec3<f32>(0.0, 0.0, 1.0));
    let lambert = clamp(dot(light_dir, normal3), 0.0, 1.0);
    let spec = pow(max(dot(normal3, half_dir), 0.0), uniforms.params4.x);
    let constant_light =
        ((lambert - uniforms.params2.z) * uniforms.diffuse.rgb +
         spec * uniforms.specular.rgb) *
        0.22 *
        mask;
    water_color += constant_light;

    let dark_response = clamp(1.0 - neon_luma, 0.0, 1.0);
    let shadow = final_rim * dark_response * 0.035 * uniforms.params2.z;
    water_color *= 1.0 - shadow;

    let base_blur = clamp(
        mist * (0.42 + uniforms.params8.w * 0.030),
        0.0,
        0.82
    );
    var base = mix(original.rgb, blurred.rgb, base_blur);
    base = mix(base, vec3<f32>(0.62, 0.72, 0.82), clamp(mist * uniforms.params8.y, 0.0, 0.18));
    let compose_mode = uniforms.params9.z;
    let body_mix = clamp(mask * uniforms.params5.w, 0.0, 1.0);

    // Treat water as a lens delta, not as an opaque dark body.
    let refracted_delta = water_color - original.rgb;

    // Large drops may sample darker scene areas, but they should not become black paint.
    let reference_lens_gain = mix(0.72, 0.92, compose_mode);
    var lens_rgb = base + refracted_delta * clamp(0.62 + depth * 0.56, 0.0, 1.25) * reference_lens_gain;

    // Keep water transparent. The body mix controls lens visibility, not opacity paint.
    lens_rgb = mix(base, lens_rgb, clamp(body_mix * mix(0.70, 0.86, compose_mode), 0.0, 0.88));

    // Darkness guard: a lens can darken locally, but should not become a black silhouette.
    let floor_rgb = base * (1.0 - mask * mix(0.16, 0.24, compose_mode));
    lens_rgb = max(lens_rgb, floor_rgb);

    // Re-apply scene-colored rim/highlight after transparency guard.
    lens_rgb += neon_color * final_rim * uniforms.params6.x * uniforms.params6.y * 0.38;

    return vec4<f32>(clamp(lens_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), original.a);
}
"#;
