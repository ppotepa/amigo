use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn lightmap_2d_source_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.lighting.lightmaps-2d.LightMap2DSource",
        "lighting",
        "LightMap2DSource",
    )
}

