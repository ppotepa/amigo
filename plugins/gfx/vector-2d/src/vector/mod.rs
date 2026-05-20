//! 2D vector shape scene services.
//! It stores lines and polygons used by gameplay, debug visualization, and lightweight rendering.

mod model;
mod plugin;
mod render_extraction;
mod reset;
mod runtime_capabilities;
mod scene_bridge;
mod scene_command;
mod service;

mod editor_capability;
#[cfg(test)]
mod tests;

pub use editor_capability::*;
pub use model::*;
pub use plugin::*;
pub use render_extraction::*;
pub use reset::*;
pub use runtime_capabilities::*;
pub use scene_bridge::*;
pub use scene_command::*;
pub use service::*;
