struct GpuNprVertex3d {
    position: vec4<f32>,
}

struct GpuNprProjectedVertex3d {
    ndc_depth: vec4<f32>,
    screen: vec4<f32>,
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
    params14: vec4<f32>,
    params15: vec4<f32>,
    params16: vec4<f32>,
    params17: vec4<f32>,
    params18: vec4<f32>,
    params19: vec4<f32>,
    params20: vec4<f32>,
    ink_color: vec4<f32>,
    seed: vec4<u32>,
}

@group(0) @binding(0) var<storage, read> vertices: array<GpuNprVertex3d>;
@group(0) @binding(3) var<storage, read_write> projected_vertices: array<GpuNprProjectedVertex3d>;
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

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= u32(arrayLength(&vertices))) {
        return;
    }

    let local = vertices[index].position.xyz * uniforms.model_scale.xyz;
    let world = rotate_euler(local, uniforms.model_rotation.xyz) + uniforms.model_translation.xyz;
    let camera_space = rotate_inverse(world - uniforms.camera_translation.xyz, uniforms.camera_rotation.xyz);
    let depth = -camera_space.z;
    let fov_y = uniforms.params0.x;
    let near_clip = uniforms.params0.y;
    let far_clip = uniforms.params0.z;
    let valid = depth > near_clip && depth < far_clip;
    let aspect = max(uniforms.viewport_half.x / max(uniforms.viewport_half.y, 0.0001), 0.0001);
    let inv_tan = 1.0 / tan(max(fov_y * 0.5, 0.001));
    let ndc = vec2<f32>(
        (camera_space.x * inv_tan / aspect) / max(depth, 0.0001),
        (camera_space.y * inv_tan) / max(depth, 0.0001),
    );
    let clip_depth = clamp((depth - near_clip) / max(far_clip - near_clip, 0.0001), 0.0, 1.0);
    projected_vertices[index].ndc_depth = vec4<f32>(ndc, clip_depth, select(0.0, 1.0, valid));
    projected_vertices[index].screen = vec4<f32>(ndc * uniforms.viewport_half.xy, clip_depth, depth);
}
