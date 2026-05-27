use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
};

#[derive(Default)]
pub struct GlobalLight2dPluginGraphProvider;

impl PluginComponentGraphProvider for GlobalLight2dPluginGraphProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.light-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.lighting.light-2d.GlobalLight2D"
    }

    fn primary_render_layer(&self, _payload: &dyn SceneComponentPayload) -> Option<String> {
        None
    }

    fn add_references(&self, _ctx: &mut PluginComponentGraphContext<'_>) {}
}
