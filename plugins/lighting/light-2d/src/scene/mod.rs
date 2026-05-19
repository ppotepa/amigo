use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn global_light_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.lighting.light-2d.GlobalLight2D",
        "lighting",
        "GlobalLight2D",
    )
}

