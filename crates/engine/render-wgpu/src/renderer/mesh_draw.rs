use amigo_math::{ColorRgba, Transform3, Vec2, Vec3};

use crate::renderer::{
    CachedMeshGeometry3d, ProjectedTriangle, Viewport, cross, dot, modulate_color, multiply_color,
    normalize, project_point_with_camera, projected_triangle_is_sane, sub, transform_point_3d,
    triangle_center,
};

pub(crate) fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    light_settings: amigo_render_api::Light3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    base_color: ColorRgba,
    render_order: i32,
    shading: amigo_render_api::Material3dShadingMode,
) {
    for triangle in &geometry.triangles {
        let world = triangle
            .indices
            .map(|index| transform_point_3d(geometry.vertices[index], transform));
        let projected = world.map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        });
        let [Some(a), Some(b), Some(c)] = projected else {
            continue;
        };
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        if dot(normal, sub(camera.translation, center)) <= 0.0 {
            continue;
        }
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }
        if !projected_triangle_overlaps_viewport([a.position, b.position, c.position]) {
            continue;
        }
        let shaded = match shading {
            amigo_render_api::Material3dShadingMode::Lit => {
                let light_dir = normalize(Vec3::new(
                    -light_settings.direction.x,
                    -light_settings.direction.y,
                    -light_settings.direction.z,
                ));
                let lit = dot(normal, light_dir).max(0.0) * light_settings.intensity.max(0.0);
                let brightness: f32 = (light_settings.ambient.max(0.0) + lit).clamp(0.0, 1.25);
                multiply_color(
                    force_opaque(modulate_color(base_color, brightness)),
                    light_settings.color,
                )
            }
            amigo_render_api::Material3dShadingMode::Unlit => force_opaque(base_color),
        };
        triangles.push(ProjectedTriangle {
            points: [a.position, b.position, c.position],
            color: shaded,
            depth: (a.depth + b.depth + c.depth) / 3.0,
            render_order,
        });
    }
}

pub(crate) fn append_mesh_black_mass_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    material_ids: &[u32],
    render_order: i32,
) {
    if material_ids.is_empty() {
        return;
    }
    let black = ColorRgba::new(0.0, 0.0, 0.0, 1.0);
    for triangle in &geometry.triangles {
        let Some(material_id) = triangle.material_id else {
            continue;
        };
        if !material_ids.contains(&material_id) {
            continue;
        }
        let world = triangle
            .indices
            .map(|index| transform_point_3d(geometry.vertices[index], transform));
        let projected = world.map(|point| {
            project_point_with_camera(
                point,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        });
        let [Some(a), Some(b), Some(c)] = projected else {
            continue;
        };
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        if dot(normal, sub(camera.translation, center)) <= 0.0 {
            continue;
        }
        if !projected_triangle_is_sane([a.position, b.position, c.position]) {
            continue;
        }
        if !projected_triangle_overlaps_viewport([a.position, b.position, c.position]) {
            continue;
        }
        triangles.push(ProjectedTriangle {
            points: [a.position, b.position, c.position],
            color: black,
            depth: (a.depth + b.depth + c.depth) / 3.0,
            render_order,
        });
    }
}

fn projected_triangle_overlaps_viewport(points: [Vec2; 3]) -> bool {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    max_x >= -1.0 && min_x <= 1.0 && max_y >= -1.0 && min_y <= 1.0
}

fn force_opaque(color: ColorRgba) -> ColorRgba {
    ColorRgba::new(color.r, color.g, color.b, 1.0)
}
