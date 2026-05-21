use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn focus_depth_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.focus-depth.FocusDepth2D",
        "camera",
        "FocusDepth2D",
    )
}

pub fn depth_map_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.focus-depth.DepthMap2D",
        "camera",
        "DepthMap2D",
    )
}

pub fn depth_aux_map_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.focus-depth.DepthAuxMap2D",
        "camera",
        "DepthAuxMap2D",
    )
}

pub struct FocusDepthSceneDescriptorProvider;

impl amigo_scene::ScenePluginDescriptorProvider for FocusDepthSceneDescriptorProvider {
    fn register_scene_descriptors(
        &self,
        registry: &mut amigo_scene::ScenePluginDescriptorRegistry,
    ) {
        registry.insert(focus_depth_2d_scene_descriptor());
        registry.insert(depth_map_2d_scene_descriptor());
        registry.insert(depth_aux_map_2d_scene_descriptor());
    }
}
