pub(crate) const COLOR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(4) depth: f32,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, vertex.depth, 1.0);
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

pub(crate) const PENCIL_COLOR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>, @location(1) color: vec4<f32>,
    @location(2) stroke: vec4<f32>, @location(3) graphite: vec4<f32>,
    @location(4) depth: f32,
}
struct VertexOut {
    @builtin(position) clip_position: vec4<f32>, @location(0) color: vec4<f32>,
    @location(1) stroke: vec4<f32>, @location(2) graphite: vec4<f32>,
}
@vertex fn vs_main(v: VertexIn) -> VertexOut {
    var o: VertexOut;
    o.clip_position = vec4<f32>(v.position, v.depth, 1.0);
    o.color = v.color; o.stroke = v.stroke; o.graphite = v.graphite;
    return o;
}
fn hash(p: vec2<f32>) -> f32 {
    let h = sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453;
    return fract(h);
}
@fragment fn fs_main(i: VertexOut) -> @location(0) vec4<f32> {
    let grain = clamp(i.graphite.x, 0.0, 1.0);
    let hardness = clamp(i.graphite.y, 0.0, 1.0);
    let dryness = clamp(i.graphite.z, 0.0, 1.0);
    let pressure = clamp(i.stroke.z, 0.1, 1.6);
    // Material coordinates travel with the gesture. No screen coordinate or
    // wall-clock noise is involved in graphite deposition.
    let uv = vec2<f32>(i.stroke.x * 128.0, i.stroke.y * 5.0);
    let cell = floor(uv) + vec2<f32>(i.stroke.w);
    let tooth = hash(cell);
    let tooth_secondary = hash(cell + vec2<f32>(17.31, 5.17));
    let tooth_edge = hash(cell + vec2<f32>(-9.73, 23.41));
    let fiber = hash(vec2<f32>(floor(uv.x * 0.2), floor(uv.y * 2.0)) + vec2<f32>(i.stroke.w));
    // Graphite is deposited by several light contacts, not one binary alpha
    // lookup. Their weights vary with pressure, so a hard pass darkens and
    // closes the tooth while a soft pass leaves broken paper showing through.
    let contact_a = smoothstep(dryness * 0.55, 0.72 + pressure * 0.12, tooth + pressure * 0.20);
    let contact_b = smoothstep(dryness * 0.70, 0.86 + pressure * 0.08, tooth_secondary + pressure * 0.12)
        * (0.24 + pressure * 0.26);
    let contact_c = smoothstep(0.20 + dryness * 0.25, 0.92, tooth_edge + pressure * 0.08)
        * (0.10 + pressure * 0.16);
    let deposited = 1.0 - (1.0 - contact_a) * (1.0 - contact_b) * (1.0 - contact_c);
    // Integrate toward average coverage as a cell becomes subpixel.
    let footprint = max(fwidth(uv.x), fwidth(uv.y));
    let detail = 1.0 - smoothstep(0.7, 2.0, footprint);
    let pigment = mix(0.72, 0.18 + deposited * 0.82, detail);
    let aa = max(fwidth(i.stroke.y), 0.02);
    let edge = 1.0 - smoothstep(1.0 - aa * 0.5, 1.0 + aa * 0.5, abs(i.stroke.y));
    let shoulder = mix(0.35 + 0.65 * pow(max(0.0, 1.0 - abs(i.stroke.y)), 0.4), 1.0, hardness);
    let coverage = edge * shoulder * mix(1.0, pigment * (0.8 + fiber * 0.2), grain);
    return vec4<f32>(i.color.rgb, i.color.a * coverage);
}
"#;

pub(crate) const TEXTURE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(color_texture, color_sampler, input.uv) * input.color;
}
"#;
