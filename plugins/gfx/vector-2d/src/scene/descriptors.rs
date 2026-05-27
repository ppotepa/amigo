use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn vector_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.gfx.vector-2d.VectorShape2D", "gfx", "VectorShape2D")
}

#[derive(Default)]
pub struct Vector2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for Vector2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(vector_2d_scene_descriptor());
    }
}
