pub mod descriptors;
pub mod document;
pub mod graph;
pub mod hydration;
mod metadata;

pub use descriptors::*;
pub use document::*;
pub use graph::*;
pub use hydration::*;
pub use metadata::*;

pub struct Text2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for Text2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.gfx.text-2d";
    const COMPONENT_TYPE: &'static str = "amigo.gfx.text-2d.Text2D";

    type Payload = Text2dDocument;
    type DescriptorProvider = Text2dSceneDescriptorProvider;
    type SchemaProvider = Text2dSceneSchemaProvider;
    type PluginHydrator = Text2dPluginComponentHydrator;
    type GraphProvider = Text2dPluginGraphProvider;
    type MetadataProvider = Text2dComponentMetadataProvider;
}
