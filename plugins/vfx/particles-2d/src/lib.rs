//! 2D particle runtime and authored emitter services.
//! It evaluates emitter configs, spawns particles, and exposes runtime controls to scripts and tools.

pub mod api;
mod dev_console;
mod model;
pub mod participation;
mod plugin;
mod render_extraction;
mod runtime;
mod runtime_capabilities;
pub mod scene;
mod scene_bridge;
mod scene_command;
mod service;
mod systems;

pub use dev_console::*;
pub use model::*;
pub use plugin::*;
pub use render_extraction::*;
pub use runtime_capabilities::*;
pub use scene_bridge::*;
pub use scene_command::*;
pub use service::*;
pub use systems::*;

#[cfg(test)]
mod tests;
