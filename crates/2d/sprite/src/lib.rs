//! 2D sprite scene services and commands.
//! It stores sprite render state hydrated from scene documents and mutated by runtime systems and scripts.

mod model;
mod plugin;
mod render_extraction;
mod runtime_capabilities;
mod scene_command;
mod scene_bridge;
mod script_command;
mod service;

pub use model::*;
pub use plugin::*;
pub use render_extraction::*;
pub use runtime_capabilities::*;
pub use scene_command::*;
pub use scene_bridge::*;
pub use script_command::*;
pub use service::*;

#[cfg(test)]
mod tests;
