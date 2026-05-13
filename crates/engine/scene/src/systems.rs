use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;

use crate::{
    EntityPoolSceneService, LifetimeSceneService,
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


