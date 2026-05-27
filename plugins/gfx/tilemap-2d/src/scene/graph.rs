use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
    SceneReferenceKind, SceneReferenceTargetKind,
};

use super::Tilemap2dDocument;

#[derive(Default)]
pub struct TileMap2dPluginGraphProvider;

impl PluginComponentGraphProvider for TileMap2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.tilemap-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.tilemap-2d.TileMap2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        let payload = payload.as_any().downcast_ref::<Tilemap2dDocument>()?;
        Some(payload.render_layer.clone())
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<Tilemap2dDocument>() else {
            return;
        };

        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
        ctx.add_external_ref(
            "tileset",
            SceneReferenceKind::UsesTileset,
            SceneReferenceTargetKind::Asset,
            &payload.tileset,
            true,
        );
        if let Some(ruleset) = &payload.ruleset {
            ctx.add_external_ref(
                "ruleset",
                SceneReferenceKind::UsesRuleset,
                SceneReferenceTargetKind::Asset,
                ruleset,
                false,
            );
        }
    }
}
