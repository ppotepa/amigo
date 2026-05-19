use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn tilemap_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.gfx.tilemap-2d.TileMap2D",
        "gfx",
        "TileMap2D",
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tilemap2dDocument {
    pub entity_name: String,
    pub render_layer: String,
}
