use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn camera_profile_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.camera-profiles.CameraProfile",
        "camera",
        "CameraProfile",
    )
}
