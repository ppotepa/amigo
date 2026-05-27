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

pub struct TileMap2dSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for TileMap2dSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.gfx.tilemap-2d";
    const COMPONENT_TYPE: &'static str = "amigo.gfx.tilemap-2d.TileMap2D";

    type Payload = Tilemap2dDocument;
    type DescriptorProvider = TileMap2dSceneDescriptorProvider;
    type SchemaProvider = TileMap2dSceneSchemaProvider;
    type PluginHydrator = TileMap2dPluginComponentHydrator;
    type GraphProvider = TileMap2dPluginGraphProvider;
    type MetadataProvider = TileMap2dComponentMetadataProvider;
}
