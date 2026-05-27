use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
};

use super::BeaconLight2dDocument;

#[derive(Default)]
pub struct BeaconLight2dPluginGraphProvider;

impl PluginComponentGraphProvider for BeaconLight2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d.BeaconLight2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        let payload = payload.as_any().downcast_ref::<BeaconLight2dDocument>()?;
        Some(payload.render_layer.clone())
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<BeaconLight2dDocument>() else {
            return;
        };

        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
    }
}
