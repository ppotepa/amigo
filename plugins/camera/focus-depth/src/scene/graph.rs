use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
    SceneReferenceKind, SceneReferenceTargetKind,
};

use super::{DepthAuxMap2dDocument, DepthMap2dDocument};

#[derive(Default)]
pub struct DepthMap2dPluginGraphProvider;

impl PluginComponentGraphProvider for DepthMap2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth"
    }

    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthMap2D"
    }

    fn primary_render_layer(&self, _payload: &dyn SceneComponentPayload) -> Option<String> {
        None
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<DepthMap2dDocument>() else {
            return;
        };

        ctx.add_external_ref(
            "asset",
            SceneReferenceKind::UsesAsset,
            SceneReferenceTargetKind::Asset,
            &payload.asset,
            true,
        );
    }
}

#[derive(Default)]
pub struct DepthAuxMap2dPluginGraphProvider;

impl PluginComponentGraphProvider for DepthAuxMap2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth"
    }

    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthAuxMap2D"
    }

    fn primary_render_layer(&self, _payload: &dyn SceneComponentPayload) -> Option<String> {
        None
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx.payload.as_any().downcast_ref::<DepthAuxMap2dDocument>() else {
            return;
        };

        ctx.add_external_ref(
            "asset",
            SceneReferenceKind::UsesAsset,
            SceneReferenceTargetKind::Asset,
            &payload.asset,
            true,
        );
        if let Some(surface_asset) = &payload.surface_asset {
            ctx.add_external_ref(
                "surface_asset",
                SceneReferenceKind::UsesAsset,
                SceneReferenceTargetKind::Asset,
                surface_asset,
                false,
            );
        }
    }
}
