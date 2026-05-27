pub mod descriptors;
pub mod document;
pub mod graph;
pub mod hydration;
mod metadata;

pub use metadata::Sprite2dComponentMetadataProvider;

pub use descriptors::*;
pub use document::*;
pub use graph::*;
pub use hydration::*;
pub use metadata::*;

pub struct Sprite2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for Sprite2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.gfx.sprite-2d";
    const COMPONENT_TYPE: &'static str = "amigo.gfx.sprite-2d.Sprite2D";

    type Payload = Sprite2dDocument;
    type DescriptorProvider = Sprite2dSceneDescriptorProvider;
    type SchemaProvider = Sprite2dSceneSchemaProvider;
    type PluginHydrator = Sprite2dPluginComponentHydrator;
    type GraphProvider = Sprite2dPluginGraphProvider;
    type MetadataProvider = Sprite2dComponentMetadataProvider;
}
