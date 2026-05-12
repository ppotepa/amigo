use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;
use amigo_scene::{EntityPoolSceneService, SceneService};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};

use crate::{Physics2dSceneService, evaluate_collision_event_rules_with_pools};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub fn tick_collision_events_2d(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let physics_scene_service = required::<Physics2dSceneService>(runtime)?;
    let entity_pool_scene_service = required::<EntityPoolSceneService>(runtime)?;
    let script_event_queue = required::<ScriptEventQueue>(runtime)?;

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
