use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_light_group2d_set_color, queue_light_group2d_set_intensity, queue_light2d_set_color,
    queue_light2d_set_intensity,
};

#[derive(Clone)]
pub struct Light2dApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

#[derive(Clone)]
pub struct Light2dHandle {
    id: String,
    command_queue: Option<Arc<ScriptCommandQueue>>,
}

#[derive(Clone)]
pub struct LightGroup2dHandle {
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

    pub fn get_group(&mut self, id: &str) -> LightGroup2dHandle {
        LightGroup2dHandle {
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

impl LightGroup2dHandle {
    pub fn id(&mut self) -> String {
        self.id.clone()
    }

    pub fn set_intensity(&mut self, intensity: rhai::FLOAT) -> bool {
        if self.id.is_empty() || !intensity.is_finite() {
            return false;
        }
        queue_light_group2d_set_intensity(
            self.command_queue.as_ref(),
            &self.id,
            (intensity as f32).max(0.0),
        )
    }

    pub fn set_color(&mut self, color: &str) -> bool {
        if self.id.is_empty() || color.is_empty() {
            return false;
        }
        queue_light_group2d_set_color(self.command_queue.as_ref(), &self.id, color)
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

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<Light2dApi>("WorldLight2d")
        .register_type_with_name::<Light2dHandle>("WorldLight2dHandle")
        .register_type_with_name::<LightGroup2dHandle>("WorldLightGroup2dHandle")
        .register_fn("get_light", Light2dApi::get_light)
        .register_fn("get_group", Light2dApi::get_group)
        .register_fn("set_intensity", Light2dApi::set_intensity)
        .register_fn("set_color", Light2dApi::set_color)
        .register_fn("id", Light2dHandle::id)
        .register_fn("set_intensity", Light2dHandle::set_intensity)
        .register_fn("set_color", Light2dHandle::set_color)
        .register_fn("id", LightGroup2dHandle::id)
        .register_fn("set_intensity", LightGroup2dHandle::set_intensity)
        .register_fn("set_color", LightGroup2dHandle::set_color);
}
