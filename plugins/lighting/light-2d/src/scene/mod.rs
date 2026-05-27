mod descriptors;
mod document;
mod graph;
mod hydration;
mod metadata;

pub use descriptors::*;
pub use document::*;
pub use graph::*;
pub use hydration::*;
pub use metadata::*;

pub struct GlobalLight2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for GlobalLight2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.lighting.light-2d";
    const COMPONENT_TYPE: &'static str = "amigo.lighting.light-2d.GlobalLight2D";

    type Payload = GlobalLight2dDocument;
    type DescriptorProvider = Lighting2dSceneDescriptorProvider;
    type SchemaProvider = GlobalLight2dSceneSchemaProvider;
    type PluginHydrator = GlobalLight2dPluginComponentHydrator;
    type GraphProvider = GlobalLight2dPluginGraphProvider;
    type MetadataProvider = GlobalLight2dComponentMetadataProvider;
}
