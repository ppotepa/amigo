use amigo_plugin_api::PluginSceneComponentDescriptor;

pub mod document;

pub fn composite_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.postfx.composite.Composite", "postfx", "Composite")
}
