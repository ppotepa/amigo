use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_camera_follow_world(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let camera_follow_scene_service = ctx.required::<CameraFollow2dSceneService>()?;

    for follow in camera_follow_scene_service.commands() {
        let Some(target_transform) = scene_service.transform_of(&follow.target) else {
            continue;
        };
        let Some(mut camera_transform) = scene_service.transform_of(&follow.entity_name) else {
            continue;
        };

        let desired_x = target_transform.translation.x + follow.offset.x;
        let desired_y = target_transform.translation.y + follow.offset.y;
        let alpha = follow.lerp.clamp(0.0, 1.0);

        if alpha >= 1.0 {
            camera_transform.translation.x = desired_x;
            camera_transform.translation.y = desired_y;
        } else {
            camera_transform.translation.x += (desired_x - camera_transform.translation.x) * alpha;
            camera_transform.translation.y += (desired_y - camera_transform.translation.y) * alpha;
        }

        let _ = scene_service.set_transform(&follow.entity_name, camera_transform);
    }

    Ok(())
}
