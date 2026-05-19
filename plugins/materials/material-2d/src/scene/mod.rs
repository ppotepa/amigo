use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn material_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.materials.material-2d.Material2D",
        "materials",
        "Material2D",
    )
}

