use std::sync::Arc;

use amigo_scripting_api::ScriptCommandQueue;

use crate::bindings::commands::{
    queue_render_layer2d_set_opacity, queue_render_layer2d_set_visible,
};

#[derive(Clone)]
pub struct Render2dApi {
    pub(crate) command_queue: Option<Arc<ScriptCommandQueue>>,
}

#[derive(Clone)]
pub struct RenderLayer2dHandle {
    id: String,
    command_queue: Option<Arc<ScriptCommandQueue>>,
}

impl Render2dApi {
    pub fn get_layer(&mut self, id: &str) -> RenderLayer2dHandle {
        RenderLayer2dHandle {
            id: id.to_owned(),
            command_queue: self.command_queue.clone(),
        }
    }
}

impl RenderLayer2dHandle {
    pub fn id(&mut self) -> String {
        self.id.clone()
    }

    pub fn inspect_layer_id(&self) -> String {
        self.id.clone()
    }

    pub fn set_opacity(&mut self, opacity: rhai::FLOAT) -> bool {
        if self.id.is_empty() || !opacity.is_finite() {
            return false;
        }
        queue_render_layer2d_set_opacity(
            self.command_queue.as_ref(),
            &self.id,
            (opacity as f32).clamp(0.0, 1.0),
        )
    }

    pub fn set_visible(&mut self, visible: bool) -> bool {
        if self.id.is_empty() {
            return false;
        }
        queue_render_layer2d_set_visible(self.command_queue.as_ref(), &self.id, visible)
    }
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<Render2dApi>("WorldRender2d")
        .register_type_with_name::<RenderLayer2dHandle>("WorldRenderLayer2dHandle")
        .register_fn("get_layer", Render2dApi::get_layer)
        .register_fn("id", RenderLayer2dHandle::id)
        .register_fn("set_opacity", RenderLayer2dHandle::set_opacity)
        .register_fn("set_visible", RenderLayer2dHandle::set_visible);
}
