use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn bloom_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.postfx.bloom.Bloom", "postfx", "Bloom")
}
