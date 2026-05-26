use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::queue_mesh3d_spawn;

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
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<Mesh3dApi>("WorldMesh3d")
        .register_fn("queue", Mesh3dApi::queue);
}
