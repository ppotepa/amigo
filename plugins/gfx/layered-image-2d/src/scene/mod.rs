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

pub struct LayeredImage2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for LayeredImage2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.gfx.layered-image-2d";
    const COMPONENT_TYPE: &'static str = "amigo.gfx.layered-image-2d.LayeredImage2D";

    type Payload = LayeredImage2dDocument;
    type DescriptorProvider = LayeredImage2dSceneDescriptorProvider;
    type SchemaProvider = LayeredImage2dSceneSchemaProvider;
    type PluginHydrator = LayeredImage2dPluginComponentHydrator;
    type GraphProvider = LayeredImage2dPluginGraphProvider;
    type MetadataProvider = LayeredImage2dComponentMetadataProvider;
}
