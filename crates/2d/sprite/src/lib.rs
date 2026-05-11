//! 2D sprite scene services and commands.
//! It stores sprite render state hydrated from scene documents and mutated by runtime systems and scripts.

mod model;
mod plugin;
mod runtime_contributions;
mod scene_bridge;
mod service;

pub use model::*;
pub use plugin::*;
pub use runtime_contributions::*;
pub use scene_bridge::*;
pub use service::*;

#[cfg(test)]
mod tests;
