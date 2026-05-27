use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{queue_physics3d_dynamic_box, queue_physics3d_static_box};

#[derive(Clone)]
pub struct Physics3dApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Physics3dApi {
    #[allow(clippy::too_many_arguments)]
    pub fn dynamic_box(
        &mut self,
        entity_name: &str,
        size_x: rhai::FLOAT,
        size_y: rhai::FLOAT,
        size_z: rhai::FLOAT,
        velocity_x: rhai::FLOAT,
        velocity_y: rhai::FLOAT,
        velocity_z: rhai::FLOAT,
    ) -> bool {
        queue_physics3d_dynamic_box(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            size_x as f32,
            size_y as f32,
            size_z as f32,
            velocity_x as f32,
            velocity_y as f32,
            velocity_z as f32,
        )
    }

    pub fn static_box(
        &mut self,
        entity_name: &str,
        size_x: rhai::FLOAT,
        size_y: rhai::FLOAT,
        size_z: rhai::FLOAT,
    ) -> bool {
        queue_physics3d_static_box(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            size_x as f32,
            size_y as f32,
            size_z as f32,
        )
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<Physics3dApi>("WorldPhysics3d")
        .register_fn("dynamic_box", Physics3dApi::dynamic_box)
        .register_fn("static_box", Physics3dApi::static_box);
}
