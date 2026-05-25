use amigo_math::{Transform2, Transform3, Vec2, Vec3};
use amigo_render_api::RenderSceneView;
use amigo_scene::SceneService;

pub fn build_render_scene_view(
    scene: &SceneService,
    active_camera_2d_entity: Option<&str>,
) -> RenderSceneView {
    let mut view = RenderSceneView::new(
        resolve_camera_2d_transform(scene, active_camera_2d_entity),
        resolve_camera_3d_transform(scene),
    );
    for entity in scene.entities() {
        view.insert_entity_transform(entity.name, entity.transform);
    }
    view
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
