use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
    SceneReferenceKind, SceneReferenceTargetKind,
};

use super::LayeredImage2dDocument;

#[derive(Default)]
pub struct LayeredImage2dPluginGraphProvider;

impl PluginComponentGraphProvider for LayeredImage2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.layered-image-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.layered-image-2d.LayeredImage2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        let payload = payload.as_any().downcast_ref::<LayeredImage2dDocument>()?;
        Some(payload.render_layer.clone())
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx
            .payload
            .as_any()
            .downcast_ref::<LayeredImage2dDocument>()
        else {
            return;
        };

        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
        ctx.add_external_ref(
            "asset",
            SceneReferenceKind::UsesAsset,
            SceneReferenceTargetKind::Asset,
            &payload.asset,
            true,
        );
        for override_ in &payload.layer_overrides {
            ctx.add_external_ref(
                "layer_overrides.id",
                SceneReferenceKind::UsesImagePart,
                SceneReferenceTargetKind::ImagePart,
                &override_.id,
                false,
            );
        }
    }
}
