pub(crate) const COLOR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return input.color;
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

pub(crate) const NPR_STROKE_SEGMENT_SHADER: &str = r#"
struct VertexIn {
    @location(0) start: vec2<f32>,
    @location(1) end: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) width_px: f32,
    @location(4) offset_px: f32,
    @location(5) overshoot_start_px: f32,
    @location(6) overshoot_end_px: f32,
    @location(7) viewport_half: vec2<f32>,
    @location(8) end_width_px: f32,
    @location(9) end_alpha: f32,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexIn, @builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var out: VertexOut;
    let tangent = vertex.end - vertex.start;
    let length = max(length(tangent), 0.000001);
    let direction = tangent / length;
    let normal = vec2<f32>(-direction.y, direction.x);
    let corner = vertex_index % 6u;
    let side = select(-1.0, 1.0, corner == 0u || corner == 1u || corner == 3u);
    let use_end = corner == 1u || corner == 2u || corner == 4u;
    let endpoint_sign = select(-1.0, 1.0, use_end);
    let overshoot_px = select(vertex.overshoot_start_px, vertex.overshoot_end_px, use_end);
    let anchor = select(vertex.start, vertex.end, use_end);
    let width_px = select(vertex.width_px, vertex.end_width_px, use_end);
    let alpha = select(vertex.color.a, vertex.end_alpha, use_end);
    let center = anchor
        + direction * overshoot_px * endpoint_sign
        + normal * vertex.offset_px;
    let expanded = center + normal * width_px * 0.5 * side;
    let clip = expanded / vertex.viewport_half;
    out.clip_position = vec4<f32>(clip, 0.0, 1.0);
    out.color = vec4<f32>(vertex.color.rgb, alpha);
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return input.color;
}
"#;
