use std::sync::Arc;

use amigo_scripting_api::{DevConsoleQueue, ScriptCommandQueue, ScriptEventQueue};

use crate::bindings::commands::{
    emit_script_event, queue_console_command, queue_debug_message, queue_placeholder_command,
};

#[derive(Clone)]
pub struct DebugApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
    pub(crate) event_queue: Option<Arc<ScriptEventQueue>>,
    pub(crate) console_queue: Option<Arc<DevConsoleQueue>>,
}

impl DebugApi {
    pub fn event(&mut self, topic: &str) {
        emit_script_event(self.event_queue.as_ref(), topic, None);
    }

    pub fn event_with_payload(&mut self, topic: &str, payload: &str) {
        emit_script_event(self.event_queue.as_ref(), topic, Some(payload));
    }

    pub fn command(&mut self, line: &str) {
        queue_console_command(self.console_queue.as_ref(), line);
    }

    pub fn log(&mut self, line: &str) {
        queue_debug_message(self.command_queue.as_ref(), "log", line);
    }

    pub fn warn(&mut self, line: &str) {
        queue_debug_message(self.command_queue.as_ref(), "warn", line);
    }

    pub fn write_text(&mut self, relative_path: &str, contents: &str) -> bool {
        if relative_path.is_empty() {
            return false;
        }
        queue_placeholder_command(
            self.command_queue.as_ref(),
            "debug",
            "write-text",
            vec![relative_path.to_owned(), contents.to_owned()],
        )
    }

    pub fn refresh_diagnostics(&mut self, target_mod: &str) -> bool {
        queue_placeholder_command(
            self.command_queue.as_ref(),
            "dev-shell",
            "refresh-diagnostics",
            vec![target_mod.to_owned()],
        )
    }
}
