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

pub struct DepthMap2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for DepthMap2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.camera.focus-depth";
    const COMPONENT_TYPE: &'static str = "amigo.camera.focus-depth.DepthMap2D";

    type Payload = DepthMap2dDocument;
    type DescriptorProvider = DepthMap2dSceneDescriptorProvider;
    type SchemaProvider = DepthMap2dSceneSchemaProvider;
    type PluginHydrator = DepthMap2dPluginComponentHydrator;
    type GraphProvider = DepthMap2dPluginGraphProvider;
    type MetadataProvider = DepthMap2dComponentMetadataProvider;
}

pub struct DepthAuxMap2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for DepthAuxMap2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.camera.focus-depth";
    const COMPONENT_TYPE: &'static str = "amigo.camera.focus-depth.DepthAuxMap2D";

    type Payload = DepthAuxMap2dDocument;
    type DescriptorProvider = DepthAuxMap2dSceneDescriptorProvider;
    type SchemaProvider = DepthAuxMap2dSceneSchemaProvider;
    type PluginHydrator = DepthAuxMap2dPluginComponentHydrator;
    type GraphProvider = DepthAuxMap2dPluginGraphProvider;
    type MetadataProvider = DepthAuxMap2dComponentMetadataProvider;
}
