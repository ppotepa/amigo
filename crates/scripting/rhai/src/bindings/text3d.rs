use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::queue_text3d_spawn;

#[derive(Clone)]
pub struct Text3dApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Text3dApi {
    pub fn queue(
        &mut self,
        entity_name: &str,
        content: &str,
        font_key: &str,
        size: rhai::FLOAT,
    ) -> bool {
        if size <= 0.0 {
            return false;
        }
        queue_text3d_spawn(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            content,
            font_key,
            size as f32,
        )
    }
}
