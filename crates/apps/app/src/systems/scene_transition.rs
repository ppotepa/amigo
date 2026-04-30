use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::{SceneCommandQueue, SceneTransitionService};

use crate::runtime_context::required;

pub(crate) fn tick_scene_transitions(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;

    for command in scene_transition_service.tick(delta_seconds) {
        scene_command_queue.submit(command);
    }

    Ok(())
}
