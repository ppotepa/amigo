use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_ui_disable, queue_ui_enable, queue_ui_hide, queue_ui_set_color, queue_ui_set_text,
    queue_ui_set_value, queue_ui_show,
};

#[derive(Clone)]
pub struct UiApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl UiApi {
    pub fn set_text(&mut self, path: &str, value: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        queue_ui_set_text(self.command_queue.as_ref(), path, value)
    }

    pub fn set_many(&mut self, updates: rhai::Map) -> rhai::INT {
        let mut queued = 0;
        for (path, value) in updates {
            if path.is_empty() {
                continue;
            }
            let value = value
                .clone()
                .try_cast::<String>()
                .unwrap_or_else(|| value.to_string());
            if queue_ui_set_text(self.command_queue.as_ref(), path.as_str(), value.as_str()) {
                queued += 1;
            }
        }
        queued
    }

    pub fn set_value(&mut self, path: &str, value: rhai::FLOAT) -> bool {
        if path.is_empty() || !value.is_finite() {
            return false;
        }
        queue_ui_set_value(self.command_queue.as_ref(), path, value as f32)
    }

    pub fn set_color(&mut self, path: &str, value: &str) -> bool {
        if path.is_empty() || value.is_empty() {
            return false;
        }
        queue_ui_set_color(self.command_queue.as_ref(), path, value)
    }

    pub fn show(&mut self, path: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        queue_ui_show(self.command_queue.as_ref(), path)
    }

    pub fn hide(&mut self, path: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        queue_ui_hide(self.command_queue.as_ref(), path)
    }

    pub fn enable(&mut self, path: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        queue_ui_enable(self.command_queue.as_ref(), path)
    }

    pub fn disable(&mut self, path: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        queue_ui_disable(self.command_queue.as_ref(), path)
    }
}
