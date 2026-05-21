//! Domain runtime for `BeaconLight2D`.
//!
//! `engine/scene` owns the serialized component document and scene command shape.
//! This crate owns the 2D beacon runtime service, script controls, scene command
//! handling, and render extraction for the resolved beacon VFX.

mod control;
mod editor_capability;
mod model;
mod plugin;
mod reset;
mod runtime_capabilities;
mod scene_bridge;
mod scene_command;
mod script_command;
mod service;

pub use control::*;
pub use editor_capability::*;
pub use model::*;
pub use plugin::*;
pub use reset::*;
pub use runtime_capabilities::*;
pub use scene_bridge::*;
pub use scene_command::*;
pub use script_command::*;
pub use service::*;

#[cfg(test)]
mod tests;
