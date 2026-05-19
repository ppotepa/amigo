use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn light_group_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.lighting.light-groups-2d.LightGroup2D",
        "lighting",
        "LightGroup2D",
    )
}

