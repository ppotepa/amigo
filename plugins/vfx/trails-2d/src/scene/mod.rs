use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn trail_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.vfx.trails-2d.Trail2D", "vfx", "Trail2D")
}
