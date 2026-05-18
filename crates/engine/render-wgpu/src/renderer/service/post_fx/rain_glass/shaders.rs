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
    params10: vec4<f32>,
    params11: vec4<f32>,
    params12: vec4<f32>,
    params13: vec4<f32>,
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

    if (kind > 1.5) {
        // Tiny droplets in raindrop-fx read as small optical beads. Keep them
        // bright and refractive without becoming flat gray disks.
        let core = pow(max(0.0, 1.0 - r * r), 0.58) * 0.58;
        let inner_lens = pow(max(0.0, 1.0 - r * 0.88), 0.30) * 0.18;
        let rim = exp(-pow((r - 0.72) * 6.4, 2.0)) * 0.24;
        let bead = core + inner_lens + rim + profile_noise(local * 3.4, seed) * 0.012;
        return clamp(bead * mask, 0.0, 0.82);
    }

    // Reference-style droplet profile: a soft body with a stronger optical rim
    // and small asymmetric profile noise. This approximates the raindrop-fx
    // lookup texture without adding an asset dependency.
    let body = pow(max(0.0, 1.0 - r * r), 0.36);
    let broad_lens = pow(max(0.0, 1.0 - r * 0.82), 0.72) * 0.34;
    let inner_lens = pow(max(0.0, 1.0 - r), 0.22) * 0.24;
    let meniscus = exp(-pow((r - 0.72) * 4.2, 2.0)) * 0.16;
    let rim = exp(-pow((r - 0.90) * 8.0, 2.0)) * 0.10;
    let lower_pull = smoothstep(-0.28, 0.98, local.y) * smoothstep(0.08, 0.96, r) * 0.13;
    let grain = profile_noise(local * 2.1, seed) * 0.018;
    let profile = body * 0.54 + broad_lens + inner_lens + meniscus + rim + lower_pull + grain;
    return clamp(profile * mask, 0.0, 1.0);
}

fn reference_drop_displacement(local: vec2<f32>, seed: f32, kind: f32) -> vec2<f32> {
    let angle = atan2(local.y, local.x);
    let r = length(local) / organic_radius(angle, seed, kind);
    let body = 1.0 - smoothstep(0.04, 1.02, r);
    let inner = 1.0 - smoothstep(0.10, 0.92, r);
    let lower = smoothstep(-0.25, 1.0, local.y) * smoothstep(0.08, 0.95, r);
    let wobble = vec2<f32>(
        sin(local.y * 5.3 + seed * 23.0) + sin(local.x * 11.0 + seed * 7.0) * 0.35,
        sin(local.x * 4.7 + seed * 19.0) + sin(local.y * 13.0 + seed * 5.0) * 0.25
    );

    // The raindrop-fx lookup carries displacement through the whole body.
    // Keep the center transparent but avoid a flat gray interior.
    let broad = 1.0 - smoothstep(0.0, 0.72, r);
    let radial = local * (0.075 + inner * 0.060 + broad * 0.045 + lower * 0.035);
    let gravity_pull = vec2<f32>(
        sin((local.y + seed) * 8.0) * lower * 0.018,
        lower * 0.060
    );
    let caustic = wobble * body * 0.016;
    let micro = vec2<f32>(
        sin(dot(local, vec2<f32>(18.1, 7.3)) + seed * 41.0),
        sin(dot(local, vec2<f32>(5.7, 21.9)) + seed * 13.0)
    ) * body * 0.008;

    return (radial + gravity_pull + caustic + micro) * body;
}

fn reference_trail_displacement(local: vec2<f32>, seed: f32) -> vec2<f32> {
    let y = clamp(local.y, -0.96, 0.96);
    let center_pull = 1.0 - smoothstep(0.0, 0.70, abs(local.x));
    let along = smoothstep(-0.96, 0.96, y);
    let head_join = smoothstep(0.04, 0.22, along);
    let taper = (1.0 - along * 0.40) * mix(0.84, 1.0, head_join);
    return vec2<f32>(
        local.x * 0.055 * taper,
        (sin(y * 10.0 + seed * 17.0) * 0.014 + taper * 0.024) * center_pull
    );
}

