use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn text_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.gfx.text-2d.Text2D", "gfx", "Text2D")
}

#[derive(Default)]
pub struct Text2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for Text2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(text_2d_scene_descriptor());
    }
}
