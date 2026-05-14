pub mod bindings;
mod graph;
mod ids;
mod image_parts;
mod inspect;
mod loader;
mod node_descriptors;
mod plugin;
mod prefabs;
mod projections;
mod refs;
mod service;

pub use amigo_editor_api::{
    AuthoringProperty, AuthoringPropertyApplyMode, AuthoringPropertyDisplay,
    AuthoringPropertyEditor, AuthoringPropertyGroup, AuthoringPropertyHints,
    AuthoringPropertyPanel, AuthoringPropertyValue, AuthoringPropertyVisibility,
    AuthoringRuntimeBinding,
};
pub use graph::*;
pub use image_parts::*;
pub use inspect::*;
pub use loader::{load_authoring_scene_graph, load_authoring_scene_graph_from_file};
pub use plugin::EditorAuthoringPlugin;
pub use projections::*;
pub use service::AuthoringSceneGraphService;

#[cfg(test)]
mod tests;
