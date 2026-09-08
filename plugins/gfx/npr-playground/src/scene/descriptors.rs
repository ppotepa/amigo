use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

use super::document::NPR_SETTINGS_COMPONENT_TYPE;

#[derive(Default)]
pub struct NprPlaygroundSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for NprPlaygroundSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(PluginSceneComponentDescriptor::new(
            NPR_SETTINGS_COMPONENT_TYPE,
            "gfx",
            "NprSettings",
        ));
    }
}
