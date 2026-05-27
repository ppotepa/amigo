use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn sprite_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.gfx.sprite-2d.Sprite2D", "gfx", "Sprite2D")
}

#[derive(Default)]
pub struct Sprite2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for Sprite2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(sprite_2d_scene_descriptor());
    }
}
