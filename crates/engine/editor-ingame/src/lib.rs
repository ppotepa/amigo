mod bounds;
mod commands;
mod component_registry;
mod input;
mod inspect;
pub mod layout;
mod overlay;
mod plugin;
mod provider_registry;
mod properties;
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
