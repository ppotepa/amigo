use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_lifetimes(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let entity_pool_scene_service = ctx.required::<EntityPoolSceneService>()?;
    let lifetime_scene_service = ctx.required::<LifetimeSceneService>()?;

    for expired in lifetime_scene_service.tick(delta_seconds) {
        match expired.outcome {
            amigo_scene::LifetimeExpirationOutcome::Hide => {
                let _ = scene_service.set_visible(&expired.entity_name, false);
            }
            amigo_scene::LifetimeExpirationOutcome::Disable => {
                let _ = scene_service.set_visible(&expired.entity_name, false);
                let _ = scene_service.set_simulation_enabled(&expired.entity_name, false);
                let _ = scene_service.set_collision_enabled(&expired.entity_name, false);
            }
            amigo_scene::LifetimeExpirationOutcome::Despawn => {
                let _ = scene_service.remove_entities_by_name(&[expired.entity_name]);
            }
            amigo_scene::LifetimeExpirationOutcome::ReturnToPool { pool } => {
                let _ =
                    entity_pool_scene_service.release(&scene_service, &pool, &expired.entity_name);
            }
        }
    }

    Ok(())
}
