//! Typed scene component for the NPR playground. The scene intentionally owns
//! drawing intent, while `amigo-render-npr` remains backend-agnostic.

mod command;
mod descriptors;
mod document;
mod graph;
mod hydration;
mod metadata;

pub use command::*;
pub use document::*;

pub struct NprPlaygroundSceneComponentSpec;

impl amigo_scene::SceneComponentPluginSpec for NprPlaygroundSceneComponentSpec {
    const PLUGIN_ID: &'static str = "amigo.gfx.npr-playground";
    const COMPONENT_TYPE: &'static str = "amigo.gfx.npr-playground.NprSettings";

    type Payload = NprPlaygroundSceneDocument;
    type DescriptorProvider = descriptors::NprPlaygroundSceneDescriptorProvider;
    type SchemaProvider = document::NprPlaygroundSceneSchemaProvider;
    type PluginHydrator = hydration::NprPlaygroundPluginComponentHydrator;
    type GraphProvider = graph::NprPlaygroundPluginGraphProvider;
    type MetadataProvider = metadata::NprPlaygroundComponentMetadataProvider;
}
