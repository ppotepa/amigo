use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn sprite_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.gfx.sprite-2d.Sprite2D",
        "gfx",
        "Sprite2D",
    )
}

