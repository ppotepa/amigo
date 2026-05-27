use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn tilemap_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.gfx.tilemap-2d.TileMap2D", "gfx", "TileMap2D")
}

#[derive(Default)]
pub struct TileMap2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for TileMap2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(tilemap_2d_scene_descriptor());
    }
}
