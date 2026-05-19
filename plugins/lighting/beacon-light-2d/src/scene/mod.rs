use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn beacon_light_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.lighting.beacon-light-2d.BeaconLight2D",
        "lighting",
        "BeaconLight2D",
    )
}

