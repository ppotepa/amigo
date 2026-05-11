//! 2D vector shape scene services.
//! It stores lines and polygons used by gameplay, debug visualization, and lightweight rendering.

mod model;
mod plugin;
mod runtime_contributions;
mod scene_bridge;
mod service;

#[cfg(test)]
mod tests;

pub use model::*;
pub use plugin::*;
pub use runtime_contributions::*;
pub use scene_bridge::*;
pub use service::*;
