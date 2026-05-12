use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};
use amigo_math::Vec2;
use amigo_runtime::Runtime;

use crate::{
    CameraFollow2dSceneService, EntityPoolSceneService, LifetimeSceneService, Parallax2dSceneService,
    SceneCommandQueue, SceneService, SceneTransitionService,
};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub fn tick_camera_follow_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let camera_follow_scene_service = required::<CameraFollow2dSceneService>(runtime)?;

    for follow in camera_follow_scene_service.commands() {
        let Some(target_transform) = scene_service.transform_of(&follow.target) else {
            continue;
        };
        let Some(mut camera_transform) = scene_service.transform_of(&follow.entity_name) else {
            continue;
        };

        let velocity = Vec2::ZERO;
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

pub fn tick_parallax_world(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let parallax_scene_service = required::<Parallax2dSceneService>(runtime)?;

    for parallax in parallax_scene_service.commands() {
        let Some(camera_transform) = scene_service.transform_of(&parallax.camera) else {
            continue;
        };
        let Some(mut entity_transform) = scene_service.transform_of(&parallax.entity_name) else {
            continue;
        };

        let camera_translation = Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        );
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

pub fn tick_lifetimes(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let entity_pool_scene_service = required::<EntityPoolSceneService>(runtime)?;
    let lifetime_scene_service = required::<LifetimeSceneService>(runtime)?;

    for expired in lifetime_scene_service.tick(delta_seconds) {
        match expired.outcome {
            crate::LifetimeExpirationOutcome::Hide => {
                let _ = scene_service.set_visible(&expired.entity_name, false);
            }
            crate::LifetimeExpirationOutcome::Disable => {
                let _ = scene_service.set_visible(&expired.entity_name, false);
                let _ = scene_service.set_simulation_enabled(&expired.entity_name, false);
                let _ = scene_service.set_collision_enabled(&expired.entity_name, false);
            }
            crate::LifetimeExpirationOutcome::Despawn => {
                let _ = scene_service.remove_entities_by_name(&[expired.entity_name]);
            }
            crate::LifetimeExpirationOutcome::ReturnToPool { pool } => {
                let _ =
                    entity_pool_scene_service.release(&scene_service, &pool, &expired.entity_name);
            }
        }
    }

    Ok(())
}

pub fn tick_scene_transitions(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;

    for command in scene_transition_service.tick(delta_seconds) {
        scene_command_queue.submit(command);
    }

    Ok(())
}
