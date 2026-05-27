pub mod descriptors;
pub mod document;
pub mod graph;
pub mod hydration;
pub mod metadata;

pub use descriptors::*;
pub use document::*;
pub use graph::*;
pub use hydration::*;
pub use metadata::*;

pub struct ParticleEmitter2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for ParticleEmitter2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.vfx.particles-2d";
    const COMPONENT_TYPE: &'static str = "amigo.vfx.particles-2d.ParticleEmitter2D";

    type Payload = ParticleEmitter2dDocument;
    type DescriptorProvider = ParticleEmitter2dSceneDescriptorProvider;
    type SchemaProvider = ParticleEmitter2dSceneSchemaProvider;
    type PluginHydrator = ParticleEmitter2dPluginComponentHydrator;
    type GraphProvider = ParticleEmitter2dPluginGraphProvider;
    type MetadataProvider = ParticleEmitter2dComponentMetadataProvider;
}
