use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn camera_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.camera.camera-core.Camera2D", "camera", "Camera2D")
}

#[derive(Default)]
pub struct Camera2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for Camera2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(camera_2d_scene_descriptor());
    }
}
