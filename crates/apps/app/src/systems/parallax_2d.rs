use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_parallax_world(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let parallax_scene_service = ctx.required::<Parallax2dSceneService>()?;

    for parallax in parallax_scene_service.commands() {
        let Some(camera_transform) = scene_service.transform_of(&parallax.camera) else {
            continue;
        };
        let Some(mut entity_transform) = scene_service.transform_of(&parallax.entity_name) else {
            continue;
        };

        let camera_translation =
            Vec2::new(camera_transform.translation.x, camera_transform.translation.y);
        let camera_origin = parallax.camera_origin.unwrap_or(camera_translation);
        if parallax.camera_origin.is_none() {
            let _ =
                parallax_scene_service.set_camera_origin(&parallax.entity_name, camera_translation);
        }

        let factor_x = parallax.factor.x.clamp(0.0, 1.0);
        let factor_y = parallax.factor.y.clamp(0.0, 1.0);
        entity_transform.translation.x =
            parallax.anchor.x + (camera_translation.x - camera_origin.x) * (1.0 - factor_x);
        entity_transform.translation.y =
            parallax.anchor.y + (camera_translation.y - camera_origin.y) * (1.0 - factor_y);

        let _ = scene_service.set_transform(&parallax.entity_name, entity_transform);
    }

    Ok(())
}
