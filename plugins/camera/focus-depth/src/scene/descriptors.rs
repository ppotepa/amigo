use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn focus_depth_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.focus-depth.FocusDepth2D",
        "camera",
        "FocusDepth2D",
    )
}

