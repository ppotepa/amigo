use amigo_math::{Transform3, Vec3};

use crate::renderer::CachedMeshGeometry3d;

pub(crate) fn camera_response_distances(
    response: amigo_render_api::NprCameraResponse3d,
    geometry: &CachedMeshGeometry3d,
    camera: Transform3,
    transform: Transform3,
) -> (f32, f32, f32) {
    if !response.auto_focus {
        return (response.near_distance, response.far_distance, 0.0);
    }

    let (local_center, local_radius) = mesh_local_center_and_radius(geometry);
    let world_center = transform_local_point(local_center, transform);
    let focus_depth = view_depth_of_world_point(world_center, camera).max(0.05);
    let radius_scale = transform
        .scale
        .x
        .abs()
        .max(transform.scale.y.abs())
        .max(transform.scale.z.abs());
    let world_radius = (local_radius * radius_scale).max(0.05);
    let near_band = response.focus_near_band.max(world_radius * 0.55);
    let far_band = response.focus_far_band.max(world_radius * 1.15);
    let near = (focus_depth - near_band).max(0.05);
    let far = (focus_depth + far_band).max(near + 0.05);
    let focus_distance01 = ((focus_depth - response.near_distance)
        / (response.far_distance - response.near_distance).max(0.001))
    .clamp(0.0, 1.0);
    (near, far, focus_distance01)
}

fn mesh_local_center_and_radius(geometry: &CachedMeshGeometry3d) -> (Vec3, f32) {
    let Some(first) = geometry.vertices().first().copied() else {
        return (Vec3::ZERO, 0.5);
    };

    let mut min = first;
    let mut max = first;
    for vertex in geometry.vertices().iter().copied().skip(1) {
        min.x = min.x.min(vertex.x);
        min.y = min.y.min(vertex.y);
        min.z = min.z.min(vertex.z);
        max.x = max.x.max(vertex.x);
        max.y = max.y.max(vertex.y);
        max.z = max.z.max(vertex.z);
    }

    let center = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    let mut radius = 0.0f32;
    for vertex in geometry.vertices().iter().copied() {
        radius = radius.max(vec3_length(vec3_sub(vertex, center)));
    }
    (center, radius.max(0.05))
}

fn transform_local_point(point: Vec3, transform: Transform3) -> Vec3 {
    let scaled = Vec3::new(
        point.x * transform.scale.x,
        point.y * transform.scale.y,
        point.z * transform.scale.z,
    );
    vec3_add(
        rotate_euler_vec3(scaled, transform.rotation_euler),
        transform.translation,
    )
}

fn view_depth_of_world_point(world: Vec3, camera: Transform3) -> f32 {
    let camera_space =
        rotate_inverse_vec3(vec3_sub(world, camera.translation), camera.rotation_euler);
    -camera_space.z
}

fn rotate_euler_vec3(v: Vec3, rotation: Vec3) -> Vec3 {
    let cx = rotation.x.cos();
    let sx = rotation.x.sin();
    let cy = rotation.y.cos();
    let sy = rotation.y.sin();
    let cz = rotation.z.cos();
    let sz = rotation.z.sin();

    let rx = Vec3::new(v.x, v.y * cx - v.z * sx, v.y * sx + v.z * cx);
    let ry = Vec3::new(rx.x * cy + rx.z * sy, rx.y, -rx.x * sy + rx.z * cy);
    Vec3::new(ry.x * cz - ry.y * sz, ry.x * sz + ry.y * cz, ry.z)
}

fn rotate_inverse_vec3(v: Vec3, rotation: Vec3) -> Vec3 {
    let around_z = rotate_euler_vec3(v, Vec3::new(0.0, 0.0, -rotation.z));
    let around_y = rotate_euler_vec3(around_z, Vec3::new(0.0, -rotation.y, 0.0));
    rotate_euler_vec3(around_y, Vec3::new(-rotation.x, 0.0, 0.0))
}

fn vec3_add(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x + right.x, left.y + right.y, left.z + right.z)
}

fn vec3_sub(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x - right.x, left.y - right.y, left.z - right.z)
}

fn vec3_length(value: Vec3) -> f32 {
    (value.x * value.x + value.y * value.y + value.z * value.z).sqrt()
}
