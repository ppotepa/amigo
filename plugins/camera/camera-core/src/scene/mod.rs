pub mod commands;
pub mod descriptors;
pub mod document;
pub mod graph;
pub mod hydration;
mod metadata;

pub use commands::*;
pub use descriptors::*;
pub use document::*;
pub use graph::*;
pub use hydration::*;
pub use metadata::*;

pub struct Camera2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for Camera2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.camera.camera-core";
    const COMPONENT_TYPE: &'static str = "amigo.camera.camera-core.Camera2D";

    type Payload = Camera2dDocument;
    type DescriptorProvider = Camera2dSceneDescriptorProvider;
    type SchemaProvider = Camera2dSceneSchemaProvider;
    type PluginHydrator = Camera2dPluginComponentHydrator;
    type GraphProvider = Camera2dPluginGraphProvider;
    type MetadataProvider = Camera2dComponentMetadataProvider;
}
