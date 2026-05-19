use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn text_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.gfx.text-2d.Text2D",
        "gfx",
        "Text2D",
    )
}

