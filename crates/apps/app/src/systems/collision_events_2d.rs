use super::super::*;
use crate::runtime_context::RuntimeContext;
use amigo_2d_physics::evaluate_collision_event_rules_with_pools;

pub(crate) fn tick_collision_events_2d(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let physics_scene_service = ctx.required::<Physics2dSceneService>()?;
    let entity_pool_scene_service = ctx.required::<EntityPoolSceneService>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;

    for event in evaluate_collision_event_rules_with_pools(
        &scene_service,
        &physics_scene_service,
        Some(&entity_pool_scene_service),
    ) {
        script_event_queue.publish(ScriptEvent::new(
            event.topic,
            vec![event.source_entity, event.target_entity],
        ));
    }

    Ok(())
}
