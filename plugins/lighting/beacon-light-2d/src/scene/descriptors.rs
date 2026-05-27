use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn beacon_light_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.lighting.beacon-light-2d.BeaconLight2D",
        "lighting",
        "BeaconLight2D",
    )
}

#[derive(Default)]
pub struct BeaconLight2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for BeaconLight2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(beacon_light_2d_scene_descriptor());
    }
}
