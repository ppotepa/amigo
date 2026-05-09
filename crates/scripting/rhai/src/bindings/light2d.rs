use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{queue_light2d_set_color, queue_light2d_set_intensity};

#[derive(Clone)]
pub struct Light2dApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

#[derive(Clone)]
pub struct Light2dHandle {
    id: String,
    command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Light2dApi {
    pub fn get_light(&mut self, id: &str) -> Light2dHandle {
        Light2dHandle {
            id: id.to_owned(),
            command_queue: self.command_queue.clone(),
        }
    }

    pub fn set_intensity(&mut self, id: &str, intensity: rhai::FLOAT) -> bool {
        if id.is_empty() || !intensity.is_finite() {
            return false;
        }
        queue_light2d_set_intensity(self.command_queue.as_ref(), id, (intensity as f32).max(0.0))
    }

    pub fn set_color(&mut self, id: &str, color: &str) -> bool {
        if id.is_empty() || color.is_empty() {
            return false;
        }
        queue_light2d_set_color(self.command_queue.as_ref(), id, color)
    }
}

impl Light2dHandle {
    pub fn id(&mut self) -> String {
        self.id.clone()
    }

    pub fn set_intensity(&mut self, intensity: rhai::FLOAT) -> bool {
        if self.id.is_empty() || !intensity.is_finite() {
            return false;
        }
        queue_light2d_set_intensity(
            self.command_queue.as_ref(),
            &self.id,
            (intensity as f32).max(0.0),
        )
    }

    pub fn set_color(&mut self, color: &str) -> bool {
        if self.id.is_empty() || color.is_empty() {
            return false;
        }
        queue_light2d_set_color(self.command_queue.as_ref(), &self.id, color)
    }
}
