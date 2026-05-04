//! 2D vector shape scene services.
//! It stores lines and polygons used by gameplay, debug visualization, and lightweight rendering.

mod model;
mod plugin;
mod scene_bridge;
mod service;

#[cfg(test)]
mod tests;

pub use model::*;
pub use plugin::*;
pub use scene_bridge::*;
pub use service::*;
