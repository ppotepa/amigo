use std::sync::Arc;

use amigo_core::LaunchSelection;
use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::queue_text2d_spawn;

#[derive(Clone)]
pub struct Text2dApi {
    pub(crate) launch_selection: Option<Arc<LaunchSelection>>,
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Text2dApi {
    pub fn queue(
        &mut self,
        entity_name: &str,
        content: &str,
        font_key: &str,
        width: rhai::INT,
        height: rhai::INT,
    ) -> bool {
        if width <= 0 || height <= 0 {
            return false;
        }
        queue_text2d_spawn(
            self.launch_selection.as_ref(),
            self.command_queue.as_ref(),
            entity_name,
            content,
            font_key,
            width,
            height,
        )
    }
}
