//! Shared audio scene and state contracts used across the engine.
//! It defines clips, commands, queues, and services for playback control.

mod plugin;
mod services;
mod types;

#[cfg(test)]
mod tests;

pub use plugin::*;
pub use services::*;
pub use types::*;
