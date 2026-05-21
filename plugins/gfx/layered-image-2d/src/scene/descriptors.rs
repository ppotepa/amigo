use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn layered_image_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.gfx.layered-image-2d.LayeredImage2D",
        "gfx",
        "LayeredImage2D",
    )
}

pub struct LayeredImage2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for LayeredImage2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(layered_image_2d_scene_descriptor());
    }
}
