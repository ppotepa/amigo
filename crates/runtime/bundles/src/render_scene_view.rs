use amigo_math::{Transform2, Transform3, Vec2, Vec3};
use amigo_render_api::{Camera3dRenderSettings, Light3dRenderSettings, RenderSceneView};
use amigo_scene::{ScenePropertyValue, SceneService};

pub fn build_render_scene_view(
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
) -> RenderSceneView {
    let mut view = RenderSceneView::new(
        resolve_camera_2d_transform(scene, active_camera_2d_entity),
        resolve_camera_3d_transform(scene),
    );
    view.set_camera_3d_settings(resolve_camera_3d_settings(scene));
    view.set_light_3d_settings(resolve_light_3d_settings(scene));
    for entity in scene.entities() {
        view.insert_entity_transform(entity.name, entity.transform);
    }
    view
}

fn resolve_camera_3d_settings(scene: &SceneService) -> Camera3dRenderSettings {
    let Some(camera) = scene.entities().into_iter().find(|entity| {
        entity.name.contains("3d-camera")
            || (entity.name.contains("camera") && entity.transform.translation.z.abs() > 0.01)
    }) else {
        return Camera3dRenderSettings::default();
    };
    Camera3dRenderSettings {
        fov_y_degrees: property_float(&camera.properties, "camera3d.fov_y_degrees")
            .unwrap_or(Camera3dRenderSettings::default().fov_y_degrees),
        near_clip: property_float(&camera.properties, "camera3d.near_clip")
            .unwrap_or(Camera3dRenderSettings::default().near_clip),
        far_clip: property_float(&camera.properties, "camera3d.far_clip")
            .unwrap_or(Camera3dRenderSettings::default().far_clip),
    }
}

fn resolve_light_3d_settings(scene: &SceneService) -> Light3dRenderSettings {
    let Some(light) = scene
        .entities()
        .into_iter()
        .find(|entity| entity.name.contains("3d-light") || entity.name.contains("light"))
    else {
        return Light3dRenderSettings::default();
    };
    let defaults = Light3dRenderSettings::default();
    Light3dRenderSettings {
        direction: Vec3::new(
            property_float(&light.properties, "light3d.direction.x")
                .unwrap_or(defaults.direction.x),
            property_float(&light.properties, "light3d.direction.y")
                .unwrap_or(defaults.direction.y),
            property_float(&light.properties, "light3d.direction.z")
                .unwrap_or(defaults.direction.z),
        ),
        color: property_string(&light.properties, "light3d.color")
            .and_then(parse_hex_color)
            .unwrap_or(defaults.color),
        intensity: property_float(&light.properties, "light3d.intensity")
            .unwrap_or(defaults.intensity),
        ambient: property_float(&light.properties, "light3d.ambient").unwrap_or(defaults.ambient),
    }
}

fn property_float(
    properties: &std::collections::BTreeMap<String, ScenePropertyValue>,
    key: &str,
) -> Option<f32> {
    match properties.get(key) {
        Some(ScenePropertyValue::Float(value)) => Some(*value as f32),
        Some(ScenePropertyValue::Int(value)) => Some(*value as f32),
        _ => None,
    }
}

fn property_string<'a>(
    properties: &'a std::collections::BTreeMap<String, ScenePropertyValue>,
    key: &str,
) -> Option<&'a str> {
    match properties.get(key) {
        Some(ScenePropertyValue::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn parse_hex_color(value: &str) -> Option<amigo_math::ColorRgba> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }
    Some(amigo_math::ColorRgba::new(
        f32::from(parse(0..2)?) / 255.0,
        f32::from(parse(2..4)?) / 255.0,
        f32::from(parse(4..6)?) / 255.0,
        if hex.len() == 8 {
            f32::from(parse(6..8)?) / 255.0
        } else {
            1.0
        },
    ))
}

fn resolve_camera_3d_transform(scene: &SceneService) -> Transform3 {
    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("3d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() > 0.01)
        })
        .map(|entity| entity.transform)
        .unwrap_or(Transform3 {
            translation: Vec3::new(0.0, 0.0, 6.0),
            ..Transform3::default()
        })
}

fn resolve_camera_2d_transform(
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
) -> Transform2 {
    if let Some(active_camera_entity) = active_camera_2d_entity {
        if let Some(entity) = scene
            .entities()
            .into_iter()
            .find(|entity| entity.name == active_camera_entity)
        {
            return transform2_from_transform3(entity.transform);
        }
    }

    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("2d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() <= 0.01)
        })
        .map(|entity| transform2_from_transform3(entity.transform))
        .unwrap_or_default()
}

fn transform2_from_transform3(transform: Transform3) -> Transform2 {
    Transform2 {
        translation: Vec2::new(transform.translation.x, transform.translation.y),
        rotation_radians: transform.rotation_euler.z,
        scale: Vec2::new(transform.scale.x, transform.scale.y),
    }
}
