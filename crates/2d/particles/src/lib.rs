//! 2D particle runtime and authored emitter services.
//! It evaluates emitter configs, spawns particles, and exposes runtime controls to scripts and tools.

mod model;
mod plugin;
mod runtime;
mod scene_bridge;
mod service;

pub use model::*;
pub use plugin::*;
pub use scene_bridge::*;
pub use service::*;

#[cfg(test)]
mod tests;
