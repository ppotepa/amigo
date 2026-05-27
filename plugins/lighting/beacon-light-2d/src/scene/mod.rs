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

pub struct BeaconLight2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for BeaconLight2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.lighting.beacon-light-2d";
    const COMPONENT_TYPE: &'static str = "amigo.lighting.beacon-light-2d.BeaconLight2D";

    type Payload = BeaconLight2dDocument;
    type DescriptorProvider = BeaconLight2dSceneDescriptorProvider;
    type SchemaProvider = BeaconLight2dSceneSchemaProvider;
    type PluginHydrator = BeaconLight2dPluginComponentHydrator;
    type GraphProvider = BeaconLight2dPluginGraphProvider;
    type MetadataProvider = BeaconLight2dComponentMetadataProvider;
}