fn trail_thickness(local: vec2<f32>, seed: f32) -> f32 {
    let y = clamp(local.y, -0.96, 0.96);
    let axis = vec2<f32>(0.0, y);
    let d = length(local - axis);

    let along = smoothstep(-0.96, 0.96, local.y);
    let head_join = smoothstep(0.04, 0.22, along);

    // Reference streaks should stay visibly broad under the parent drop and
    // only narrow out near the tail.
    let taper = mix(1.24, 0.52, along) * mix(0.88, 1.0, head_join);

    let wobble =
        1.0
        + sin((local.y + seed * 11.0) * 5.0) * 0.13
        + sin((local.y + seed * 29.0) * 13.0) * 0.055;

    let width = 0.78 * taper * wobble;

    let mask = (1.0 - smoothstep(width, width + 0.120, d)) * mix(0.62, 1.0, head_join);
    let inner = pow(max(0.0, 1.0 - d / max(width, 0.001)), 0.28);

    let breakup =
        0.78
        + sin(local.y * 19.0 + seed * 17.0) * 0.18
        + sin(local.y * 43.0 + seed * 5.0) * 0.10;

    return clamp(mask * inner * breakup, 0.0, 1.0);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let is_trail = input.params.w > 0.5 && input.params.w < 1.5;
    let is_micro = input.params.w > 1.5;
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
    var displacement = vec2<f32>(hx, hy);
    if (uniforms.params9.w > 0.5) {
        if (is_trail) {
            displacement = reference_trail_displacement(input.local, input.params.z);
        } else {
            displacement = reference_drop_displacement(input.local, input.params.z, input.params.w);
        }
    }
    if (is_micro) {
        displacement *= 0.92;
    }
    let normal_xy = clamp(
        displacement * uniforms.params5.y + vec2<f32>(0.5),
        vec2<f32>(0.0),
        vec2<f32>(1.0)
    );
    var alpha_scale = 1.0;
    var depth_scale = 1.0;
    if (is_trail) {
        alpha_scale = uniforms.params6.w;
        depth_scale = uniforms.params6.z * 0.78;
    } else if (is_micro) {
        alpha_scale = 0.96;
        depth_scale = 0.46;
    }
    let mass_depth = mix(0.55, 1.35, clamp(input.params.y, 0.0, 1.0));
    let alpha = clamp(thickness * input.params.x * alpha_scale, 0.0, 1.0);
    let depth = clamp(thickness * mass_depth * depth_scale, 0.0, 1.0);
    let encoded = vec3<f32>(normal_xy, depth);
    return vec4<f32>(encoded * alpha, alpha);
}
"#;

pub(crate) const RAIN_GLASS_FADE_SHADER: &str = r#"
struct Uniforms { params0: vec4<f32>, params1: vec4<f32>, params2: vec4<f32>, params3: vec4<f32>, params4: vec4<f32>, params5: vec4<f32>, params6: vec4<f32>, params7: vec4<f32>, params8: vec4<f32>, params9: vec4<f32>, params10: vec4<f32>, params11: vec4<f32>, params12: vec4<f32>, params13: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32> }
struct VertexOut { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32> }
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;
fn quad(index: u32) -> vec2<f32> { let x=array<f32,6>(-1.0,1.0,1.0,-1.0,1.0,-1.0); let y=array<f32,6>(-1.0,-1.0,1.0,-1.0,1.0,1.0); return vec2<f32>(x[index], y[index]); }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->VertexOut{ let p=quad(i); var o:VertexOut; o.clip_position=vec4<f32>(p,0.0,1.0); o.uv=p*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return o; }
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{ let dt=uniforms.diffuse.w; let fade=exp(-max(uniforms.params1.z,0.0)*dt*0.22); return textureSample(source_tex, source_sampler, input.uv)*fade; }
"#;

