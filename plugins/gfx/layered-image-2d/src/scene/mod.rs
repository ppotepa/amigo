use amigo_plugin_api::PluginSceneComponentDescriptor;

mod metadata;

pub use metadata::*;

pub fn layered_image_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.gfx.layered-image-2d.LayeredImage2D",
        "gfx",
        "LayeredImage2D",
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayeredImage2dDocument {
    pub entity_name: String,
    pub layers: Vec<crate::api::LayeredImage2dLayer>,
}
