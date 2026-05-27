use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
};

use super::Vector2dDocument;

#[derive(Default)]
pub struct Vector2dPluginGraphProvider;

impl PluginComponentGraphProvider for Vector2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.vector-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.vector-2d.VectorShape2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        let payload = payload.as_any().downcast_ref::<Vector2dDocument>()?;
        Some(payload.render_layer.clone())
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<Vector2dDocument>() else {
            return;
        };

        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
    }
}
