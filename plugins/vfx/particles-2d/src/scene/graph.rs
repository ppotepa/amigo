use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
    SceneReferenceKind,
};

use super::ParticleEmitter2dDocument;

#[derive(Default)]
pub struct ParticleEmitter2dPluginGraphProvider;

impl PluginComponentGraphProvider for ParticleEmitter2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.vfx.particles-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.vfx.particles-2d.ParticleEmitter2D"
    }

    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String> {
        let payload = payload
            .as_any()
            .downcast_ref::<ParticleEmitter2dDocument>()?;
        Some(payload.render_layer.clone())
    }

    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>) {
        let Some(payload) = ctx
            .payload
            .as_any()
            .downcast_ref::<ParticleEmitter2dDocument>()
        else {
            return;
        };

        ctx.add_draw_layer_ref("render_layer", &payload.render_layer);
        if let Some(attached_to) = &payload.attached_to {
            ctx.add_scene_object_ref(
                "attached_to",
                SceneReferenceKind::AttachedToSceneObject,
                attached_to,
                "missing_attached_scene_object",
            );
        }
    }
}
