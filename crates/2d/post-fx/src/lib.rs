//! Shared 2D post-processing effect models.
//! Renderer integrations consume these data types; authored domains decide where to attach them.

mod model;
mod plugin;
mod runtime_capabilities;
mod service;

pub use model::*;
pub use plugin::*;
pub use runtime_capabilities::*;
pub use service::*;
