use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;
use amigo_ui::UiThemeService;

use crate::bindings::commands::{
    queue_placeholder_command, queue_ui_disable, queue_ui_enable, queue_ui_hide,
    queue_ui_set_color, queue_ui_set_text, queue_ui_set_value, queue_ui_show,
};

#[derive(Clone)]
pub struct UiApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
    pub(crate) theme_service: Option<Arc<UiThemeService>>,
}

impl UiApi {
    pub fn set_theme(&mut self, theme_id: &str) -> bool {
        if theme_id.is_empty() {
            return false;
        }
        self.theme_service
            .as_ref()
            .map(|themes| themes.set_active_theme(theme_id))
            .unwrap_or(false)
    }

    pub fn theme(&mut self) -> String {
        self.theme_service
            .as_ref()
            .and_then(|themes| themes.active_theme_id())
            .unwrap_or_default()
    }

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

    pub fn set_selected(&mut self, path: &str, value: &str) -> bool {
        if path.is_empty() || value.is_empty() {
            return false;
        }
        self.command_queue
            .as_ref()
            .map(|queue| {
                queue.submit(amigo_scripting_api::ScriptCommand::new(
                    "ui",
                    "set_selected",
                    vec![path.to_owned(), value.to_owned()],
                ));
                true
            })
            .unwrap_or(false)
    }

    pub fn set_options(&mut self, path: &str, options: rhai::Array) -> bool {
        if path.is_empty() {
            return false;
        }
        let mut arguments = vec![path.to_owned()];
        for option in options {
            let value = option
                .clone()
                .try_cast::<String>()
                .unwrap_or_else(|| option.to_string());
            if !value.is_empty() {
                arguments.push(value);
            }
        }
        self.command_queue
            .as_ref()
            .map(|queue| {
                queue.submit(amigo_scripting_api::ScriptCommand::new(
                    "ui",
                    "set-options",
                    arguments,
                ));
                true
            })
            .unwrap_or(false)
    }

    pub fn set_color(&mut self, path: &str, value: &str) -> bool {
        if path.is_empty() || value.is_empty() {
            return false;
        }
        queue_ui_set_color(self.command_queue.as_ref(), path, value)
    }

    pub fn set_background(&mut self, path: &str, value: &str) -> bool {
        if path.is_empty() || value.is_empty() {
            return false;
        }
        queue_placeholder_command(
            self.command_queue.as_ref(),
            "ui",
            "set-background",
            vec![path.to_owned(), value.to_owned()],
        )
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