pub(crate) const RAIN_GLASS_ERASE_SHADER: &str = r#"
struct Uniforms { params0: vec4<f32>, params1: vec4<f32>, params2: vec4<f32>, params3: vec4<f32>, params4: vec4<f32>, params5: vec4<f32>, params6: vec4<f32>, params7: vec4<f32>, params8: vec4<f32>, params9: vec4<f32>, params10: vec4<f32>, params11: vec4<f32>, params12: vec4<f32>, params13: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32> }
struct VertexOut { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32> }
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var eraser_tex: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;
fn quad(index: u32) -> vec2<f32> { let x=array<f32,6>(-1.0,1.0,1.0,-1.0,1.0,-1.0); let y=array<f32,6>(-1.0,-1.0,1.0,-1.0,1.0,1.0); return vec2<f32>(x[index], y[index]); }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->VertexOut{ let p=quad(i); var o:VertexOut; o.clip_position=vec4<f32>(p,0.0,1.0); o.uv=p*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return o; }
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{ let src=textureSample(source_tex,source_sampler,input.uv); let erase_raw=textureSample(eraser_tex,source_sampler,input.uv).a; let erase=clamp(erase_raw*1.28,0.0,1.0); let keep=1.0-smoothstep(uniforms.params9.x*0.82,uniforms.params9.y*0.96,erase); return src*keep; }
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
    params10: vec4<f32>,
    params11: vec4<f32>,
    params12: vec4<f32>,
    params13: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var mist_prev_tex: texture_2d<f32>;
@group(0) @binding(1) var raindrop_tex: texture_2d<f32>;
@group(0) @binding(2) var droplet_tex: texture_2d<f32>;
@group(0) @binding(3) var streak_tex: texture_2d<f32>;
@group(0) @binding(4) var source_sampler: sampler;
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

