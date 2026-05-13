//! 2D vector shape scene services.
//! It stores lines and polygons used by gameplay, debug visualization, and lightweight rendering.

mod model;
mod plugin;
mod render_extraction;
mod runtime_capabilities;
mod scene_command;
mod scene_bridge;
mod service;

#[cfg(test)]
mod tests;
mod editor_capability;

pub use model::*;
pub use editor_capability::*;
pub use plugin::*;
pub use render_extraction::*;
pub use runtime_capabilities::*;
pub use scene_command::*;
pub use scene_bridge::*;
pub use service::*;

