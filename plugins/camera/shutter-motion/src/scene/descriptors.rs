use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn shutter_motion_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.shutter-motion.ShutterMotion2D",
        "camera",
        "ShutterMotion2D",
    )
}