fn coverage(v: vec4<f32>) -> f32 {
    return clamp(v.a, 0.0, 1.0);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let enabled = uniforms.params7.y;
    if (enabled < 0.5) {
        return textureSample(mist_prev_tex, source_sampler, input.uv) * 0.985;
    }

    let dt = uniforms.diffuse.w;
    let mist_time = max(uniforms.params8.x, 0.001);
    let mist_blur_px = max(uniforms.params8.w, uniforms.params7.x);
    let px = uniforms.params0.zw;
    let step = px * mist_blur_px;
    let old_center = textureSample(mist_prev_tex, source_sampler, input.uv);

    var old_blur = old_center * 0.30;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv + vec2<f32>( step.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv - vec2<f32>( step.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv + vec2<f32>(0.0,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv - vec2<f32>(0.0,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.115;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv + vec2<f32>( step.x,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv + vec2<f32>(-step.x,  step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv + vec2<f32>( step.x, -step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;
    old_blur += textureSample(mist_prev_tex, source_sampler, clamp(input.uv + vec2<f32>(-step.x, -step.y), vec2<f32>(0.0), vec2<f32>(1.0))) * 0.060;

    let large = fbm(input.uv * vec2<f32>(3.2, 2.0) + vec2<f32>(0.17, 0.31));
    let medium = fbm(input.uv * vec2<f32>(9.0, 5.0) + vec2<f32>(0.71, 0.19));
    let condensation = smoothstep(0.62, 0.96, large * 0.76 + medium * 0.24) * 0.11;
    let rain = coverage(textureSample(raindrop_tex, source_sampler, input.uv));
    let droplets = coverage(textureSample(droplet_tex, source_sampler, input.uv));
    let streaks = coverage(textureSample(streak_tex, source_sampler, input.uv));
    let water = max(max(rain, droplets * 0.65), streaks * 0.85);
    let mist_strength = max(uniforms.params8.y, 0.02) * max(uniforms.params4.y, 0.35);
    let add = (water * 0.92 + condensation * 0.42) * mist_strength * dt * 1.35 / mist_time;
    let fade = exp(-dt / (mist_time * 0.92));
    let v = clamp(old_blur.r * fade + add, 0.0, 1.0);
    return vec4<f32>(v, v, v, v);
}
"#;

pub(crate) const RAIN_GLASS_BLUR_SHADER: &str = r#"
struct Uniforms { params0: vec4<f32>, params1: vec4<f32>, params2: vec4<f32>, params3: vec4<f32>, params4: vec4<f32>, params5: vec4<f32>, params6: vec4<f32>, params7: vec4<f32>, params8: vec4<f32>, params9: vec4<f32>, params10: vec4<f32>, params11: vec4<f32>, params12: vec4<f32>, params13: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32> }
struct VertexOut { @builtin(position) clip_position: vec4<f32>, @location(0) uv: vec2<f32> }
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;
@group(2) @binding(0) var<uniform> direction: vec4<f32>;
fn quad(index: u32) -> vec2<f32> { let x=array<f32,6>(-1.0,1.0,1.0,-1.0,1.0,-1.0); let y=array<f32,6>(-1.0,-1.0,1.0,-1.0,1.0,1.0); return vec2<f32>(x[index], y[index]); }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->VertexOut{ let p=quad(i); var o:VertexOut; o.clip_position=vec4<f32>(p,0.0,1.0); o.uv=p*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return o; }
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{ let radius_px=max(uniforms.params2.w, uniforms.params8.z*2.0); let step=direction.xy*radius_px*uniforms.params0.zw; var c=textureSample(source_tex,source_sampler,input.uv)*0.28; c+=textureSample(source_tex,source_sampler,clamp(input.uv+step,vec2<f32>(0.0),vec2<f32>(1.0)))*0.22; c+=textureSample(source_tex,source_sampler,clamp(input.uv-step,vec2<f32>(0.0),vec2<f32>(1.0)))*0.22; c+=textureSample(source_tex,source_sampler,clamp(input.uv+step*2.0,vec2<f32>(0.0),vec2<f32>(1.0)))*0.10; c+=textureSample(source_tex,source_sampler,clamp(input.uv-step*2.0,vec2<f32>(0.0),vec2<f32>(1.0)))*0.10; c+=textureSample(source_tex,source_sampler,clamp(input.uv+step*3.5,vec2<f32>(0.0),vec2<f32>(1.0)))*0.04; c+=textureSample(source_tex,source_sampler,clamp(input.uv-step*3.5,vec2<f32>(0.0),vec2<f32>(1.0)))*0.04; return c; }
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
    params10: vec4<f32>,
    params11: vec4<f32>,
    params12: vec4<f32>,
    params13: vec4<f32>,
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
@group(0) @binding(6) var normal_tex: texture_2d<f32>;
@group(0) @binding(7) var wetness_tex: texture_2d<f32>;
@group(0) @binding(8) var highlight_tex: texture_2d<f32>;
@group(0) @binding(9) var emissive_tex: texture_2d<f32>;
@group(0) @binding(10) var source_sampler: sampler;
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

fn sample_highlight_source(uv: vec2<f32>) -> vec3<f32> {
    return max(
        textureSample(highlight_tex, source_sampler, safe_uv(uv)).rgb,
        textureSample(emissive_tex, source_sampler, safe_uv(uv)).rgb
    );
}

fn sample_material_normal(uv: vec2<f32>) -> vec2<f32> {
    return textureSample(normal_tex, source_sampler, safe_uv(uv)).xy * 2.0 - vec2<f32>(1.0);
}

fn sample_material_wetness(uv: vec2<f32>) -> f32 {
    let wet = textureSample(wetness_tex, source_sampler, safe_uv(uv)).rgb;
    return clamp(dot(wet, vec3<f32>(0.18, 0.50, 0.32)) * 2.4, 0.0, 1.0);
}

fn sample_optical_blurred(tex: texture_2d<f32>, uv: vec2<f32>, blur_px: f32) -> vec4<f32> {
    if (blur_px <= 0.001) {
        return textureSample(tex, source_sampler, uv);
    }

    let offset = vec2<f32>(uniforms.params0.z * blur_px, uniforms.params0.w * blur_px);
    let center = textureSample(tex, source_sampler, uv) * 0.40;
    let a = textureSample(tex, source_sampler, safe_uv(uv + vec2<f32>( offset.x, 0.0))) * 0.15;
    let b = textureSample(tex, source_sampler, safe_uv(uv + vec2<f32>(-offset.x, 0.0))) * 0.15;
    let c = textureSample(tex, source_sampler, safe_uv(uv + vec2<f32>(0.0,  offset.y))) * 0.15;
    let d = textureSample(tex, source_sampler, safe_uv(uv + vec2<f32>(0.0, -offset.y))) * 0.15;
    return center + a + b + c + d;
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

fn screen_compose(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    if (a.a <= 0.001) {
        return b;
    }
    if (b.a <= 0.001) {
        return a;
    }
    let rgb = a.rgb + b.rgb - vec3<f32>(2.0) * a.rgb * b.rgb;
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), max(a.a, b.a));
}

fn scene_tint(color: vec3<f32>) -> vec3<f32> {
    let peak = max(max(color.r, color.g), max(color.b, 0.0001));
    return clamp(color / peak, vec3<f32>(0.0), vec3<f32>(1.35));
}

fn apply_scene_light(scene_color: vec3<f32>, effect_color: vec3<f32>, mask: f32, depth: f32) -> vec3<f32> {
    if (uniforms.params10.x < 0.5 || uniforms.params6.x <= 0.001) {
        return effect_color;
    }

    let light_luma = luma(scene_color);
    let visibility = mix(
        uniforms.params10.z,
        1.0,
        clamp(light_luma * (0.65 + uniforms.params6.x), 0.0, 1.0)
    );
    let tint = mix(vec3<f32>(1.0), scene_tint(scene_color), uniforms.params10.y);
    let glow = scene_color * uniforms.params6.x * (0.16 + depth * 0.34);
    let shaded = effect_color * visibility * tint;
    let glow_mix = clamp(mask * (0.42 + depth * 0.26), 0.0, 1.0);
    return clamp(
        mix(shaded, stronger_color(shaded, shaded + glow), glow_mix),
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );
}

fn blood_color(depth: f32) -> vec3<f32> {
    let fresh = uniforms.params11.rgb;
    let dark = vec3<f32>(
        fresh.r * 0.42,
        fresh.g * 0.34,
        fresh.b * 0.30
    );
    let thickness = smoothstep(0.08, 0.92, depth);
    return mix(fresh, dark, thickness);
}

fn apply_blood_lens(lens_rgb: vec3<f32>, mask: f32, depth: f32) -> vec3<f32> {
    let blood_amount = clamp(uniforms.params11.w, 0.0, 1.0);
    if (blood_amount <= 0.001) {
        return lens_rgb;
    }

    let blood_mask = clamp(mask * blood_amount, 0.0, 1.0);
    let tint = blood_color(depth);
    let tinted = lens_rgb * vec3<f32>(0.70, 0.24, 0.22) + tint * (0.28 + depth * 0.32);
    return clamp(mix(lens_rgb, tinted, blood_mask), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn apply_blood_trail_smear(
    scene_color: vec3<f32>,
    current_rgb: vec3<f32>,
    trail_map: vec4<f32>,
    scene_light_response: f32
) -> vec3<f32> {
    let blood_amount = clamp(uniforms.params11.w, 0.0, 1.0);
    if (blood_amount <= 0.001 || trail_map.a <= 0.001) {
        return current_rgb;
    }

    let trail_mask = clamp(trail_map.a * blood_amount * 1.18, 0.0, 1.0);
    let normal_mag = length((trail_map.xy - vec2<f32>(0.5)) * 2.0);
    let rim = smoothstep(0.10, 0.48, normal_mag) * (1.0 - smoothstep(0.52, 0.92, normal_mag));
    let body = trail_mask * (1.0 - rim * 0.82);
    let gloss = smoothstep(0.16, 0.55, normal_mag) * (1.0 - smoothstep(0.60, 0.95, normal_mag));

    let dark_blood = vec3<f32>(0.020, 0.002, 0.002);
    let thick_blood = vec3<f32>(0.055, 0.005, 0.004);
    let stain = mix(dark_blood, thick_blood, clamp(trail_map.z * 0.45, 0.0, 1.0));

    let light_tint = scene_tint(scene_color) * scene_light_response;
    let reflective_rim =
        stronger_color(
            current_rgb,
            current_rgb + light_tint * rim * trail_mask * (0.06 + uniforms.params10.y * 0.10)
        );
    let glossy_sheen =
        stronger_color(
            reflective_rim,
            reflective_rim + light_tint * gloss * trail_mask * 0.035
        );

    let smeared = mix(current_rgb, stain, clamp(body * 0.92, 0.0, 0.92));
    return clamp(
        mix(smeared, glossy_sheen, clamp((rim * 0.82 + gloss * 0.38) * trail_mask, 0.0, 1.0)),
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );
}

fn darken_scene(color: vec3<f32>) -> vec3<f32> {
    return color * (1.0 - clamp(uniforms.params12.x, 0.0, 1.0));
}

fn z_depth_focus_amount() -> f32 {
    if (uniforms.params12.y <= 0.5) {
        return 0.0;
    }
    let z_depth = uniforms.params12.z;
    let focus_depth = uniforms.params12.w;
    let focus_width = max(uniforms.params13.x, 0.001);
    let blur_scale = uniforms.params13.y;
    let response = uniforms.params13.z;
    let z_gap = abs(z_depth - focus_depth);
    return smoothstep(focus_width * 0.45, focus_width * 2.2, z_gap) * blur_scale * response;
}

fn reference_background(blurred: vec4<f32>, mist_raw: vec4<f32>) -> vec3<f32> {
    let mist = smoothstep(0.015, 0.85, mist_raw.r) * uniforms.params4.y;
    let mist_strength = max(uniforms.params8.y, 0.02);
    var base = mix(
        darken_scene(blurred.rgb),
        vec3<f32>(0.62, 0.72, 0.82),
        clamp(mist * (0.10 + mist_strength * 4.4), 0.0, 0.16)
    );
    return base;
}

fn reference_compose_color(
    uv0: vec2<f32>,
    original: vec4<f32>,
    composed: vec4<f32>,
    blurred: vec4<f32>,
    mist_raw: vec4<f32>,
    trails_dbg: vec4<f32>
) -> vec4<f32> {
    let scene_blend = uniforms.params7.z;
    let mask = smoothstep(uniforms.params2.x, uniforms.params2.y, composed.a) * uniforms.params1.z;
    let refract_strength = (composed.z * uniforms.params1.y + uniforms.params1.x) * 0.62;
    let refract_uv = safe_uv(uv0 - (composed.xy - vec2<f32>(0.5)) * vec2<f32>(refract_strength));

    let normal3 = normalize(vec3<f32>((composed.xy - vec2<f32>(0.5)) * vec2<f32>(2.0), 1.0));
    let light_dir = normalize(uniforms.params3.xyz - uniforms.params3.w * vec3<f32>(uv0, 0.0));
    let half_dir = normalize(light_dir + vec3<f32>(0.0, 0.0, 1.0));
    let lambertian = clamp(dot(light_dir, normal3), 0.0, 1.0);
    let blinn_phong = pow(max(dot(normal3, half_dir), 0.0), max(uniforms.params4.x, 1.0));

    let chroma = uniforms.params4.z;
    let chroma_offset = (composed.xy - vec2<f32>(0.5)) * vec2<f32>(refract_strength * chroma * 0.18);
    var lit = vec3<f32>(
        textureSample(blurred_tex, source_sampler, safe_uv(refract_uv + chroma_offset)).r,
        textureSample(blurred_tex, source_sampler, refract_uv).g,
        textureSample(blurred_tex, source_sampler, safe_uv(refract_uv - chroma_offset)).b
    );
    lit += (lambertian - uniforms.params2.z) * uniforms.diffuse.rgb;
    lit += vec3<f32>(blinn_phong) * uniforms.specular.rgb;

    let base = reference_background(blurred, mist_raw);
    var effect_rgb = clamp(mix(base, lit, clamp(mask, 0.0, 1.0)), vec3<f32>(0.0), vec3<f32>(1.0));
    effect_rgb = apply_scene_light(
        max(textureSample(scene_tex, source_sampler, refract_uv).rgb, sample_highlight_source(refract_uv)),
        effect_rgb,
        mask,
        composed.z
    );
    effect_rgb = apply_blood_lens(effect_rgb, mask, composed.z);
    effect_rgb = apply_blood_trail_smear(
        max(textureSample(scene_tex, source_sampler, refract_uv).rgb, sample_highlight_source(refract_uv)),
        effect_rgb,
        trails_dbg,
        uniforms.params6.x
    );
    return vec4<f32>(mix(darken_scene(original.rgb), effect_rgb, scene_blend), 1.0);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let debug = uniforms.params4.w;
    let original = textureSample(scene_tex, source_sampler, input.uv);
    let blurred = textureSample(blurred_tex, source_sampler, input.uv);
    let z_focus = z_depth_focus_amount();
    let z_focus_blur_px = clamp(z_focus * 5.5, 0.0, 12.0);
    let rain_raw = sample_optical_blurred(raindrop_tex, input.uv, uniforms.params10.w + z_focus_blur_px);
    let droplets_raw = sample_optical_blurred(droplet_tex, input.uv, uniforms.params10.w + z_focus_blur_px);
    let trails_raw = sample_optical_blurred(trail_tex, input.uv, uniforms.params10.w + z_focus_blur_px);
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
    let rain_cover = smoothstep(uniforms.params2.x, uniforms.params2.y, rain_dbg.a);
    let trail_under = max(0.0, 1.0 - rain_cover * 0.92);
    let trails = vec4<f32>(trails_dbg.rgb * trail_under, trails_dbg.a * trail_under);
    let reference_mode = uniforms.params9.w > 0.5;

    var droplet_layer: vec4<f32>;
    var composed: vec4<f32>;
    if (reference_mode) {
        droplet_layer = droplets;
        composed = screen_compose(rain, droplet_layer);
    } else {
        droplet_layer = compose_optical(droplets, trails);
        composed = compose_optical(rain, droplet_layer);
    }
    let compose_a = composed.a;

    if (compose_a <= 0.001) {
        if (reference_mode) {
            return vec4<f32>(mix(darken_scene(original.rgb), reference_background(blurred, mist_raw), uniforms.params7.z), original.a);
        }
        let mist_fog = smoothstep(0.015, 0.85, mist_raw.r) * uniforms.params4.y;
        let fog_blur = clamp(
            mist_fog * (0.16 + uniforms.params8.w * 0.018),
            0.0,
            0.38
        );
        var fogged = mix(darken_scene(original.rgb), darken_scene(blurred.rgb), fog_blur);
        let veil_color = vec3<f32>(0.62, 0.72, 0.82);
        fogged = mix(fogged, veil_color, clamp(mist_fog * uniforms.params8.y * 0.42, 0.0, 0.08));
        return vec4<f32>(fogged, original.a);
    }

    let normal_encoded = composed.xy;
    let depth = composed.z;
    let normal_xy = (normal_encoded - vec2<f32>(0.5)) * 2.0;
    let mask = smoothstep(uniforms.params2.x, uniforms.params2.y, compose_a) * uniforms.params1.z;

    if (debug > 5.5 && debug < 6.5) { return vec4<f32>(normal_encoded, depth, 1.0); }
    if (debug > 6.5 && debug < 7.5) { return vec4<f32>(vec3<f32>(compose_a), 1.0); }

    if (reference_mode) {
        if (debug > 8.5 && debug < 9.5) {
            let refract_strength = depth * uniforms.params1.y + uniforms.params1.x;
            let reference_offset = (normal_encoded - vec2<f32>(0.5)) * vec2<f32>(refract_strength);
            return vec4<f32>(vec3<f32>(length(reference_offset)), 1.0);
        }
        return reference_compose_color(input.uv, original, composed, blurred, mist_raw, trails_dbg);
    }

    let distortion_uv = vec2<f32>(
        uniforms.params5.x * uniforms.params0.z,
        uniforms.params5.x * uniforms.params0.w
    );
    let refract_power = uniforms.params1.x + depth * uniforms.params1.y;
    let refraction_visibility = clamp(0.55 + depth * 1.35, 0.0, 2.25);
    let material_normal = sample_material_normal(input.uv);
    let material_wetness = sample_material_wetness(input.uv);
    let combined_normal = normalize(normal_xy + material_normal * (0.20 + material_wetness * 0.55));
    let refract_offset = combined_normal * distortion_uv * refract_power * refraction_visibility * (1.0 + material_wetness * 0.32);
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
        droplet_focus + z_focus * 0.72 + mist * (0.16 + uniforms.params8.w * 0.012),
        0.0,
        0.86
    );
    let optical_source = sample_highlight_source(refract_uv);
    var water_color = mix(max(sharp, optical_source * 0.70), max(soft, optical_source * 0.42), blur_mix);

    let n = normalize(combined_normal + vec2<f32>(0.0001, 0.0001));
    let px = uniforms.params0.zw;
    let scene_center = max(sample_scene(input.uv), sample_highlight_source(input.uv));
    let scene_a = max(sample_scene(input.uv + n * 0.030 + px * 2.0), sample_highlight_source(input.uv + n * 0.030 + px * 2.0));
    let scene_b = max(sample_scene(input.uv - n * 0.030 - px * 2.0), sample_highlight_source(input.uv - n * 0.030 - px * 2.0));
    let scene_c = max(sample_scene(input.uv + vec2<f32>(-n.y, n.x) * 0.020), sample_highlight_source(input.uv + vec2<f32>(-n.y, n.x) * 0.020));
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
        smoothstep(0.08, 0.75, length(combined_normal))
        * smoothstep(0.02, 0.85, compose_a)
        * (0.30 + smoothstep(0.18, 0.90, neon_luma));

    let final_rim = max(rim, normal_rim * 0.65);

    water_color += neon_color * final_rim * uniforms.params6.x * uniforms.params6.y * (1.0 + material_wetness * 0.45);

    let normal3 = normalize(vec3<f32>(combined_normal * uniforms.params1.w, 1.0));
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
        mist * (0.16 + uniforms.params8.w * 0.018),
        0.0,
        0.36
    );
    var base = mix(darken_scene(original.rgb), darken_scene(blurred.rgb), base_blur);
    base = mix(base, vec3<f32>(0.62, 0.72, 0.82), clamp(mist * uniforms.params8.y * 0.42, 0.0, 0.08));
    let compose_mode = uniforms.params9.z;
    let body_mix = clamp(mask * uniforms.params5.w * (1.0 + material_wetness * 0.28), 0.0, 1.0);

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
    lens_rgb = apply_blood_lens(lens_rgb, mask, depth);
    lens_rgb = apply_blood_trail_smear(
        max(textureSample(scene_tex, source_sampler, refract_uv).rgb, sample_highlight_source(refract_uv)),
        lens_rgb,
        trails_dbg,
        uniforms.params6.x
    );

    let final_effect = apply_scene_light(
        max(textureSample(scene_tex, source_sampler, refract_uv).rgb, sample_highlight_source(refract_uv)),
        clamp(lens_rgb, vec3<f32>(0.0), vec3<f32>(1.0)),
        mask,
        depth
    );
    return vec4<f32>(mix(darken_scene(original.rgb), final_effect, uniforms.params7.z), original.a);
}
"#;
