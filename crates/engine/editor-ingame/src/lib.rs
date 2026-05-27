mod bounds;
mod commands;
mod component_registry;
mod input;
mod inspect;
pub mod layout;
mod overlay;
mod plugin;
mod properties;
mod provider_registry;
mod runtime_apply;
mod selection;
mod state;
mod theme;

pub use input::handle_editor_input;
pub use overlay::{EditorOverlayRenderOutput, append_editor_overlay};
pub use plugin::IngameEditorPlugin;
pub use provider_registry::*;
pub use state::IngameEditorState;

#[cfg(test)]
mod tests;
