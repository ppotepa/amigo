use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::queue_material3d_bind;

#[derive(Clone)]
pub struct Material3dApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Material3dApi {
    pub fn bind(&mut self, entity_name: &str, label: &str, material_key: &str) -> bool {
        queue_material3d_bind(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            label,
            material_key,
        )
    }
}
