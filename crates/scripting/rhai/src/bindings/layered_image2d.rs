use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_layered_image_set_base_opacity, queue_layered_image_set_blend,
    queue_layered_image_set_enabled, queue_layered_image_set_opacity,
};

#[derive(Clone)]
pub struct LayeredImage2dApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl LayeredImage2dApi {
    pub fn set_base_opacity(&mut self, entity_name: &str, opacity: rhai::FLOAT) -> bool {
        if entity_name.is_empty() || !opacity.is_finite() {
            return false;
        }
        queue_layered_image_set_base_opacity(
            self.command_queue.as_ref(),
            entity_name,
            (opacity as f32).clamp(0.0, 1.0),
        )
    }

    pub fn set_opacity(&mut self, entity_name: &str, layer_id: &str, opacity: rhai::FLOAT) -> bool {
        if entity_name.is_empty() || layer_id.is_empty() || !opacity.is_finite() {
            return false;
        }
        queue_layered_image_set_opacity(
            self.command_queue.as_ref(),
            entity_name,
            layer_id,
            (opacity as f32).clamp(0.0, 4.0),
        )
    }

    pub fn set_enabled(&mut self, entity_name: &str, layer_id: &str, enabled: bool) -> bool {
        if entity_name.is_empty() || layer_id.is_empty() {
            return false;
        }
        queue_layered_image_set_enabled(self.command_queue.as_ref(), entity_name, layer_id, enabled)
    }

    pub fn set_blend(&mut self, entity_name: &str, layer_id: &str, blend: &str) -> bool {
        if entity_name.is_empty() || layer_id.is_empty() || blend.is_empty() {
            return false;
        }
        queue_layered_image_set_blend(self.command_queue.as_ref(), entity_name, layer_id, blend)
    }
}

