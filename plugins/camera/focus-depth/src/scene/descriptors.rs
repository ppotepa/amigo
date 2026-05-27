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

#[derive(Default)]
pub struct DepthMap2dSceneDescriptorProvider;

impl amigo_scene::ScenePluginDescriptorProvider for DepthMap2dSceneDescriptorProvider {
    fn register_scene_descriptors(
        &self,
        registry: &mut amigo_scene::ScenePluginDescriptorRegistry,
    ) {
        registry.insert(depth_map_2d_scene_descriptor());
    }
}

#[derive(Default)]
pub struct DepthAuxMap2dSceneDescriptorProvider;

impl amigo_scene::ScenePluginDescriptorProvider for DepthAuxMap2dSceneDescriptorProvider {
    fn register_scene_descriptors(
        &self,
        registry: &mut amigo_scene::ScenePluginDescriptorRegistry,
    ) {
        registry.insert(depth_aux_map_2d_scene_descriptor());
    }
}
