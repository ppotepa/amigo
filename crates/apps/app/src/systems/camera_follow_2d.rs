use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_camera_follow_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let camera_follow_scene_service = ctx.required::<CameraFollow2dSceneService>()?;
    let motion_scene_service = ctx.optional::<Motion2dSceneService>();

    for follow in camera_follow_scene_service.commands() {
        let Some(target_transform) = scene_service.transform_of(&follow.target) else {
            continue;
        };
        let Some(mut camera_transform) = scene_service.transform_of(&follow.entity_name) else {
            continue;
        };

        let velocity = motion_scene_service
            .as_ref()
            .map(|motion| motion.current_velocity(&follow.target))
            .unwrap_or(Vec2::ZERO);
        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        let (velocity_dir_x, velocity_dir_y) = if speed > f32::EPSILON {
            (velocity.x / speed, velocity.y / speed)
        } else {
            (0.0, 0.0)
        };
        let lookahead_distance =
            (speed * follow.lookahead_velocity_scale).min(follow.lookahead_max_distance.max(0.0));
        let speed_factor = (speed / 360.0).clamp(0.0, 1.0);
        let sway_phase = (target_transform.translation.x * 0.013
            + target_transform.translation.y * 0.017)
            * follow.sway_frequency.max(0.0);
        let sway = sway_phase.sin() * follow.sway_amount * speed_factor;
        let perpendicular_x = -velocity_dir_y;
        let perpendicular_y = velocity_dir_x;

        let desired_x = target_transform.translation.x
            + follow.offset.x
            + velocity_dir_x * lookahead_distance
            + perpendicular_x * sway;
        let desired_y = target_transform.translation.y
            + follow.offset.y
            + velocity_dir_y * lookahead_distance
            + perpendicular_y * sway;
        let alpha = if follow.lerp >= 1.0 {
            1.0
        } else {
            1.0 - (1.0 - follow.lerp.clamp(0.0, 1.0)).powf((delta_seconds * 60.0).max(0.0))
        };

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
