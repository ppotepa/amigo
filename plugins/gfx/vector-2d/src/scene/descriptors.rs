use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn vector_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.gfx.vector-2d.VectorShape2D",
        "gfx",
        "VectorShape2D",
    )
}

