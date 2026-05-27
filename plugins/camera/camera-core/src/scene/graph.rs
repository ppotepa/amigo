use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
};

#[derive(Default)]
pub struct Camera2dPluginGraphProvider;

impl PluginComponentGraphProvider for Camera2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.camera-core"
    }

    fn component_type(&self) -> &'static str {
        "amigo.camera.camera-core.Camera2D"
    }

    fn primary_render_layer(&self, _payload: &dyn SceneComponentPayload) -> Option<String> {
        None
    }

    fn add_references(&self, _ctx: &mut PluginComponentGraphContext<'_>) {}
}
