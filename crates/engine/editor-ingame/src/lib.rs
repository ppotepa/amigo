mod bounds;
mod commands;
mod input;
pub mod layout;
mod overlay;
mod plugin;
mod properties;
mod runtime_apply;
mod selection;
mod state;
mod theme;

pub use input::handle_editor_input;
pub use overlay::append_editor_overlay;
pub use plugin::IngameEditorPlugin;
pub use state::IngameEditorState;

#[cfg(test)]
mod tests;
