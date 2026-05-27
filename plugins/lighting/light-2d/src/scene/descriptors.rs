use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn global_light_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.lighting.light-2d.GlobalLight2D",
        "lighting",
        "GlobalLight2D",
    )
}

#[derive(Default)]
pub struct Lighting2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for Lighting2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(global_light_2d_scene_descriptor());
    }
}
