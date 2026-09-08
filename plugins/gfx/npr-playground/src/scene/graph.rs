use amigo_scene::{
    PluginComponentGraphContext, PluginComponentGraphProvider, SceneComponentPayload,
};

use super::document::NPR_SETTINGS_COMPONENT_TYPE;

#[derive(Default)]
pub struct NprPlaygroundPluginGraphProvider;

impl PluginComponentGraphProvider for NprPlaygroundPluginGraphProvider {
    fn provider_id(&self) -> &'static str { "amigo.gfx.npr-playground" }
    fn component_type(&self) -> &'static str { NPR_SETTINGS_COMPONENT_TYPE }
    fn primary_render_layer(&self, _payload: &dyn SceneComponentPayload) -> Option<String> { None }
    fn add_references(&self, _ctx: &mut PluginComponentGraphContext<'_>) {}
}
