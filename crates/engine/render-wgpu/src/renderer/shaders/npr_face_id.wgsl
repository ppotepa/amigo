struct GpuNprVertex3d {
    position: vec4<f32>,
}

struct GpuNprTriangle3d {
    indices: vec4<u32>,
    normal: vec4<f32>,
    material_id: u32,
    _pad0: vec3<u32>,
}

struct GpuNprFrameUniforms3d {
    model_translation: vec4<f32>,
    model_rotation: vec4<f32>,
    model_scale: vec4<f32>,
    camera_translation: vec4<f32>,
    camera_rotation: vec4<f32>,
    viewport_half: vec4<f32>,
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
    ink_color: vec4<f32>,
    seed: vec4<u32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) face_id: u32,
}

@group(0) @binding(0) var<storage, read> vertices: array<GpuNprVertex3d>;
@group(0) @binding(1) var<storage, read> triangles: array<GpuNprTriangle3d>;
@group(0) @binding(8) var<uniform> uniforms: GpuNprFrameUniforms3d;

fn rotate_euler(v: vec3<f32>, rotation: vec3<f32>) -> vec3<f32> {
    let cx = cos(rotation.x);
    let sx = sin(rotation.x);
    let cy = cos(rotation.y);
    let sy = sin(rotation.y);
    let cz = cos(rotation.z);
    let sz = sin(rotation.z);

    let rx = vec3<f32>(v.x, v.y * cx - v.z * sx, v.y * sx + v.z * cx);
    let ry = vec3<f32>(rx.x * cy + rx.z * sy, rx.y, -rx.x * sy + rx.z * cy);
    return vec3<f32>(ry.x * cz - ry.y * sz, ry.x * sz + ry.y * cz, ry.z);
}

fn rotate_inverse(v: vec3<f32>, rotation: vec3<f32>) -> vec3<f32> {
    let around_z = rotate_euler(v, vec3<f32>(0.0, 0.0, -rotation.z));
    let around_y = rotate_euler(around_z, vec3<f32>(0.0, -rotation.y, 0.0));
    return rotate_euler(around_y, vec3<f32>(-rotation.x, 0.0, 0.0));
}

fn transform_vertex(vertex_index: u32) -> vec3<f32> {
    let local = vertices[vertex_index].position.xyz * uniforms.model_scale.xyz;
    return rotate_euler(local, uniforms.model_rotation.xyz) + uniforms.model_translation.xyz;
}

fn triangle_world_points(face_index: u32) -> array<vec3<f32>, 3> {
    let triangle = triangles[face_index];
    let a = transform_vertex(triangle.indices.x);
    let b = transform_vertex(triangle.indices.y);
    let c = transform_vertex(triangle.indices.z);
    return array<vec3<f32>, 3>(a, b, c);
}

fn triangle_normal(face_index: u32) -> vec3<f32> {
    let world = triangle_world_points(face_index);
    return normalize(cross(world[1] - world[0], world[2] - world[0]));
}

fn triangle_center(face_index: u32) -> vec3<f32> {
    let world = triangle_world_points(face_index);
    let a = world[0];
    let b = world[1];
    let c = world[2];
    return (a + b + c) / 3.0;
}

fn triangle_front(face_index: u32) -> bool {
    let world_normal = triangle_normal(face_index);
    let to_camera = normalize(uniforms.camera_translation.xyz - triangle_center(face_index));
    return dot(world_normal, to_camera) > 0.0;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    let face_index = vertex_index / 3u;
    let corner_index = vertex_index % 3u;
    let triangle = triangles[face_index];
    let mesh_vertex_index = triangle.indices[corner_index];
    let world = transform_vertex(mesh_vertex_index);
    let camera_space = rotate_inverse(world - uniforms.camera_translation.xyz, uniforms.camera_rotation.xyz);
    let depth = -camera_space.z;
    let fov_y = uniforms.params0.x;
    let near_clip = uniforms.params0.y;
    let far_clip = uniforms.params0.z;
    let valid = depth > near_clip && depth < far_clip && triangle_front(face_index);
    let aspect = max(uniforms.viewport_half.x / max(uniforms.viewport_half.y, 0.0001), 0.0001);
    let inv_tan = 1.0 / tan(max(fov_y * 0.5, 0.001));
    let ndc = vec2<f32>(
        (camera_space.x * inv_tan / aspect) / max(depth, 0.0001),
        (camera_space.y * inv_tan) / max(depth, 0.0001),
    );
    let clip_depth = clamp((depth - near_clip) / max(far_clip - near_clip, 0.0001), 0.0, 1.0);
    var out: VertexOut;
    out.clip_position = select(
        vec4<f32>(2.0, 2.0, 1.0, 1.0),
        vec4<f32>(ndc, clip_depth, 1.0),
        valid,
    );
    out.face_id = face_index + 1u;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) u32 {
    return input.face_id;
}
