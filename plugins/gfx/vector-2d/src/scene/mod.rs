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

pub struct Vector2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for Vector2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.gfx.vector-2d";
    const COMPONENT_TYPE: &'static str = "amigo.gfx.vector-2d.VectorShape2D";

    type Payload = Vector2dDocument;
    type DescriptorProvider = Vector2dSceneDescriptorProvider;
    type SchemaProvider = Vector2dSceneSchemaProvider;
    type PluginHydrator = Vector2dPluginComponentHydrator;
    type GraphProvider = Vector2dPluginGraphProvider;
    type MetadataProvider = Vector2dComponentMetadataProvider;
}
