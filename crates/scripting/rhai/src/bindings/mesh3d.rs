use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_mesh3d_apply_npr_preset, queue_mesh3d_set_npr_temporal_path_smoothing,
    queue_mesh3d_spawn,
};

#[derive(Clone)]
pub struct Mesh3dApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Mesh3dApi {
    pub fn queue(&mut self, entity_name: &str, mesh_key: &str) -> bool {
        queue_mesh3d_spawn(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            mesh_key,
        )
    }

    pub fn apply_npr_preset(&mut self, entity_name: &str, preset_id: &str) -> bool {
        queue_mesh3d_apply_npr_preset(self.command_queue.as_ref(), entity_name, preset_id)
    }

    pub fn set_npr_temporal_path_smoothing(&mut self, entity_name: &str, enabled: bool) -> bool {
        queue_mesh3d_set_npr_temporal_path_smoothing(
            self.command_queue.as_ref(),
            entity_name,
            enabled,
        )
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<Mesh3dApi>("WorldMesh3d")
        .register_fn("queue", Mesh3dApi::queue)
        .register_fn("apply_npr_preset", Mesh3dApi::apply_npr_preset)
        .register_fn(
            "set_npr_temporal_path_smoothing",
            Mesh3dApi::set_npr_temporal_path_smoothing,
        );
}
