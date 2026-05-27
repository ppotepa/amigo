use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
    SceneReferenceKind, SceneReferenceTargetKind,
};

use super::Sprite2dDocument;

#[derive(Default)]
pub struct Sprite2dPluginGraphProvider;

impl PluginComponentGraphProvider for Sprite2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.sprite-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.sprite-2d.Sprite2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        let payload = payload.as_any().downcast_ref::<Sprite2dDocument>()?;
        Some(payload.render_layer.clone())
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<Sprite2dDocument>() else {
            return;
        };

        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
        ctx.add_external_ref(
            "texture",
            SceneReferenceKind::UsesAsset,
            SceneReferenceTargetKind::Asset,
            &payload.texture,
            true,
        );
    }
}
