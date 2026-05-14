mod graph;
mod ids;
mod inspect;
mod loader;
mod metadata_hints;
mod plugin;
mod prefabs;
mod refs;
mod service;

pub use amigo_editor_api::{
    AuthoringProperty, AuthoringPropertyEditor, AuthoringPropertyGroup, AuthoringPropertyPanel,
    AuthoringPropertyValue, AuthoringRuntimeBinding,
};
pub use graph::*;
pub use inspect::*;
pub use loader::{load_authoring_scene_graph, load_authoring_scene_graph_from_file};
pub use plugin::EditorAuthoringPlugin;
pub use service::AuthoringSceneGraphService;

#[cfg(test)]
mod tests;
